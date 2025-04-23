mod camera;
mod components;
mod map;
mod map_builder;
mod spawner;
mod systems;
mod turn_state;

// define a prelude module with imported elements
mod prelude {
    pub use bracket_lib::prelude::*;
    pub use legion::systems::CommandBuffer;
    pub use legion::world::SubWorld;
    pub use legion::*;
    pub const SCREEN_WIDTH: i32 = 80;
    pub const SCREEN_HEIGHT: i32 = 50;
    pub const DISPLAY_WIDTH: i32 = SCREEN_WIDTH / 2;
    pub const DISPLAY_HEIGHT: i32 = SCREEN_HEIGHT / 2;
    // use crate::mod pulls a module defined at root scope
    pub use crate::camera::*;
    pub use crate::components::*;
    pub use crate::map::*;
    pub use crate::map_builder::*;
    pub use crate::spawner::*;
    pub use crate::systems::*;
    pub use crate::turn_state::*;
}

use std::collections::HashSet;
use prelude::*;

struct State {
    ecs: World,
    resources: Resources,
    input_systems: Schedule,
    player_systems: Schedule,
    monster_systems: Schedule,
}

impl State {
    fn new() -> Self {
        let mut state = State {
            ecs: World::default(),
            resources: Resources::default(),
            input_systems: build_input_scheduler(),
            player_systems: build_player_scheduler(),
            monster_systems: build_monster_scheduler(),
        };
        state.setup();
        state
    }

    fn game_over(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(2);
        ctx.print_color_centered(2, RED, BLACK, "Your quest has ended.");
        ctx.print_color_centered(
            4,
            WHITE,
            BLACK,
            "Slain by a monster, your hero's journey has come to an end.",
        );
        ctx.print_color_centered(
            5,
            WHITE,
            BLACK,
            "The amulet of Yala remains unclaimed, and your hometown is not saved.",
        );
        ctx.print_color_centered(8, YELLOW, BLACK, "Try again with a new hero?");
        ctx.print_color_centered(9, GREEN, BLACK, "Press 1 to play again.");

        if let Some(VirtualKeyCode::Key1) = ctx.key {
            self.reset();
            self.setup();
        }
    }

    fn victory(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(2);
        ctx.print_color_centered(2, GREEN, BLACK, "You have won!");
        ctx.print_color_centered(
            4,
            WHITE,
            BLACK,
            "You put the amulet on and feel it's power course through your veins...",
        );
        ctx.print_color_centered(
            5,
            WHITE,
            BLACK,
            "Your town is saved, and you can return to your normal life.",
        );
        ctx.print_color_centered(7, GREEN, BLACK, "Press 1 to play again.");

        if let Some(VirtualKeyCode::Key1) = ctx.key {
            self.reset();
            self.setup();
        }
    }

    fn advance_level(&mut self) {
        let player_entity = *<Entity>::query()
            .filter(component::<Player>())
            .iter(&mut self.ecs)
            .next()
            .unwrap();

        let mut entities_to_keep = HashSet::new();
        entities_to_keep.insert(player_entity);

        <(Entity, &Carried)>::query()
            .iter(&mut self.ecs)
            .filter(|(_e, carry)| carry.0 == player_entity)
            .map(|(e, _carry)| *e)
            .for_each(|e| {
                entities_to_keep.insert(e);
            });

        // remove all other entities
        let mut cb = CommandBuffer::new(&mut self.ecs);
        for e in Entity::query().iter(&self.ecs) {
            if !entities_to_keep.contains(e) {
                cb.remove(*e);
            }
        }
        // flush the command buffer to the world to apply the changes
        cb.flush(&mut self.ecs);

        // trigger a re-render of the game
        <&mut FieldOfView>::query()
            .iter_mut(&mut self.ecs)
            .for_each(|fov| fov.is_dirty = true);

        // create a new map
        let mut rng = RandomNumberGenerator::new();
        let mut map_builder = MapBuilder::new(&mut rng);

        let mut map_level = 0;
        <(&mut Player, &mut Point)>::query()
            .iter_mut(&mut self.ecs)
            .for_each(|(player, pos)| {
                player.map_level += 2;
                map_level = player.map_level;
                pos.x = map_builder.player_start.x;
                pos.y = map_builder.player_start.y;
            });

        // spawn the amulet or player
        if map_level == 2 {
            spawn_amulet_of_yala(&mut self.ecs, map_builder.amulet_start);
        } else {
            let exit_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
            map_builder.map.tiles[exit_idx] = TileType::Exit;
        }

        map_builder
            .monster_spawns
            .iter()
            .for_each(|pos| spawn_entity(&mut self.ecs, &mut rng, *pos));
        self.add_default_resources(map_builder);
    }

    pub fn reset(&mut self) {
        self.ecs = World::default();
        self.resources = Resources::default();
    }

    fn setup(&mut self) {
        self.ecs = World::default();
        self.resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let mut map_builder = MapBuilder::new(&mut rng);
        spawn_player(&mut self.ecs, map_builder.player_start);
        // spawn_amulet_of_yala(&mut self.ecs, map_builder.amulet_start);
        let exit_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
        map_builder.map.tiles[exit_idx] = TileType::Exit;
        map_builder
            .monster_spawns
            .iter()
            .for_each(|pos| spawn_entity(&mut self.ecs, &mut rng, *pos));
        self.add_default_resources(map_builder);
    }

    fn add_default_resources(&mut self, map_builder: MapBuilder) {
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        // map layer
        ctx.set_active_console(0);
        ctx.cls();
        // entity layer
        ctx.set_active_console(1);
        ctx.cls();
        // hud layer
        ctx.set_active_console(2);
        ctx.cls();

        ctx.set_active_console(0);
        self.resources.insert(ctx.key);
        self.resources.insert(Point::from_tuple(ctx.mouse_pos()));

        // add the keyboard resource to all systems
        let current_state = *self.resources.get::<TurnState>().unwrap();

        match current_state {
            TurnState::AwaitingInput => self
                .input_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::PlayerTurn => self
                .player_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::MonsterTurn => self
                .monster_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::GameOver => self.game_over(ctx),
            TurnState::Victory => self.victory(ctx),
            TurnState::NextLevel => self.advance_level(),
        }

        render_draw_buffer(ctx).expect("Render error");
    }
}

fn main() -> BError {
    const DUNGEON_FONT: &str = "dungeonfont.png";
    const TERMINAL_FONT: &str = "terminal8x8.png";

    let context = BTermBuilder::new()
        .with_title("Dungeon Crawler")
        .with_fps_cap(30.0)
        .with_dimensions(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .with_tile_dimensions(32, 32)
        .with_resource_path("resources/")
        .with_font(DUNGEON_FONT, 32, 32)
        .with_font(TERMINAL_FONT, 8, 8)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, DUNGEON_FONT)
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, DUNGEON_FONT)
        .with_simple_console_no_bg(SCREEN_WIDTH * 2, SCREEN_HEIGHT * 2, TERMINAL_FONT)
        .build()?;

    main_loop(context, State::new())
}
