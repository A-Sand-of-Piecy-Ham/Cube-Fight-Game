use std::f32::consts::PI;
use std::ops::DerefMut;

use bevy::input::keyboard::KeyboardInput;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::na::Rotation;
use leafwing_input_manager::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::prelude::Ball};
use bevy::asset::AssetMetaCheck;
use monkey::core::protocol::ProtocolPlugin;
use monkey::core::shared::SharedPlugin;
use serde::{Deserialize, Serialize};

use lightyear::prelude::*;

use monkey::core::player::*;

// mod client;
// mod server;
// use client::*;
// use server::*;

#[derive(Component)]
struct Ground;



// #[derive(States, Debug, Hash, Eq, PartialEq, Clone, Copy)]
// pub enum ActiveInput {
//     MouseAndKeyboard,
//     Gamepad
// }

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::linear_rgb(0.0, 0.0, 0.0)))
        .add_plugins(SharedPlugin)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy game".to_string(), // ToDo
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
        // .add_systems(Startup, spawn_player)
        // .add_systems(Startup, setup_graphics)
        // .add_systems(Startup, setup_level)
        // .add_systems(Startup, grab_mouse_cursor)
        // .add_systems(Update, control_ball)
        // .add_systems(Update, check_data)
        // .add_systems(Startup, setup_physics)
        // .add_systems(Update, print_ball_altitude)
        // .add_plugins(GamePlugin)
        // .add_systems(Startup, set_window_icon)
        .run();
}

// fn init_camera() {}





// fn check_data(query: Query<&ActionState<Action>, With<AlliedBallTag>>) {
//     let action_state = query.single().expect("Player actions not found");
    
//     // Check button actions
//     if action_state.pressed(&Action::Jump) {
//         println!("Jump pressed");
//     }
    
//     // Check dual-axis actions
//     let move_input = action_state.axis_pair(&Action::Move);
//     if move_input != Vec2::ZERO {
//         println!("Move: {:?}", move_input);
//     }
// }


fn grab_mouse_cursor(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    window.cursor_options.grab_mode = CursorGrabMode::Confined; // or Locked
    window.cursor_options.visible = false; // Hide the cursor
}

