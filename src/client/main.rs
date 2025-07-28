use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use lightyear::prelude::*;
use lightyear::prelude::client::*;
use leafwing_input_manager::prelude::*;
// use lightyear::client::prelude::*;


use monkey::core::player::{Action, shared_control_ball, PlayerId};

// const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 25565);
// const CLIENT_ADDR: SocketAddr = todo!();//"127.0.0.1:25565";

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, init_client);
        app.add_systems(FixedUpdate, player_movement);
        // DEBUG
        app.add_systems(PostUpdate, print_overstep);
    }
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

    let tick = timeline.tick();
    
    for (action_state, player_id, velocity, transform, entity) in query.iter_mut() {
        if !action_state.get_pressed().is_empty() {
            trace!(?entity, ?tick, ?transform, actions = ?action_state.get_pressed(), "applying movement to predicted player");
            // note that we also apply the input to the other predicted clients! even though
            //  their inputs are only replicated with a delay!
            // TODO: add input delay?
            shared_control_ball(action_state, velocity, transform, entity, camera.forward(), &rapier_context);
        }
    }

    // for (position, inputs) in position_query.iter_mut() {
    //     if let Some(inputs) = &inputs.value {
    //         shared::shared_movement_behaviour(position, inputs);
    //     }
    // }
}

// Debug system to check on the oversteps
fn print_overstep(time: Res<Time<Fixed>>, timeline: Single<&InputTimeline, With<Client>>) {
    let input_overstep = timeline.overstep();
    let input_overstep_ms = input_overstep.value() * (time.timestep().as_millis() as f32);
    let time_overstep = time.overstep();
    trace!(?input_overstep_ms, ?time_overstep, "overstep");
}