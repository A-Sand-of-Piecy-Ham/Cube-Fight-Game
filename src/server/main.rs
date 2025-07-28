use bevy::{ecs::spawn, prelude::*};
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, time::Duration};
use bevy_rand::prelude::*;

use monkey::core::player::{shared_spawn_player, PlayerId, PlayerBundle, color_from_id};

const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 25565);
pub const SEND_INTERVAL: Duration = Duration::from_millis(100);

pub struct ServerPlugin;

impl Plugin for ServerPlugin{

    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, init_server);
        app.add_plugins(EntropyPlugin::<WyRand>::default());
        app.add_observer(handle_new_client);
        app.add_observer(spawn_connected_player);
        // app.add_observer(handle_connected);
        // app.add_systems(FixedUpdate, movement);
    }
}

// fn init_server(commands: Commands) {
//     let server = commands
//         .spawn((
//             NetcodeServer::new(NetcodeConfig::default()),
//             LocalAddr(SERVER_ADDR),
//             ServerUdpIo::default(),
//         ))
//         .id();
//     commands.trigger_targets(Start, server);
// }


pub(crate) fn handle_new_client(trigger: Trigger<OnAdd, LinkOf>, mut commands: Commands) {
    commands.entity(trigger.target()).insert((
        ReplicationSender::new(SEND_INTERVAL, SendUpdatesMode::SinceLastAck, false),
        Name::from("Client"),
    ));
}

// pub(crate) fn handle_connected(
//     trigger: Trigger<OnAdd, Connected>,
//     query: Query<&RemoteId, With<ClientOf>>,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut rng: GlobalEntropy<WyRand>,
// ) {
//     let Ok(client_id) = query.get(trigger.target()) else { return; };


//     let client_id = client_id.0;
//     let entity = spawn_player(client_id, &mut commands, &mut meshes, &mut materials, &mut rng);
//     // let entity = commands
//     //     .spawn((
//     //         // PlayerBundle::new(client_id, Vec2::ZERO),
            
//     //         // we replicate the Player entity to all clients that are connected to this server
//     //         Replicate::to_clients(NetworkTarget::All),
//     //     ))
//     //     .id();
//     info!(
//         "Create player entity {:?} for client {:?}",
//         entity, client_id
//     );
// }

fn spawn_connected_player(
    trigger: Trigger<OnAdd, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
    replicated_players: Query<
        (Entity, &InitialReplicated),
        (Added<InitialReplicated>, With<PlayerId>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(client_id) = query.get(trigger.target()) else { return; };

    let client_id = client_id.0;
    let color = color_from_id(client_id);
    let entity = commands
        .spawn((PlayerBundle::new(
            client_id, 
            Transform::from_xyz(0.0, 4.0, 0.0),
            color,
            // TODO: REUSE PLAYER MESHES!!!
            &mut meshes,
            &mut materials,
        ),
        
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::Single(client_id)),
            InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(client_id)),
            ControlledBy {
                owner: trigger.target(),
                // TODO: implement reconnecting on disconnect
                lifetime: Lifetime::SessionBased,
            },
            PlayerId(client_id),
            // PlayerBundle::new(client_id, Vec2::ZERO),

            // we replicate the Player entity to all clients that are connected to this server
            Replicate::to_clients(NetworkTarget::All),
        )).id();
    info!(
        "Created player entity {:?} for client {:?}",
        entity, client_id
    );
}


fn main() {
    // App::new()
    //     .add_plugins(DefaultPlugins)
    //     .add_plugins(ProtocolPlugin)
    //     .add_plugins(ServerPlugin)
    //     .run();
}
