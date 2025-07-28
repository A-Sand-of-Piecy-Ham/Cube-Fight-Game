use std::{f32::consts::TAU, hash::{DefaultHasher, Hash}, ops::Deref};

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use bevy_rapier3d::prelude::*;
use lightyear::prelude::*;
use leafwing_input_manager::plugin::InputManagerSystem;
use bevy_rand::prelude::*;
use rand_core::RngCore;


const MAX_ROTATION_VELOCITY: f32 = 500.0;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum Action {
    // TODO: Implement Q and E rotations
    // TODO: Implement deep surfaces (cube can have traction while still not quite touching hard ground)
    #[actionlike(DualAxis)]
    Move,
    #[actionlike(DualAxis)]
    LookAround,
    #[actionlike(Axis)]
    Zoom,
    #[actionlike(Button)]
    Jump,
    #[actionlike(Button)]
    Reset,
}


#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
#[actionlike(Button)]
pub enum SpecialAction {
    Primary,
    Secondary,
    Brace,
    Special,
}

pub enum Levels {
    PlainGrass,
    BallPit,
    Castle,
    MudPit,
    IceAndSnow,
    Pilars,
}

pub enum Abilities {
    Brace,
    Fist,
    BidenBlast,
    AlreadyDead,
    GraplingHook,
    // Stick to ground, killing momentum and rotation. Can be broken if hit with enough force 
    NormalBrace,
    // Quickly remove momentum and rotation, levitate in air allowing rotation build up until released
    LeviBrace,
}

// #[derive(Bundle)]
// pub struct PlayerBundle {
//     RigidBody::Dynamic,
//     // Collider::ball(1.0),
//     Collider::cuboid(1.0, 1.0, 1.0),
//     Restitution::coefficient(0.7),
//     Transform::from_xyz(0.0, 4.0, 0.0),
//     // Mesh3d(meshes.add(Sphere::new(1.0).mesh().ico(5).unwrap())),
//     Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0).mesh())),
//     MeshMaterial3d(materials.add(Color::srgb(0.1, 0.2, 0.7))),
//     // KinematicCharacterController {
//     //     ..KinematicCharacterController::default()
//     // },
//     AlliedBallTag,
//     Velocity::default(),
//     input_map,
// }

/// Called by server
pub fn shared_spawn_player(
    client_id: PeerId,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    rng: &mut GlobalEntropy<WyRand>,
) -> Entity {
    // let mut input_map = InputMap::default()
    let mut input_map = InputMap::new([
        (Action::Jump, KeyCode::Space),
        (Action::Reset, KeyCode::KeyR),
    ])
        .with_dual_axis(
            Action::Move,
            VirtualDPad::wasd()
                // You can configure a processing pipeline to handle axis-like user inputs.
                //
                // This step adds a circular deadzone that normalizes input values
                // by clamping their magnitude to a maximum of 1.0,
                // excluding those with a magnitude less than 0.1,
                // and scaling other values linearly in between.
                .with_circle_deadzone(0.1)
                // Followed by appending Y-axis inversion for the next processing step.
                // .inverted_y()
                // Or reset the pipeline, leaving no any processing applied.
                // .reset_processing_pipeline(),
        );
    // commands.spawn(input_map).insert(BallTag);
    
    // Create the bouncing ball
    let color = Color::srgb_u8((rng.next_u32() % 256).try_into().unwrap() , (rng.next_u32() % 256).try_into().unwrap(), (rng.next_u32() % 256).try_into().unwrap());
    let entity = commands
        .spawn((
            RigidBody::Dynamic,
            // Collider::ball(1.0),
            Collider::cuboid(1.0, 1.0, 1.0),
            Restitution::coefficient(0.7),
            Transform::from_xyz(0.0, 4.0, 0.0),
            // Mesh3d(meshes.add(Sphere::new(1.0).mesh().ico(5).unwrap())),
            Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0).mesh())),
            MeshMaterial3d(materials.add(color)),
            // MeshMaterial3d(materials.add(Color::srgb(0.1, 0.2, 0.7))),
            // KinematicCharacterController {
            //     ..KinematicCharacterController::default()
            // },
            PlayerId(client_id),
            Velocity::default(),
            input_map,
    )).id();

    // TODO: for Katamari game
    // commands.spawn((
    //     Transform::from_xyz(0.0, 2.0, 0.0),
    //     Mesh3d(meshes.add(Sphere::new(1.0).mesh().ico(0).unwrap())),
    //     MeshMaterial3d(materials.add(Color::srgb(0.1, 0.2, 0.7)))
    // ));
    entity
}


#[derive(Component)]
pub struct PlayerId(pub PeerId);

// #[derive(Component)]
// struct EnemyBallTag;



pub fn shared_control_ball(
    // query: Single<(&ActionState<Action>, &mut Velocity, &mut Transform, Entity), (With<PlayerId>, Without<Camera3d>)>,
    action_state: &ActionState<Action>,
    mut ball_velocity: Mut<Velocity>,
    mut ball_transform: Mut<Transform>,
    ball_entity: Entity,
   
    // camera_transform: Single<&Transform, With<Camera3d>>,
    camera_dir: Dir3,
    rapier_context: &ReadRapierContext,
) {
    // let (action_state, mut ball_velocity, mut ball_transform,  ball_entity) = query.into_inner(); 
    // println!("controlling!");
    if action_state.axis_pair(&Action::Move) != Vec2::ZERO {

        // Y is forward-back, X is left-right?
        let axis_pair = action_state.axis_pair(&Action::Move);
        // let delta_ball_velocity: Vec3 = Vec3::new(1.0, 1.0, 1.0) * (Quat::from_rotation_x(PI/2.0) * axis_pair.yx().extend(0.0));
        // TODO: Left right movement acts incorrectly when viewing from above | FIXED?
        let camera_yaw = Quat::from_rotation_y(camera_dir.with_y(0.0).normalize().dot(-Vec3::Z));
        // let camera_yaw = Quat::from_rotation_y(camera.rotation.to_euler(EulerRot::YXZ).0);
        let delta_ball_velocity: Vec3 = Vec3::new(1.0, 1.0, 1.0) * (camera_yaw * Quat::from_rotation_x(TAU) * -axis_pair.yx().extend(0.0));
        // Mat3::from_rotation_x(PI/2.0) * axis_pair.extend(0.0);
        // Mat3(
        //     Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)
        // ).into();

        ball_velocity.angvel += delta_ball_velocity;
    }

    if action_state.just_pressed(&Action::Jump) {
        let ray_pos = ball_transform.translation;
        let ray_dir = Vec3::new(0.0, -1.0, 0.0);
        let max_toi = 4.0;
        let solid = true;
        let filter = QueryFilter::new().exclude_sensors().exclude_rigid_body(ball_entity);
        if let Some((_, toi)) = rapier_context.single().unwrap().cast_ray(ray_pos, ray_dir, max_toi, solid, filter) {
            ball_velocity.linvel += Vec3::new(0.0, 1.0, 0.0) * 10.0 * (1.0/toi).powi(2).clamp(0.5, 4.0);
        }
    }
    if action_state.just_pressed(&Action::Reset) {
        ball_transform.translation = Vec3::new(0.0, 1.0, 0.0);
    }
}

#[derive(Component)]
struct EntityColor(pub Color);

impl Deref for EntityColor {
    type Target = Color;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    // client_id: 
    color: EntityColor,
    physics_bundle: PhysicsBundle,
    transform: Transform,
    mesh_3d: Mesh3d,
    mesh_mat: MeshMaterial3d<StandardMaterial>,
    // controller: KinematicCharacterController,
    // tag: AlliedBallTag,
    velocity: Velocity,
    input_map: InputMap<Action>,
    tag: PlayerId,
}

impl PlayerBundle {
    pub fn new(
        client_id: PeerId,
        transform: Transform,
        color: Color, 
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Self {
        
        let mut input_map = InputMap::new([
            (Action::Jump, KeyCode::Space),
            (Action::Reset, KeyCode::KeyR),
        ]).with_dual_axis(
            Action::Move,
            VirtualDPad::wasd()
                // You can configure a processing pipeline to handle axis-like user inputs.
                //
                // This step adds a circular deadzone that normalizes input values
                // by clamping their magnitude to a maximum of 1.0,
                // excluding those with a magnitude less than 0.1,
                // and scaling other values linearly in between.
                .with_circle_deadzone(0.1)
                // Followed by appending Y-axis inversion for the next processing step.
                // .inverted_y()
                // Or reset the pipeline, leaving no any processing applied.
                // .reset_processing_pipeline(),
        );

        Self {
            color: EntityColor(color),
            physics_bundle: PhysicsBundle(
                RigidBody::Dynamic,
                Collider::cuboid(1.0, 1.0, 1.0),
                Restitution::coefficient(0.7),
            ),
            transform: Transform::from_xyz(0.0, 4.0, 0.0),
            // Mesh3d(meshes.add(Sphere::new(1.0).mesh().ico(5).unwrap())),
            mesh_3d: Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0).mesh())),
            mesh_mat: MeshMaterial3d(materials.add(color)),
            // MeshMaterial3d(materials.add(Color::srgb(0.1, 0.2, 0.7))),
            // KinematicCharacterController {
            //     ..KinematicCharacterController::default()
            // },
            // AlliedBallTag,
            velocity: Velocity::default(),
            tag: PlayerId(client_id),
            input_map,
        }
    }
}

#[derive(Bundle)]
struct PhysicsBundle(RigidBody, Collider, Restitution);

// impl PhysicsBundle {
//     pub fn new() -> Self {
//         Self(
//             RigidBody::Dynamic,
//             // Collider::ball(1.0),
//             Collider::cuboid(1.0, 1.0, 1.0),
//             Restitution::coefficient(0.7),
//         )
//     }
// }


/// Generate pseudo-random color from id
pub fn color_from_id(client_id: PeerId) -> Color {
    let h = (((client_id.to_bits().wrapping_mul(30)) % 360) as f32) / 360.0;
    let s = 1.0;
    let l = 0.5;
    Color::hsl(h, s, l)
}
