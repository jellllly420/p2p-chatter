use futures::StreamExt;
use libp2p::{
    core::upgrade,
    floodsub::{self, Floodsub, FloodsubEvent},
    identity,
    mdns::{Mdns, MdnsEvent},
    noise,
    swarm::{SwarmBuilder, SwarmEvent},
    tcp::TokioTcpConfig,
    Multiaddr,
    PeerId,
    Transport,
    yamux,
};
use log::info;
use p2p_chatter::{
    MyBehaviour,
    OutEvent,
};
use std::error::Error;
use tokio::io::{
    self, 
    AsyncBufReadExt,
};

// Set up a tokio runtime for async Rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Init a env_logger for debug
    env_logger::init();

    // Create a peer id from the public key of a Secp256k1 keypair
    let key_pair = identity::Keypair::generate_secp256k1();
    let local_peer_id = PeerId::from(key_pair.public());
    info!("Local peer id: {:?}", local_peer_id);

    // Create a key pair for encryption
    let noise_key_pair = noise::Keypair::<noise::X25519Spec>::new()
                                                             .into_authentic(&key_pair)
                                                             .expect("Failed while turning the key_pair into a AuthenticKeypair");
    
    // Create a tokio TCP transport
    let transport = TokioTcpConfig::new()
                                   .nodelay(true)
                                   .upgrade(upgrade::Version::V1)
                                   .authenticate(noise::NoiseConfig::xx(noise_key_pair).into_authenticated())
                                   .multiplex(yamux::YamuxConfig::default())
                                   .boxed();

    // Create a Floodsub topic
    let topic = floodsub::Topic::new("chat");

    // Create a Swarm to manage peers and events
    let mut swarm = {
        let mdns = Mdns::new(Default::default()).await?;
        let mut behaviour = MyBehaviour {
            floodsub: Floodsub::new(local_peer_id.clone()),
            mdns,
            ignored_member: false,
        };

        behaviour.floodsub.subscribe(topic.clone());

        SwarmBuilder::new(transport, behaviour, local_peer_id)
                     .executor(Box::new(|fut| {
                        tokio::spawn(fut);
                     }))
                     .build()

    };

    // Dial to another node (Optional)
    if let Some(raw_addr) = std::env::args().nth(1) {
        let addr: Multiaddr = raw_addr.parse()?;
        swarm.dial(addr)?;
        info!("Dialed to {:?}", raw_addr);
    }

    // Create a line reader
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Listen on all available network interfaces
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Response to events
    loop{
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("Stdin closed");
                swarm.behaviour_mut()
                     .floodsub
                     .publish(topic.clone(), line.as_bytes());
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on {:?}", address);
                }
                SwarmEvent::Behaviour(OutEvent::Floodsub(FloodsubEvent::Message(message))) => {
                    info!("Received: '{:?}' from {:?}",
                          String::from_utf8_lossy(&message.data),
                          message.source);
                }
                SwarmEvent::Behaviour(OutEvent::Mdns(MdnsEvent::Discovered(list))) => {
                    for (peer, _) in list {
                        swarm.behaviour_mut()
                             .floodsub
                             .add_node_to_partial_view(peer);
                    }
                }
                SwarmEvent::Behaviour(OutEvent::Mdns(MdnsEvent::Expired(list))) => {
                    for (peer, _) in list {
                        if !swarm.behaviour_mut().mdns.has_node(&peer) {
                            swarm.behaviour_mut()
                                 .floodsub
                                 .remove_node_from_partial_view(&peer);
                        }
                    }
                },
                _ => {}
            }
        }
    }
}