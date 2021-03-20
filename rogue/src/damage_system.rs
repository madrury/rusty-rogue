use specs::prelude::*;
use super::{
    CombatStats, WantsToTakeDamage, Player, Name, GameLog, Equipped,
    ElementalDamageKind, GrantsMeleeDefenseBonus, StatusIsImmuneToFire,
    StatusIsImmuneToChill, InSpellBook
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
            let mut log = ecs.write_resource::<GameLog>();
            let entities = ecs.entities();
            for (entity, stats) in (&entities, &combat_stats).join() {
                if stats.hp <= 0 {
                    // We have different branches if the dead entity is the
                    // player vs. some other entity.
                    let player = players.get(entity);
                    match player {
                        Some(_) => {}//println!("You are dead."),
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


// Process queued damage.
impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, Equipped>,
        WriteStorage<'a, WantsToTakeDamage>,
        ReadStorage<'a, GrantsMeleeDefenseBonus>,
        ReadStorage<'a, StatusIsImmuneToFire>,
        ReadStorage<'a, StatusIsImmuneToChill>,
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
        ) = data;

        for (entity, stats, damage) in (&entities, &mut stats, &wants_to_take_damage).join() {

            let defense_bonus: i32 = (&entities, &melee_defense_bonus, &equipped)
                .join()
                .filter(|(_e, _ab, eq)| eq.owner == entity)
                .map(|(_e, ab, _eq)| ab.bonus)
                .sum();
            let is_immune_to_fire: bool = status_fire_immunity.get(entity).is_some();
            let is_immune_to_chill: bool = status_chill_immunity.get(entity).is_some();

            for (dmg, kind) in damage.amounts.iter().zip(&damage.kinds) {
                match *kind {
                    ElementalDamageKind::Physical => {
                        stats.take_damage(i32::max(0, dmg - defense_bonus));
                    }
                    ElementalDamageKind::Hunger => {
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