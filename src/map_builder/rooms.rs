use crate::prelude::*;
use super::MapArchitect;

pub struct RoomArchitect {}

impl MapArchitect for RoomArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder {
        let mut mb = MapBuilder::default();
        mb.fill(TileType::Wall);
        mb.build_rooms(rng);
        mb.build_corridors(rng);
        mb.player_start = mb.rooms[0].center();
        mb.amulet_start = mb.find_most_distant();
        for room in mb.rooms.iter().skip(1) {
            mb.monster_spawns.push(room.center());
        }
        mb
    }
}
