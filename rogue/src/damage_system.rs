use specs::prelude::*;
use super::{
    Position, CombatStats, WantsToTakeDamage, Player, Name, GameLog,
    Equipped, ElementalDamageKind, GrantsMeleeDefenseBonus,
    StatusIsImmuneToFire, StatusIsImmuneToChill,
    StatusIsPhysicalDefenseBuffed, InSpellBook, SpawnEntityWhenKilled,
    EntitySpawnRequest, EntitySpawnRequestBuffer
};

pub struct DamageSystem {}

impl DamageSystem {

    pub fn clean_up_the_dead(ecs: &mut World) {
        let mut dead: Vec<Entity> = Vec::new();
        { // Scope to contain the lifetime of the immutable borrow of ecs in
          //the line below.
            let combat_stats = ecs.read_storage::<CombatStats>();
            let players = ecs.read_storage::<Player>();
            let names = ecs.read_storage::<Name>();
            let spawns_when_killed = ecs.read_storage::<SpawnEntityWhenKilled>();
            let positions = ecs.read_storage::<Position>();
            let mut spawn_buffer = ecs.write_resource::<EntitySpawnRequestBuffer>();
            let mut log = ecs.write_resource::<GameLog>();
            let entities = ecs.entities();
            for (entity, stats) in (&entities, &combat_stats).join() {
                if stats.hp <= 0 {
                    // We have different branches if the dead entity is the
                    // player vs. some other entity.
                    let player = players.get(entity);
                    match player {
                        Some(_) => {} // TODO: Actually do something here!
                        None => {
                            let victim = names.get(entity);
                            if let Some(victim) = victim {
                                log.entries.push(format!("{} is dead.", victim.name));
                            }
                            // Grab any entities owned by the dead entity. I.e. spells, etc.
                            let spells = DamageSystem::get_any_owned_entities(ecs, &entity);
                            for spell in spells {
                                dead.push(spell);
                            }
                            // Check if we're supposed to drop anything.
                            if let Some(spawns) = spawns_when_killed.get(entity) {
                                let pos = positions.get(entity)
                                    .expect("Attempting to drop item but entity has no position.");
                                spawn_buffer.request(EntitySpawnRequest {
                                    x: pos.x, y: pos.y, kind: spawns.kind
                                })
                            }
                            dead.push(entity);
                        }
                    }
                }
            }
        }
        for victim in dead {
            ecs.delete_entity(victim).expect("Unable to delete a dead entity.")
        }
    }

    // Find any entities that are owned by a given entity. I.e., they are in the
    // entities backpack or spellbook.
    fn get_any_owned_entities(ecs: &World, entity: &Entity) -> Vec<Entity> {
        let entities = ecs.entities();
        let spellbooks = ecs.read_storage::<InSpellBook>();
        let spells_owned_by_entity: Vec<Entity> = (&entities, &spellbooks).join()
            .filter(|(_e, book)| book.owner == *entity)
            .map(|(e, _book)| e)
            .collect();
        spells_owned_by_entity
    }
}

//----------------------------------------------------------------------------
// Process queued damage.
//
// Actually commit any damage an enetity has sustained this turn. We
// additionally apply any defense buffs, resitances, or invulnrabilities here.
//----------------------------------------------------------------------------
impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, Equipped>,
        WriteStorage<'a, WantsToTakeDamage>,
        ReadStorage<'a, GrantsMeleeDefenseBonus>,
        ReadStorage<'a, StatusIsImmuneToFire>,
        ReadStorage<'a, StatusIsImmuneToChill>,
        ReadStorage<'a, StatusIsPhysicalDefenseBuffed>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut stats,
            equipped,
            mut wants_to_take_damage,
            melee_defense_bonus,
            status_fire_immunity,
            status_chill_immunity,
            is_defense_buffs,
        ) = data;

        for (entity, stats, damage) in (&entities, &mut stats, &wants_to_take_damage).join() {

            let melee_defense_bonus: i32 = (&entities, &melee_defense_bonus, &equipped)
                .join()
                .filter(|(_e, _ab, eq)| eq.owner == entity)
                .map(|(_e, ab, _eq)| ab.bonus)
                .sum();
            let defense_buff_factor: i32 = is_defense_buffs.get(entity)
                .map_or(1, |_b| 2);
            let is_immune_to_fire: bool = status_fire_immunity.get(entity).is_some();
            let is_immune_to_chill: bool = status_chill_immunity.get(entity).is_some();

            for (dmg, kind) in damage.amounts.iter().zip(&damage.kinds) {
                match *kind {
                    ElementalDamageKind::Physical => {
                        stats.take_damage(i32::max(0, (dmg - melee_defense_bonus) / defense_buff_factor));
                    }
                    ElementalDamageKind::Hunger => {
                        stats.take_damage(*dmg);
                    }
                    ElementalDamageKind::Drowning => {
                        stats.take_damage(*dmg);
                    }
                    ElementalDamageKind::Fire => {
                        if !is_immune_to_fire {
                            stats.take_damage(*dmg);
                        }
                    }
                    ElementalDamageKind::Chill => {
                        if !is_immune_to_chill {
                            stats.take_damage(*dmg);
                        }
                    }
                }
            }
        }
        wants_to_take_damage.clear();
    }
}