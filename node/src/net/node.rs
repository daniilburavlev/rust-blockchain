use crate::chain::block::Block;
use crate::chain::blockchain::Blockchain;
use crate::chain::config::Config;
use crate::chain::tx::Tx;
use crate::net::behaviour::{
    BlockRequest, BlockResponse, NodeBehaviour, NodeBehaviourEvent, NonceRequest, NonceResponse,
    TxResponse,
};
use futures::StreamExt;
use libp2p::gossipsub::IdentTopic;
use libp2p::identity::Keypair;
use libp2p::swarm::SwarmEvent;
use libp2p::{
    StreamProtocol, Swarm, gossipsub, identity, mdns, noise, request_response, tcp, yamux,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc;
use tokio_cron_scheduler::{Job, JobScheduler};
use wallet::wallet::Wallet;
use crate::net::client::Client;

pub struct Node {
    port: i64,
    swarm: Swarm<NodeBehaviour>,
    blockchain: Arc<Blockchain>,
    tx_topic: IdentTopic,
    block_topic: IdentTopic,
}

impl Node {
    pub fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        println!("Enter password:");
        let password = rpassword::read_password().unwrap();
        let wallet = Wallet::read(
            &config.keystore_path(),
            &config.validator(),
            password.as_bytes(),
        )?;
        Ok(Self {
            port: config.port(),
            swarm: Self::build_swarm(&wallet)?,
            blockchain: Arc::new(Blockchain::new(wallet.clone(), &config)?),
            tx_topic: IdentTopic::new("txs"),
            block_topic: IdentTopic::new("block"),
        })
    }

    fn build_swarm(validator: &Wallet) -> Result<Swarm<NodeBehaviour>, Box<dyn std::error::Error>> {
        let secret = identity::ecdsa::SecretKey::try_from_bytes(validator.secret())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let keypair = identity::ecdsa::Keypair::from(secret);
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .try_init();
        let swarm = libp2p::SwarmBuilder::with_existing_identity(Keypair::from(keypair))
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_quic()
            .with_behaviour(|key| {
                let gossibsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .build()
                    .map_err(tokio::io::Error::other)?;
                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossibsub_config,
                )?;
                let mdns = mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?;
                let nonce_behaviour =
                    request_response::json::Behaviour::<NonceRequest, NonceResponse>::new(
                        [(
                            StreamProtocol::new("/nonce/0.0.1"),
                            request_response::ProtocolSupport::Full,
                        )],
                        request_response::Config::default(),
                    );
                let tx_behaviour = request_response::json::Behaviour::<Tx, TxResponse>::new(
                    [(
                        StreamProtocol::new("/tx/0.0.1"),
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                );
                let find_block_behaviour =
                    request_response::json::Behaviour::<BlockRequest, BlockResponse>::new(
                        [(
                            StreamProtocol::new("/block/0.0.1"),
                            request_response::ProtocolSupport::Full,
                        )],
                        request_response::Config::default(),
                    );
                Ok(NodeBehaviour {
                    gossipsub,
                    mdns,
                    nonce: nonce_behaviour,
                    tx: tx_behaviour,
                    find_block: find_block_behaviour,
                })
            })?
            .build();
        Ok(swarm)
    }

    pub async fn sync(&self, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        let mut synced = false;
        let mut client = Client::new(config).await?;
        while !synced {
            let latest_block = self.blockchain.find_latest()?;
            match client.find_block_by_idx(latest_block.idx + 1).await {
                Some(block) => {
                    println!("block: {:?}", block);
                    self.blockchain.add_block(&block)?;
                },
                None => synced = true,
            }
        }
        Ok(())
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&self.tx_topic)?;
        self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&self.block_topic)?;

        self.swarm
            .listen_on(format!("/ip4/0.0.0.0/tcp/{}", self.port).parse()?)?;
        self.swarm
            .listen_on(format!("/ip4/0.0.0.0/udp/{}/quic-v1", self.port).parse()?)?;

        let (validator_tx, mut validator_rx) = mpsc::channel::<Block>(100);

        let blockchain = Arc::clone(&self.blockchain);

        let scheduler = JobScheduler::new().await?;
        scheduler
            .add(Job::new_async("*/12 * * * * *", move |_, _| {
                let validator_tx = validator_tx.clone();
                let blockchain = Arc::clone(&blockchain);
                Box::pin(async move {
                    match blockchain.proof_of_stake() {
                        Ok(block) => match validator_tx.send(block).await {
                            Err(e) => println!("Error sending block: {:?}", e),
                            _ => {}
                        },
                        Err(e) => println!("Cannot create block: {}", e),
                    };
                })
            })?)
            .await?;

        scheduler.start().await?;

        loop {
            select! {
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await;
                },
                event = validator_rx.recv() => {
                    if let Some(block) = event {
                        let json = serde_json::to_string(&block)?;
                        println!("{}", serde_json::to_string_pretty(&json)?);
                        if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(
                            self.block_topic.clone(),
                            json
                        ) {
                            println!("Error publishing block: {:?}", e);
                        }
                    }
                }
            }
        }
    }

    async fn handle_swarm_event(&mut self, event: SwarmEvent<NodeBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(NodeBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, _multiaddr) in list {
                    println!("mDNS discovered a new peer: {}", peer_id);
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .add_explicit_peer(&peer_id);
                }
            }
            SwarmEvent::Behaviour(NodeBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                for (peer_id, _multiaddr) in list {
                    println!("mDNS discovered peer has expired: {}", peer_id);
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .remove_explicit_peer(&peer_id);
                }
            }
            SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source: _,
                message_id: _,
                message,
            })) => {
                self.process_topic_message(&message).await;
            }
            SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(
                gossipsub::Event::Subscribed { peer_id, topic },
            )) => {
                println!("subscribed {} to {}", peer_id, topic);
            }
            SwarmEvent::NewListenAddr { address, .. } => println!("Node started: {}", address),
            SwarmEvent::Behaviour(NodeBehaviourEvent::Nonce(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request {
                    request_id: _,
                    request,
                    channel,
                } => {
                    let nonce = self.blockchain.nonce(request.address.clone());
                    self.swarm
                        .behaviour_mut()
                        .nonce
                        .send_response(
                            channel,
                            NonceResponse {
                                nonce: nonce.unwrap(),
                            },
                        )
                        .unwrap();
                }
                request_response::Message::Response {
                    request_id: _,
                    response: _,
                } => {}
            },
            SwarmEvent::Behaviour(NodeBehaviourEvent::Tx(request_response::Event::Message {
                message,
                ..
            })) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    let response = match self.blockchain.add_tx(&request) {
                        Ok(_) => {
                            let json = serde_json::to_string(&request).unwrap();
                            if let Err(e) = self
                                .swarm
                                .behaviour_mut()
                                .gossipsub
                                .publish(self.tx_topic.clone(), json)
                            {
                                println!("Error publishing to swarm: {:?}", e);
                            }
                            TxResponse { error: None }
                        }
                        Err(e) => TxResponse {
                            error: Some(format!("{:?}", e)),
                        },
                    };
                    if let Err(e) = self
                        .swarm
                        .behaviour_mut()
                        .tx
                        .send_response(channel, response)
                    {
                        println!("Error sending response: {:?}", e);
                    }
                }
                _ => {}
            },
            SwarmEvent::Behaviour(NodeBehaviourEvent::FindBlock(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    if let Ok(block) = self.blockchain.find_block_by_idx(request.idx) {
                        let response = BlockResponse { block };
                        if let Err(e) = self
                            .swarm
                            .behaviour_mut()
                            .find_block
                            .send_response(channel, response)
                        {
                            println!("Error sending response: {:?}", e);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    async fn process_topic_message(&self, message: &gossipsub::Message) {
        let topic = message.topic.clone();
        if topic == self.tx_topic.hash() {
            let tx: Tx =
                serde_json::from_str(String::from_utf8(message.clone().data).unwrap().as_str())
                    .unwrap();
            if let Err(e) = self.blockchain.add_tx(&tx) {
                println!("Error sending message: {:?}", e);
            }
        } else if topic == self.block_topic.hash() {
            let block: Block =
                serde_json::from_str(String::from_utf8(message.clone().data).unwrap().as_str())
                    .unwrap();
            match self.blockchain.add_block(&block) {
                Err(e) => println!("Error adding block: {:?}", e),
                _ => {}
            }
        }
    }
}
