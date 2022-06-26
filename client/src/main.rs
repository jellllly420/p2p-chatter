use std::{
    convert::TryInto,
    error::Error,
    net::Ipv4Addr,
    str::FromStr,
};
use log::info;
use clap::Parser;
use futures::{
    executor::block_on,
    future::FutureExt,
    stream::StreamExt,
};
use libp2p::{
    core::{
        multiaddr::{Multiaddr, Protocol},
        transport::OrTransport,
        upgrade,
    },
    dcutr,
    dns::DnsConfig,
    identify::{Identify, IdentifyConfig, IdentifyEvent, IdentifyInfo},
    noise,
    ping::{Ping, PingConfig, PingEvent},
    relay::v2::client::{self, Client},
    swarm::{SwarmBuilder, SwarmEvent},
    tcp::TcpConfig,
    Transport,
    {identity, NetworkBehaviour, PeerId},
};


#[derive(Debug, Parser, PartialEq)]
#[clap(name = "Client Mode")]
enum ClientMode {
    Dialing,
    Listening,
}

impl FromStr for ClientMode {
    type Err = String;
    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        match mode {
            "Dialing" => Ok(ClientMode::Dialing),
            "Listening" => Ok(ClientMode::Listening),
            _ => Err("Dialing/Listening Needed.".to_string()),
        }
    }
}

#[derive(Debug, Parser)]
#[clap(name = "Client Identity")]
struct ClientIdentity {
    #[clap(short, long)]
    mode: ClientMode,

    #[clap(short, long)]
    secret_key_seed: u8,

    #[clap(short, long)]
    relay_address: Multiaddr,

    #[clap(short, long)]
    target_peer_id: Option<PeerId>,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let identity = ClientIdentity::parse();
    let local_key_pair = generate_ed25519(identity.secret_key_seed);
    let local_peer_id = PeerId::from(local_key_pair.public());
    info!("Local Peer ID: {:?}", local_peer_id);

    











    fn generate_ed25519(secret_key_seed: u8) -> identity::Keypair {
        let mut bytes = [0u8; 32];
        bytes[0] = secret_key_seed;
    
        let secret_key_pair = identity::ed25519::SecretKey::from_bytes(&mut bytes)
            .expect("this returns `Err` only if the length is wrong; the length is correct; qed");
        identity::Keypair::Ed25519(secret_key_pair.into())
    }

    Ok(())

}
