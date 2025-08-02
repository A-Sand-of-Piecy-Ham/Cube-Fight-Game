use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use lightyear::prelude::input::leafwing;
// use lightyear::prelude::Plugin
use serde::{Deserialize, Serialize};
use lightyear::{input::config::InputConfig, prelude::*};
use lightyear::prelude::client::*;

use crate::core::player::{Action, CameraSettings, EntityColor, PlayerHandle, PlayerId, PlayerInput};

#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq)]
struct PlayerData{
    // player_id: ClientId,
    color: Color,

}


// pub struct ProtocolPlugin;

// impl Plugin for ProtocolPlugin{
//     fn build(&self, app: &mut App) {
//         app.register_component::<PlayerData>();
        
//         // app.add_plugins(InputPlugin::<Inputs>::default());

//         app.add_channel::<Channel1>(ChannelSettings {
//           mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
//           ..default()
//         });
//         // app.register_component::<PlayerPosition>();

//         // app.register_component::<PlayerColor>();
//     }
// }



// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
// pub struct Message1(pub usize);




// All inputs need to implement the `MapEntities` trait
// impl MapEntities for Inputs {
//     fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {}
// }




// pub struct Channel1;

// impl Ease for Transform {
//     fn interpolating_curve_unbounded(start: Self, end: Self) -> impl Curve<Self> {
//         FunctionCurve::new(Interval::UNIT, move |t| {
//             Position(Vec2::lerp(start.0, end.0, t))
//         })
//     }
// }

// impl Diffable for Transform {
//     fn diff(&self, other: &Self) -> Option<Self> {
//         if self.0 == other.0 {
//             None
//         } else {
//             Some(*self)
//         }
//     }
// }

// Protocol
#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // inputs

        app.add_plugins(
            // ServerInputPlugin::<PlayerInput> {
            leafwing::InputPlugin::<Action> {
                config: InputConfig {
                    rebroadcast_inputs: true,
                    ..default()
                },
        });
        // app.add_plugins(NetworkDebug);
        
        // components
        app.register_component::<PlayerId>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        app.register_component::<EntityColor>()
            .add_prediction(PredictionMode::Once)
            .add_interpolation(InterpolationMode::Once);

        // TODO: Exchage for case specific version?
        app.register_component::<Transform>()
            .add_prediction(PredictionMode::Full)
            .add_should_rollback(transform_should_rollback)
            .add_interpolation(InterpolationMode::Full);
            // .add_linear_interpolation_fn()
            // .add_linear_correction_fn();


        // NOTE: interpolation/correction is only needed for components that are visually displayed!
        // we still need prediction to be able to correctly predict the physics on the client
        app.register_component::<Velocity>()
            .add_prediction(PredictionMode::Full);

        app.register_component::<CameraSettings>()
            .add_prediction(PredictionMode::Full)
            .add_interpolation(InterpolationMode::Full);

        app.add_observer(handle_color_change);

        // app.register_component::<RigidBody>();
        // app.register_component::<Collider>();
        // app.register_component::<Restitution>();

        // app.register_component::<AngularVelocity>()
        //     .add_prediction(PredictionMode::Full);
    }
}


pub(crate) fn handle_color_change(
    trigger: Trigger<OnInsert, EntityColor>,
    color: Query<(&EntityColor, &mut MeshMaterial3d<StandardMaterial>), With<PlayerId>>,
    mut material_color: ResMut<Assets<StandardMaterial>>
) {
    let Ok(color) = color.get(trigger.target()) else { return; };
    let mesh = color.1;
    let Some(mut material) = material_color.get_mut(mesh) else {
        panic!("Could not find material for entity!");
        // return;
    };
    material.base_color = **color.0;
    // material_color.get_mut(trigger.target()).unwrap().base_color = color.0; 
}

// TODO: Finetune these rollback constants
fn transform_should_rollback(this: &Transform, that: &Transform) -> bool {
    (this.translation.distance(that.translation) >= 0.01) 
    ||
    (this.rotation.angle_between(that.rotation) >= 0.01)
     
}

// fn rotation_should_rollback(this: &Transform, that: &Transform) -> bool {
//     this.angle_between(*that) >= 0.01
// }