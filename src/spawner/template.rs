use std::cmp::PartialEq;
use crate::prelude::*;
use legion::systems::CommandBuffer;
use ron::de::from_reader;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs::File;

#[derive(Clone, Deserialize, Debug)]
pub struct Template {
    pub entity_type: EntityType,
    pub levels: HashSet<usize>,
    pub frequency: i32,
    pub name: String,
    pub glyph: char,
    pub provides: Option<Vec<(String, i32)>>,
    pub hp: Option<i32>,
    pub base_damage: Option<i32>,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub enum EntityType {
    Item,
    Enemy
}

#[derive(Clone, Deserialize, Debug)]
pub struct Templates {
    pub entities: Vec<Template>
}

impl Templates {
    pub fn load() -> Self {
        let file = File::open("resources/template.ron")
            .expect("Failed to open template file");
        from_reader(file).expect("Failed to parse template file")
    }

    pub fn spawn_entities(
        &self,
        ecs: &mut World,
        rng: &mut RandomNumberGenerator,
        level: usize,
        spawn_points: &[Point]
    ) {
        let mut available_entities = Vec::new();
        self.entities
            .iter()
            .filter(|e| e.levels.contains(&level))
            .for_each(|t| {
                // push n references to this entity, do not clone
                for _ in 0..t.frequency {
                    available_entities.push(t);
                }
            });

        let mut commands = CommandBuffer::new(ecs);
        spawn_points.iter().for_each(|pt| {
            if let Some(entity) = rng.random_slice_entry(&mut available_entities) {
                self.spawn_entity(pt, entity, &mut commands);
            }
        });
        commands.flush(ecs);
    }

    fn spawn_entity(
        &self,
        pt: &Point,
        template: &Template,
        commands: &mut CommandBuffer
    ) {
        let entity = commands.push(
            (
                pt.clone(),
                Render {
                    colour: ColorPair::new(WHITE, BLACK),
                    glyph: to_cp437(template.glyph),
                },
                Name(template.name.clone()),
            )
        );

        match template.entity_type {
            EntityType::Item => {
                commands.add_component(entity, Item{});
            }
            EntityType::Enemy => {
                commands.add_component(entity, Enemy{});
                commands.add_component(entity, FieldOfView::new(6));
                commands.add_component(entity, ChasingPlayer{});
                commands.add_component(entity, Health{
                    current: template.hp.unwrap(),
                    max: template.hp.unwrap()
                });
            }
        }

        if let Some(effects) = &template.provides {
            effects.iter().for_each(|(provides, n)| {
                match provides.as_str() {
                    "Healing" => commands.add_component(entity, ProvidesHealing { amount: *n }),
                    "MagicMap" => commands.add_component(entity, ProvidesDungeonMap{}),
                    _ => println!("Warning: unknown effect: {}", provides)
                }
            })
        }
        
        if let Some(damage) = &template.base_damage {
            println!("Spawning {} with base damage {}", template.name, damage);
            commands.add_component(entity, Damage(*damage));
            if template.entity_type == EntityType::Item {
                commands.add_component(entity, Weapon{})
            }
        } else {
            println!("Spawning {} without damage", template.name);       
        }
    }
}