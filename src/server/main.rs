// Lightyear server does not compile to any wasm targets, which I suppose makes sense.
use bevy::{asset::AssetMetaCheck, ecs::{component::HookContext, error::info, spawn, world::DeferredWorld}, prelude::*};
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::{netcode::PRIVATE_KEY_BYTES, prelude::server::*};
use lightyear::prelude::*;
use lightyear::netcode::{NetcodeServer};
use lightyear::input::prelude::server::*;
// use lightyear::connection::prelude::*;
use monkey::core::{player::{shared_camera_movement, shared_control_ball, CameraSettings, PlayerInput, ServerCameraBundle}, shared::{SharedSettings, FIXED_TIMESTEP_HZ, SHARED_SETTINGS}};
use serde::{Serialize, Deserialize};
use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, ops::DerefMut, time::Duration};
// use bevy_rand::prelude::*;
#[cfg(not(target_family = "wasm"))]
use bevy::tasks::IoTaskPool;
#[cfg(not(target_family = "wasm"))]
use async_compat::Compat;

use monkey::core::{player::{color_from_id, Action, PlayerBundle, PlayerId, PlayerHandle}, shared::SharedPlugin};
use monkey::core::shared::{SEND_INTERVAL, SERVER_PORT};


pub struct ServerPlugin;

impl Plugin for ServerPlugin{

    fn build(&self, app: &mut App) {
        // app.add_plugins(EntropyPlugin::<WyRand>::default());
        app.add_systems(Startup, setup_level);
        app.add_systems(FixedUpdate, (debug_server_inputs, server_movement).chain());
        app.add_observer(handle_new_client);
        app.add_observer(spawn_connected_player);
        app.add_observer(intercept_camera_creation);
        app.add_observer(debug_camera_added);
        // app.add_observer(handle_connected);
        // app.add_systems(FixedUpdate, server_movement);
        // app.add_plugins(
        //     ServerInputPlugin {
        //         rebroadcast_inputs: true,
        //         ..default() 
        //     }
        // );
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

fn debug_server_inputs(
    query: Query<(&ActionState<Action>, Entity), (Without<Confirmed>, Without<Predicted>)>,
) {
    for (action_state, entity) in query.iter() {
        let has_input = !action_state.get_pressed().is_empty() || 
                       action_state.axis_pair(&Action::Move) != Vec2::ZERO;
        
        if has_input {
            info!(?entity, actions = ?action_state.get_pressed(), move_input = ?action_state.axis_pair(&Action::Move), "Server received input");
        }
    }
}

fn server_movement(
    timeline: Single<&LocalTimeline, With<Server>>,
    mut players: Query<
        (
            &ActionState<Action>,
            &mut Velocity,
            &mut Transform,
            Entity,
        ),
        // if we run in host-server mode, we don't want to apply this system to the local client's entities
        // because they are already moved by the client plugi
        (Without<Confirmed>, Without<Predicted>, Without<CameraSettings>)
    >,
    // cameras: Query<(&Transform, &PlayerHandle), With<CameraSettings>>,
    mut cameras: Query<(&mut Transform, &mut CameraSettings, &ActionState<Action>, &PlayerHandle), Without<PlayerId>>,
    rapier_context: ReadRapierContext,
) {
    let tick = timeline.tick();

    if cameras.is_empty() {
        // info!("No cameras found on server");
    } else {
        info!("Found {} cameras on server", cameras.iter().len());
    }
    
    for (mut camera_transform, mut camera_settings, camera_action_state, player_handle) in cameras.iter_mut() {
        info!(?player_handle, "Processing camera for player");
        let Ok((action_state, velocity, mut player_transform, entity)) = players.get_mut(player_handle.0) else {return;};
         
        let has_input = !action_state.get_pressed().is_empty() || 
                    action_state.axis_pair(&Action::Move) != Vec2::ZERO;
        
        if !camera_action_state.get_pressed().is_empty() {
            // let Ok(player_transform) = players.get(player_handle.0) else {return;};
        
            shared_camera_movement(camera_transform.deref_mut(), camera_settings.deref_mut(), &player_transform.translation, action_state);
        }
        if has_input {
            let camera_direction = camera_transform.forward();
            shared_control_ball(action_state, velocity, &mut player_transform, entity, camera_direction, &rapier_context);
            trace!(?entity, ?tick, actions = ?action_state.get_pressed(), "applying movement to server player");
        }
    }
    

}

fn setup_level(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    
    let camera_input = InputMap::default()
        .with_dual_axis(
            Action::LookAround,
            // You can also use a sequence of processors as the processing pipeline.
            MouseMove::default().replace_processing_pipeline([
                // The first processor is a circular deadzone.
                CircleDeadZone::new(0.1).into(),
                // The next processor doubles inputs normalized by the deadzone.
                DualAxisSensitivity::all(0.1).into(),
            ]),
        )
        .with_axis(Action::Zoom, MouseScrollAxis::Y.sensitivity(if cfg!(target_arch = "wasm32") { 0.1 } else { 1.0 }));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera_input
    ));
    // // Create a camera, and a light so we can see the 3D scene
    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     ..default()
    // });

    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        Replicate::to_clients(NetworkTarget::All),
    ));
    // commands.spawn((
    //     PointLight {
    //         intensity: 1500.0,
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     Transform::from_xyz(4.0, 8.0, 4.0),
    //     Replicate::to_clients(NetworkTarget::All),
    // ));


    // Create the ground
    commands.spawn((
        Collider::cuboid(20.0, 0.1, 20.0), 
        Transform::from_xyz(0.0, -2.0, 0.0),
        Mesh3d(meshes.add(Cuboid::new(40.0, 0.1, 40.0).mesh())),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Replicate::to_clients(NetworkTarget::All),
    ));

}

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

fn handle_server_camera() {todo!()}

fn spawn_connected_player(
    trigger: Trigger<OnAdd, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
    // replicated_players: Query<
    //     (Entity, &InitialReplicated),
    //     (Added<InitialReplicated>, With<PlayerId>),
    // >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(client_id) = query.get(trigger.target()) else { return; };

    let client_id = client_id.0;
    let client = trigger.target();

    let color = color_from_id(client_id);
    let player_entity = commands
        .spawn((PlayerBundle::new(
            client_id, 
            Transform::from_xyz(0.0, 4.0, 0.0),
            color,
        ),
        
            // we replicate the Player entity to all clients that are connected to this server
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::Single(client_id)),
            // PredictionTarget::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(client_id)),
            ControlledBy {
                owner: client,
                // TODO: implement reconnecting on disconnect
                lifetime: Lifetime::SessionBased,
            },
            // PlayerId(client_id),
        ))
        // .insert(PlayerId(client_id))
        .id();

    // commands.spawn((
    //     ServerCameraBundle {
    //         // player_id: client_id,
    //         player: PlayerHandle(player_entity),
    //         transform: Transform::from_xyz(15.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    //         camera_settings: CameraSettings{orbit_distance: 3.0, pitch_speed: 0.3, yaw_speed: 1.0}
    //     },
    //     ControlledBy {
    //         owner: client,
    //         // TODO: implement reconnecting on disconnect
    //         lifetime: Lifetime::SessionBased,
    //     },
    //     Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0).mesh())),
    //     MeshMaterial3d(materials.add(color)),
    // ));



    info!(
        "Created player entity {:?} for client {:?}",
        player_entity, client_id
    );
}

fn debug_camera_added(
    trigger: Trigger<OnAdd, CameraSettings>,
) {
    info!("Camera added to server: {:?}", trigger.target());
}

fn intercept_camera_creation(
    trigger: Trigger<OnAdd, CameraSettings>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<&ControlledBy>,
    player_query: Query<Entity, With<PlayerId>>,
) {
    let camera_entity = trigger.target();
    info!("Camera added to server: {:?}", camera_entity);

    // Get the player entity for this camera's owner
    if let Ok(controlled_by) = camera_query.get(camera_entity) {
        // Find the player entity owned by the same client
        for player_entity in player_query.iter() {
            if let Ok(player_controlled_by) = camera_query.get(player_entity) {
                if controlled_by.owner == player_controlled_by.owner {
                    let color = Color::srgb(1.0, 1.0, 0.0);
                    commands.entity(camera_entity).insert((
                        Mesh3d(meshes.add(Sphere::new(0.5).mesh().ico(5).unwrap())),
                        MeshMaterial3d(materials.add(color)),
                        PlayerHandle(player_entity),
                        ControlledBy {
                            owner: controlled_by.owner,
                            lifetime: Lifetime::SessionBased,
                        },
                    ));
                    break;
                }
            }
        }
    }
}

fn main() {
    let tick_duration = Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ);
    let mut app = App::new();
    app.add_plugins(
            DefaultPlugins
                .build()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Cube Fight Server".to_string(), // ToDo
                        // Bind to canvas included in `index.html`
                        canvas: Some("#bevy".to_owned()),
                        fit_canvas_to_parent: true,
                        // Tells wasm not to override default event handling, like F5 and Ctrl+R
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins(ServerPlugins { tick_duration })
        .add_plugins(SharedPlugin);
        spawn_server(&mut app);

        app.add_plugins(ServerPlugin)
        .run();
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServerTransports {
    WebTransport {
        local_port: u16,
        certificate: WebTransportCertificateSettings,
    },
    #[cfg(feature = "udp")]
    Udp { local_port: u16 },
    #[cfg(feature = "websocket")]
    WebSocket { local_port: u16 },
    #[cfg(feature = "steam")]
    Steam { local_port: u16 },
}

#[derive(Component, Debug)]
#[component(on_add = CubeServer::on_add)]
pub struct CubeServer {
    /// Possibly add a conditioner to simulate network conditions
    pub conditioner: Option<RecvLinkConditioner>,
    /// Which transport to use
    pub transport: ServerTransports,
    pub shared: SharedSettings,
}

impl CubeServer {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        world.commands().queue(move |world: &mut World| -> Result {
            let mut entity_mut = world.entity_mut(entity);
            let settings = entity_mut.take::<CubeServer>().unwrap();
            entity_mut.insert((Name::from("Server"),));

            let add_netcode = |entity_mut: &mut EntityWorldMut| {
                // Use private key from environment variable, if set. Otherwise from settings file.
                let private_key = if let Some(key) = parse_private_key_from_env() {
                    info!("Using private key from LIGHTYEAR_PRIVATE_KEY env var");
                    key
                } else {
                    settings.shared.private_key
                };
                // TODO: Reserch alternate Server types without netcode feature
                entity_mut.insert(NetcodeServer::new(NetcodeConfig {
                    protocol_id: settings.shared.protocol_id,
                    private_key,
                    ..Default::default()
                }));
            };

            match settings.transport {
                #[cfg(feature = "udp")]
                ServerTransports::Udp { local_port } => {
                    add_netcode(&mut entity_mut);
                    let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), local_port);
                    entity_mut.insert((LocalAddr(server_addr), ServerUdpIo::default()));
                }
                ServerTransports::WebTransport {
                    local_port,
                    certificate,
                } => {
                    add_netcode(&mut entity_mut);
                    let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), local_port);
                    // let server_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), local_port);
                    entity_mut.insert((
                        LocalAddr(server_addr),
                        WebTransportServerIo {
                            certificate: (&certificate).into(),
                            // certificate: certificate.into(),
                        },
                    ));
                }
            };
            Ok(())
        });
    }
}
// fn spawn_server(app: &mut App) {
fn spawn_server(app: &mut App) -> Entity {
    // let conditioner = LinkConditionerConfig::average_condition();
    let server = app
        .world_mut()
        .spawn(CubeServer {
            conditioner: None,
            // transport: ServerTransports::Udp {
            //     local_port: SERVER_PORT,
            // },
            transport: ServerTransports::WebTransport {
                local_port: SERVER_PORT,
                certificate: WebTransportCertificateSettings::FromFile {
                    cert: "certificates/cert.pem".to_string(),
                    key: "certificates/key.pem".to_string(),
                },
            },
            shared: SHARED_SETTINGS,
        })
        .id();
    app.add_systems(Startup, start);
    server

}

fn start(mut commands: Commands, server: Single<Entity, With<Server>>) {
    commands.trigger_targets(Start, server.into_inner());
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WebTransportCertificateSettings {
    /// Generate a self-signed certificate, with given SANs list to add to the certifictate
    /// eg: ["example.com", "*.gameserver.example.org", "10.1.2.3", "::1"]
    AutoSelfSigned(Vec<String>),
    /// Load certificate pem files from disk
    FromFile {
        /// Path to cert .pem file
        cert: String,
        /// Path to private key .pem file
        key: String,
    },
}

// TODO: Look over default
impl Default for WebTransportCertificateSettings {
    fn default() -> Self {
        let sans = vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "::1".to_string(),
        ];
        WebTransportCertificateSettings::AutoSelfSigned(sans)
    }
}

// TODO: Lookover Identity code
impl From<&WebTransportCertificateSettings> for Identity {
    fn from(wt: &WebTransportCertificateSettings) -> Identity {
        match wt {
            WebTransportCertificateSettings::AutoSelfSigned(sans) => {
                // In addition to and Subject Alternate Names (SAN) added via the config,
                // we add the public ip and domain for edgegap, if detected, and also
                // any extra values specified via the SELF_SIGNED_SANS environment variable.
                let mut sans = sans.clone();
                // Are we running on edgegap?
                if let Ok(public_ip) = std::env::var("ARBITRIUM_PUBLIC_IP") {
                    println!("🔐 SAN += ARBITRIUM_PUBLIC_IP: {public_ip}");
                    sans.push(public_ip);
                    sans.push("*.pr.edgegap.net".to_string());
                }
                // generic env to add domains and ips to SAN list:
                // SELF_SIGNED_SANS="example.org,example.com,127.1.1.1"
                if let Ok(san) = std::env::var("SELF_SIGNED_SANS") {
                    println!("🔐 SAN += SELF_SIGNED_SANS: {san}");
                    sans.extend(san.split(',').map(|s| s.to_string()));
                }
                println!("🔐 Generating self-signed certificate with SANs: {sans:?}");
                let identity = Identity::self_signed(sans).unwrap();
                let digest = identity.certificate_chain().as_slice()[0].hash();
                println!("🔐 Certificate digest: {digest}");
                identity
            }
            WebTransportCertificateSettings::FromFile {
                cert: cert_pem_path,
                key: private_key_pem_path,
            } => {
                println!(
                    "Reading certificate PEM files:\n * cert: {cert_pem_path}\n * key: {private_key_pem_path}",
                );
                // this is async because we need to load the certificate from io
                // we need async_compat because wtransport expects a tokio reactor
                let identity = IoTaskPool::get()
                    .scope(|s| {
                        s.spawn(Compat::new(async {
                            Identity::load_pemfiles(cert_pem_path, private_key_pem_path)
                                .await
                                .unwrap()
                        }));
                    })
                    .pop()
                    .unwrap();
                let digest = identity.certificate_chain().as_slice()[0].hash();
                println!("🔐 Certificate digest: {digest}");
                identity
            }
        }
    }
}


/// Reads and parses the LIGHTYEAR_PRIVATE_KEY environment variable into a private key.
pub fn parse_private_key_from_env() -> Option<[u8; PRIVATE_KEY_BYTES]> {
    let Ok(key_str) = std::env::var("LIGHTYEAR_PRIVATE_KEY") else {
        return None;
    };
    let private_key: Vec<u8> = key_str
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == ',')
        .collect::<String>()
        .split(',')
        .map(|s| {
            s.parse::<u8>()
                .expect("Failed to parse number in private key")
        })
        .collect();

    if private_key.len() != PRIVATE_KEY_BYTES {
        panic!("Private key must contain exactly {PRIVATE_KEY_BYTES} numbers",);
    }

    let mut bytes = [0u8; PRIVATE_KEY_BYTES];
    bytes.copy_from_slice(&private_key);
    Some(bytes)
}


// fn server_camera_movement(
//     mut cameras: Query<(&mut Transform, &mut CameraSettings, &ActionState<Action>, &PlayerHandle), Without<PlayerId>>,
//     players: Query<&Transform, With<PlayerId>>,
// ) {
//     for (mut camera_transform, mut camera_settings, action_state, player_handle) in cameras.iter_mut() {
//         let Ok(player_transform) = players.get(player_handle.0) else {return;};
    
//         shared_camera_movement(camera_transform, camera_settings, &player_transform.translation, action_state);
//     }
// }