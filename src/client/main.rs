use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, ops::DerefMut, time::Duration};

use bevy::{asset::AssetMetaCheck, ecs::{component::HookContext, world::DeferredWorld}, log::LogPlugin, prelude::*};
use bevy_rapier3d::prelude::*;
use lightyear::prelude::*;
use lightyear::prelude::client::*;
use lightyear::webtransport::prelude::client::*;
use leafwing_input_manager::prelude::*;
// use lightyear::client::prelude::*;

use monkey::core::{player::{shared_control_ball, Action, EntityColor, PhysicsBundle, PlayerId, CameraSettings, shared_camera_movement}, shared::{SharedPlugin, SharedSettings, CLIENT_PORT, FIXED_TIMESTEP_HZ, SERVER_ADDR, SHARED_SETTINGS}};
use serde::{Deserialize, Serialize};

// const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 25565);
// const CLIENT_ADDR: SocketAddr = todo!();//"127.0.0.1:25565";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClientTransports {
    WebTransport,
    // #[cfg(not(target_family = "wasm"))]
    // Udp,
    #[cfg(feature = "websocket")]
    WebSocket,
    #[cfg(feature = "steam")]
    Steam,
}




pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, init_client);
        let tick_duration = Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ);

        app.insert_resource(ClearColor(Color::linear_rgb(0.4, 0.4, 0.4)));
        // app.insert_resource(CameraSettings{orbit_distance: 3.0, pitch_speed: 0.3, yaw_speed: 1.0});
        app.add_systems(FixedUpdate, player_movement);
        app.add_systems(Update, client_camera_movement);
        app.add_systems(Startup, spawn_camera);
        // DEBUG
        app.add_systems(PostUpdate, print_overstep);
    }
}

fn spawn_camera(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
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
        camera_input,
        CameraSettings{orbit_distance: 3.0, pitch_speed: 0.3, yaw_speed: 1.0},
        Replicate::to_server(),
        PredictionTarget::to_server(),
    ));
    
    // Light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    // let light = commands.spawn((
    //     PointLight {
    //         intensity: 150.0,
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     Transform::from_xyz(4.0, 8.0, 4.0),
    // )).id();
    // info!("Spawned light: {:?}", light);
    
    // Create the ground
    let ground = commands.spawn((
        Collider::cuboid(20.0, 0.1, 20.0), 
        Transform::from_xyz(0.0, -2.0, 0.0),
        Mesh3d(meshes.add(Cuboid::new(40.0, 0.1, 40.0).mesh())),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    )).id();
    println!("{:?}", ground)

    // let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    // let material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
    // commands.spawn((
    //     mesh,
    //     material,
    //     Transform::from_xyz(0.0, 0.5, 0.0),
    // ));
}

// fn init_client(commands: Commands) {
//     let auth = Authentication::Manual {
//         server_addr: SERVER_ADDR,
//         client_id: 0,
//         private_key: Key::default(),
//         protocol_id: 0,
//     };
//     let client = commands
//         .spawn((
//             Client::default(),
//             LocalAddr(CLIENT_ADDR),
//             PeerAddr(SERVER_ADDR),
//             Link::new(None),
//             ReplicationReceiver::default(),
//             NetcodeClient::new(auth, NetcodeConfig::default())?,
//             UdpIo::default(),
//         ))
//         .id();
//     commands.trigger_targets(Connect, client);
    
// }

fn main() {
    let tick_duration = Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ);
    let mut app = App::new();
    app
        .add_plugins(
            DefaultPlugins
                .build()
                .set(LogPlugin {
                    level: bevy::log::Level::TRACE,
                    // level: bevy::log::Level::TRACE,
                    // filter: "warn,monkey=trace".into(),
                    filter: "info,monkey=trace".into(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Cube Fight Client".to_string(), // ToDo
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
                })
        ).
        add_plugins((
            ClientPlugins { tick_duration },
            // #[cfg(feature = "gui")]
            // ExampleClientRendererPlugin::new(format!("Client {client_id:?}")),
        ))
        .add_plugins(SharedPlugin);
        spawn_client(&mut app);
        app.add_plugins(ClientPlugin)
        // .add_observer(finish_spawned_player)
        .add_observer(handle_interpolated_spawn)
        .add_observer(handle_predicted_spawn)
        .run();
}


// The client input only gets applied to predicted entities that we own
// This works because we only predict the user's controlled entity.
// If we were predicting more entities, we would have to only apply movement to the player owned one.
fn player_movement(
    timeline: Single<&LocalTimeline, With<Client>>,
    mut query: Query<
        (
            &ActionState<Action>,
            &PlayerId,
            &mut Velocity,
            &mut Transform,
            Entity,
        ), 
        With<Predicted>
    >,
    camera: Single<&Transform, (With<Camera3d>, Without<PlayerId>)>,
    rapier_context: ReadRapierContext,
) {

    // let tick = timeline.tick();
    
    if query.is_empty() {
        // trace!("No predicted players found with ActionState");
        info!("No predicted players found with ActionState");
    }
    
    for (action_state, player_id, velocity, mut transform, entity) in query.iter_mut() {
        let has_input = !action_state.get_pressed().is_empty() || 
                       action_state.axis_pair(&Action::Move) != Vec2::ZERO;
        
        if has_input {
            let tick = timeline.tick();
            // trace!(?entity, ?tick, ?transform, actions = ?action_state.get_pressed(), "applying movement to predicted player");
            info!(?entity, ?tick, ?transform, actions = ?action_state.get_pressed(), "applying movement to predicted player");
            // note that we also apply the input to the other predicted clients! even though
            //  their inputs are only replicated with a delay!
            // TODO: add input delay?
            shared_control_ball(action_state, velocity, transform.deref_mut(), entity, camera.forward(), &rapier_context);
        }
    }

    // for (position, inputs) in position_query.iter_mut() {
    //     if let Some(inputs) = &inputs.value {
    //         shared::shared_movement_behaviour(position, inputs);
    //     }
    // }
}

// Made to allow lerp cam
// Could we have camera be a child of the player and still lerp?
fn client_camera_movement(
    camera: Single<(&mut Transform, &mut CameraSettings), With<Camera3d>>,
    player: Single<&Transform, (With<Predicted>, With<PlayerId>, Without<Camera3d>)>,
    // mut camera_settings: ResMut<CameraSettings>,
    opt_action_state: Option<Single<&ActionState<Action>, With<Camera3d>>>,
) {
    // camera_transform.translation = player.translation + Vec3::new(0.0, 5.0, 10.0);
    // Transform::
    let (mut camera_transform, mut camera_settings) = camera.into_inner();
    let Some(action_state) = opt_action_state else {return;};

    shared_camera_movement(camera_transform.deref_mut(), camera_settings.deref_mut(), &player.translation, action_state.into_inner());
    
    
    
}

// Observer that spawns visual components when PlayerId is added
// fn finish_spawned_player_BAD(
//     trigger: Trigger<OnAdd, PlayerId>,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     query: Query<&EntityColor>,
//     predicted_query: Query<(), With<Predicted>>,
// ) {
//     let entity = trigger.target();
//     let Ok(color) = query.get(entity) else { return; };
    
//     let mut entity_commands = commands.entity(entity);
//     entity_commands.insert((
//         Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0).mesh())),
//         MeshMaterial3d(materials.add(**color)),
//         PhysicsBundle::player(),
//     ));
    
//     // Only add input_map to predicted (client-controlled) entities
//     if predicted_query.get(entity).is_ok() {
//         let input_map = InputMap::new([
//             (Action::Jump, KeyCode::Space),
//             (Action::Reset, KeyCode::KeyR),
//         ]).with_dual_axis(
//             Action::Move,
//             VirtualDPad::wasd().with_circle_deadzone(0.1)
//         );
//         entity_commands.insert(input_map);
//     }
// }

// When the predicted copy of the client-owned entity is spawned, do stuff
// - assign it a different saturation
// - add physics components so that its movement can be predicted
pub(crate) fn handle_predicted_spawn(
    trigger: Trigger<OnAdd, (PlayerId, Predicted)>,
    mut commands: Commands,
    mut player_query: Query<(&mut EntityColor, Has<Controlled>), With<Predicted>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Ok((mut color, controlled)) = player_query.get_mut(trigger.target()) {
        let hsva = Hsva {
            saturation: 0.4,
            ..Hsva::from(color.0)
        };
        color.0 = Color::from(hsva);
        let mut entity_mut = commands.entity(trigger.target());
        entity_mut.insert((
            PhysicsBundle::player(),
            Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0).mesh())),
            MeshMaterial3d(materials.add(**color)),
        ));
        if controlled {
            let input_map = InputMap::new([
                (Action::Jump, KeyCode::Space),
                (Action::Reset, KeyCode::KeyR),
            ]).with_dual_axis(
                Action::Move,
                VirtualDPad::wasd().with_circle_deadzone(0.1)
            );
            entity_mut.insert(input_map);
        }
    }
}


// When the interpolated copy of the client-owned entity is spawned, do stuff
// - assign it a different color
pub(crate) fn handle_interpolated_spawn(
    trigger: Trigger<OnAdd, EntityColor>,
    mut interpolated: Query<&mut EntityColor, Added<Interpolated>>,
) {
    if let Ok(mut color) = interpolated.get_mut(trigger.target()) {
        let hsva = Hsva {
            saturation: 0.1,
            ..Hsva::from(color.0)
        };
        color.0 = Color::from(hsva);
    }
}

// Debug system to check on the oversteps
fn print_overstep(time: Res<Time<Fixed>>, timeline: Single<&InputTimeline, With<Client>>) {
    let input_overstep = timeline.overstep();
    let input_overstep_ms = input_overstep.value() * (time.timestep().as_millis() as f32);
    let time_overstep = time.overstep();
    info!(?input_overstep_ms, ?time_overstep, "overstep");
}

/// Event that examples can trigger to spawn a client.
#[derive(Component, Clone, Debug)]
#[component(on_add = CubeClient::on_add)]
pub struct CubeClient {
    pub client_id: u64,
    /// The client port to listen on
    pub client_port: u16,
    /// The socket address of the server
    pub server_addr: SocketAddr,
    /// Possibly add a conditioner to simulate network conditions
    pub conditioner: Option<RecvLinkConditioner>,
    /// Which transport to use
    pub transport: ClientTransports,
    pub shared: SharedSettings,
}

impl CubeClient {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;

        world.commands().queue(move |world: &mut World| -> Result {
            let mut entity_mut = world.entity_mut(entity);
            let settings = entity_mut.take::<CubeClient>().unwrap();
            let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), settings.client_port);
            entity_mut.insert((
                Client::default(),
                Link::new(settings.conditioner.clone()),
                LocalAddr(client_addr),
                PeerAddr(settings.server_addr),
                ReplicationReceiver::default(),
                // ReplicationSender::default(),
                PredictionManager::default(),
                InterpolationManager::default(),
                Name::from("Client"),
            ));

            
            let add_netcode = |entity_mut: &mut EntityWorldMut| -> Result {
                // use dummy zeroed key explicitly here.
                let auth = Authentication::Manual {
                    server_addr: settings.server_addr,
                    client_id: settings.client_id,
                    private_key: settings.shared.private_key,
                    protocol_id: settings.shared.protocol_id,
                };
                let netcode_config = NetcodeConfig {
                    // Make sure that the server times out clients when their connection is closed
                    client_timeout_secs: 3,
                    token_expire_secs: -1,
                    ..default()
                };
                entity_mut.insert(NetcodeClient::new(auth, netcode_config)?);
                Ok(())
            };
            match settings.transport {
                // #[cfg(not(target_family = "wasm"))]
                // ClientTransports::Udp => {
                //     add_netcode(&mut entity_mut)?;
                //     entity_mut.insert(UdpIo::default());
                // }
                ClientTransports::WebTransport => {
                    add_netcode(&mut entity_mut)?;
                    let certificate_digest = {
                        #[cfg(target_family = "wasm")]
                        {
                            include_str!("../../certificates/digest.txt").to_string()
                        }
                        #[cfg(not(target_family = "wasm"))]
                        {
                            "".to_string()
                        }
                    };
                    entity_mut.insert(WebTransportClientIo { certificate_digest });
                }
                #[cfg(feature = "steam")]
                ClientTransports::Steam => {
                    entity_mut.insert(SteamClientIo {
                        target: ConnectTarget::Addr(settings.server_addr),
                        config: Default::default(),
                    });
                }
            };
            Ok(())
        });
    }
}

fn spawn_client(app: &mut App) -> Entity {
    // TODO: Allow for changing client id!
    let client_id = 1;
    // let conditioner = LinkConditionerConfig::average_condition();

    let client = app
        .world_mut()
        .spawn(CubeClient {
            client_id: client_id, //client_id.expect("You need to specify a client_id via `-c ID`"),
            client_port: CLIENT_PORT,
            server_addr: SERVER_ADDR,
            // conditioner: Some(RecvLinkConditioner::new(conditioner.clone())),
            conditioner: None,
            // transport: ClientTransports::Udp,
            transport: ClientTransports::WebTransport,
            // #[cfg(feature = "steam")]
            // transport: ClientTransports::Steam,
            shared: SHARED_SETTINGS,
        })
        .id();
    app.add_systems(Startup, connect);
    client
}

fn connect(mut commands: Commands, client: Single<Entity, With<Client>>) {
    commands.trigger_targets(Connect, client.into_inner());
}