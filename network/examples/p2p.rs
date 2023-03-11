use anyhow::Result;
use futures::StreamExt;
use libp2p::{
    core::upgrade,
    gossipsub::{
        self, Gossipsub, GossipsubConfigBuilder, GossipsubEvent, MessageAuthenticity, Sha256Topic,
    },
    identity,
    mdns::{Mdns, MdnsEvent},
    noise,
    swarm::{NetworkBehaviourEventProcess, SwarmBuilder, SwarmEvent},
    tcp::TokioTcpConfig,
    yamux, NetworkBehaviour, PeerId, Swarm, Transport,
};
use std::borrow::Cow;
use tokio::io::{stdin, AsyncBufReadExt, BufReader};

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
struct ChatBehavior {
    sub: Gossipsub,
    mdns: Mdns,
}

impl ChatBehavior {
    pub async fn new(id: PeerId) -> Result<Self> {
        Ok(Self {
            mdns: Mdns::new(Default::default()).await?,
            sub: Gossipsub::new(
                MessageAuthenticity::Author(id),
                GossipsubConfigBuilder::default()
                    .validation_mode(gossipsub::ValidationMode::Permissive)
                    .build()
                    .unwrap(),
            )
            .unwrap(),
        })
    }
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for ChatBehavior {
    fn inject_event(&mut self, event: GossipsubEvent) {
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = event
        {
            let text = String::from_utf8_lossy(&message.data);
            println!("{:?}: {:?}", message.source, text);
        }
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for ChatBehavior {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (id, addr) in list {
                    println!("got peer: {} with addr {}", &id, &addr);
                    self.sub.add_explicit_peer(&id);
                }
            }
            MdnsEvent::Expired(list) => {
                for (id, addr) in list {
                    println!("removed peer: {} with addr {}", &id, &addr);
                    self.sub.remove_explicit_peer(&id);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let name = match std::env::args().nth(1) {
        Some(arg) => Cow::Owned(arg),
        None => Cow::Borrowed("test_topic"),
    };

    let topic = gossipsub::Topic::new(name);

    let mut swarm = create_swarm(topic.clone()).await?;

    swarm.listen_on("/ip4/127.0.0.1/tcp/0".parse()?)?;

    let mut stdin = BufReader::new(stdin()).lines();

    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                swarm.behaviour_mut().sub.publish(topic.clone(), line.as_bytes()).unwrap();
            }
            event = swarm.select_next_some() => {
                if let SwarmEvent::NewListenAddr { address, .. } = event {
                    println!("listening on {:?}", address);
                }
            }
        }
    }
}

async fn create_swarm(topic: Sha256Topic) -> Result<Swarm<ChatBehavior>> {
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    println!("peer id: {:?}", peer_id);

    let noise_keys = noise::Keypair::<noise::X25519Spec>::new().into_authentic(&id_keys)?;

    let transport = TokioTcpConfig::new()
        .nodelay(true)
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(yamux::YamuxConfig::default())
        .boxed();

    let mut behavior = ChatBehavior::new(peer_id).await?;
    behavior.sub.subscribe(&topic).unwrap();
    let swarm = SwarmBuilder::new(transport, behavior, peer_id)
        .executor(Box::new(|fut| {
            tokio::spawn(fut);
        }))
        .build();

    Ok(swarm)
}
