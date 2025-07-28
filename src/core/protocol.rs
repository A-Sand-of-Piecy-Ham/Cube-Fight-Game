use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use lightyear::prelude::*;
use lightyear::prelude::client::*;

#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq)]
struct PlayerData{
    // player_id: ClientId,
    color: Color,

}


pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin{
    fn build(&self, app: &mut App) {
        app.register_component::<PlayerData>();
        
        // app.add_plugins(InputPlugin::<Inputs>::default());

        app.add_channel::<Channel1>(ChannelSettings {
          mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
          ..default()
        });
        // app.register_component::<PlayerPosition>();

        // app.register_component::<PlayerColor>();
    }
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Message1(pub usize);




// #[derive(Serialize, Deserialize, Debug, PartialEq, Reflect, Eq, Clone)]
// pub enum Inputs {
//     Direction(Direction),
//     Delete,
//     Spawn,
// }

// All inputs need to implement the `MapEntities` trait
// impl MapEntities for Inputs {
//     fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {}
// }




pub struct Channel1;
