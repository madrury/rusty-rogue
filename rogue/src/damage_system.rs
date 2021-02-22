use specs::prelude::*;
use super::{CombatStats, SufferDamage, Player};

pub struct DamageSystem {}

impl DamageSystem {

    pub fn clean_up_the_dead(ecs: &mut World) {
        let mut dead: Vec<Entity> = Vec::new();
        { // Scope to contain the lifetime of the immutable borrow of ecs in
          //the line below.
            let combat_stats = ecs.read_storage::<CombatStats>();
            let players = ecs.read_storage::<Player>();
            let entities = ecs.entities();
            for (entity, stats) in (&entities, &combat_stats).join() {
                if stats.hp <= 0 {
                    // We have different branches if the dead entity is the
                    // player vs. some other entity.
                    let player = players.get(entity);
                    match player {
                        Some(_) => println!("You are dead."),
                        None => dead.push(entity)
                    }
                }
            }
        }
        for victim in dead {
            ecs.delete_entity(victim).expect("Unable to delete a dead entity.")
        }
    }

}

// Process queued damage.
impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;
        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amounts.iter().sum::<i32>();
        }
        damage.clear();
    }

}