use crate::prelude::*;

#[system]
#[read_component(Point)]
#[read_component(MovingRandomly)]
#[read_component(Health)]
#[read_component(Player)]
pub fn random_move(ecs: &SubWorld, commands: &mut CommandBuffer) {
    let mut movers = <(Entity, &Point, &MovingRandomly)>::query();
    let mut positions = <(Entity, &Point, &Health)>::query();
    movers.iter(ecs).for_each(|(entity, pos, _)| {
        let mut rng = RandomNumberGenerator::new();
        let destination = match rng.range(0, 4) {
            0 => Point::new(-1, 0),
            1 => Point::new(1, 0),
            2 => Point::new(0, -1),
            _ => Point::new(0, 1),
        } + *pos;

        let mut attacked = false;
        positions
            .iter(ecs)
            .filter(|(_, target_pos, _)| **target_pos == destination)
            .for_each(|(target, _, _)| {
                if ecs.entry_ref(*target).unwrap().get_component::<Player>().is_ok() {
                    // found a player in destination, attack
                    commands.push(((), WantsToAttack {
                        attacker: *entity,
                        target: *target
                    }));
                }
                // set to false anyway to prevent moving if another non player entity is there
                attacked = true
            });

        if !attacked {
            commands.push(((), WantsToMove { entity: *entity, destination }));
        }
    });
}
