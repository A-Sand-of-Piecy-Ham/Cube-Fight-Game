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
use serde::{Deserialize, Serialize};

use lightyear::prelude::*;

use monkey::core::player::*;

// mod client;
// mod server;
// use client::*;
// use server::*;

#[derive(Component)]
struct Ground;


#[derive(Resource, Debug)]
struct CameraSettings {
    orbit_distance: f32,
    pitch_speed: f32,
    yaw_speed: f32,
}

// #[derive(States, Debug, Hash, Eq, PartialEq, Clone, Copy)]
// pub enum ActiveInput {
//     MouseAndKeyboard,
//     Gamepad
// }

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::linear_rgb(0.4, 0.4, 0.4)))
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
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(InputManagerPlugin::<Action>::default())
        // .add_systems(Startup, spawn_player)
        // .add_systems(Startup, setup_graphics)
        .add_systems(Startup, setup_level)
        // .add_systems(Startup, grab_mouse_cursor)
        .add_systems(Update, control_ball)
        .add_systems(Update, camera_movement)
        .insert_resource(CameraSettings{orbit_distance: 3.0, pitch_speed: 0.3, yaw_speed: 1.0})
        // .add_systems(Update, check_data)
        // .add_systems(Startup, setup_physics)
        // .add_systems(Update, print_ball_altitude)
        // .add_plugins(GamePlugin)
        // .add_systems(Startup, set_window_icon)
        .run();
}

// fn init_camera() {}

fn setup_level(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    // commands.;.add();
    shared_spawn_player(&mut commands, &mut meshes, &mut materials);

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

    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));


    // Create the ground
    commands.spawn((
        Collider::cuboid(20.0, 0.1, 20.0), 
        Transform::from_xyz(0.0, -2.0, 0.0),
        Mesh3d(meshes.add(Cuboid::new(40.0, 0.1, 40.0).mesh())),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

}




fn check_data(query: Query<&ActionState<Action>, With<AlliedBallTag>>) {
    let action_state = query.single().expect("Player actions not found");
    
    // Check button actions
    if action_state.pressed(&Action::Jump) {
        println!("Jump pressed");
    }
    
    // Check dual-axis actions
    let move_input = action_state.axis_pair(&Action::Move);
    if move_input != Vec2::ZERO {
        println!("Move: {:?}", move_input);
    }
}


fn grab_mouse_cursor(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    window.cursor_options.grab_mode = CursorGrabMode::Confined; // or Locked
    window.cursor_options.visible = false; // Hide the cursor
}

// Made to allow lerp cam
// Could we have camera be a child of the player and still lerp?
fn camera_movement(
    mut camera: Single<&mut Transform, (With<Camera3d>, Without<AlliedBallTag>)>,
    player: Single<&Transform, (With<AlliedBallTag>, Without<Camera3d>)>,
    mut camera_settings: ResMut<CameraSettings>,
    opt_action_state: Option<Single<&ActionState<Action>, With<Camera3d>>>,
) {
    // camera_transform.translation = player.translation + Vec3::new(0.0, 5.0, 10.0);
    // Transform::
    if let Some(action_state) = opt_action_state {
        // for action in action_state.get_pressed(){
        //     match action {
        //         Action::LookAround => {
        //             let axis_pair = action_state.axis_pair(&Action::LookAround);
        //             // let delta_pitch = axis_pair.y * camera_settings.pitch_speed;
        //             // let delta_yaw = axis_pair.x * camera_settings.yaw_speed;
        //             camera.rotation = Quat::from_rotation_y(axis_pair.x) * camera.rotation;
        //             camera.rotation = Quat::from_rotation_x(axis_pair.y) * camera.rotation;
        //         }
        //         Action::Zoom => {
        //             let zoom = action_state.value(&Action::Zoom);
        //             // camera.translation += camera.forward().as_vec3() * zoom;
        //             camera_settings.orbit_distance += zoom;
        //         }
        //         _ => {/*Do nothing*/}
        //     }
        // }
        {
            let look_input = action_state.axis_pair(&Action::LookAround);
            if look_input != Vec2::ZERO {                    
                let axis_pair = action_state.axis_pair(&Action::LookAround);
                // let delta_pitch = axis_pair.y * camera_settings.pitch_speed;
                // let delta_yaw = axis_pair.x * camera_settings.yaw_speed;
                // Use world y-axis
                camera.rotation = Quat::from_rotation_y(-axis_pair.x * camera_settings.yaw_speed) * camera.rotation;
                // Use camera x axis
                camera.rotation *= Quat::from_rotation_x(axis_pair.y * camera_settings.pitch_speed);
                // camera.rotation *= Quat::from_rotation_y(axis_pair.x);
                // camera.rotation *= Quat::from_rotation_x(axis_pair.y);
            }
            
        }
            let zoom = action_state.value(&Action::Zoom);
            if zoom != 0.0 {
                // camera.translation += camera.forward().as_vec3() * zoom;
                camera_settings.orbit_distance = (camera_settings.orbit_distance - zoom).clamp(5.0, 50.0);
            }
        {
            
        }
    }
    
    camera.translation = player.translation + Vec3::new(0.0, 2.0, 0.0) - camera.forward().as_vec3() * camera_settings.orbit_distance;
    
    camera.looking_at(player.translation, Vec3::Y);

}