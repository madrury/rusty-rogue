use specs::prelude::*;
use super::{
    Viewshed, Monster, CanAct, MonsterBasicAI, MonsterAttackSpellcasterAI, Position, Map,
    RoutingMap, WantsToMeleeAttack, WantsToUseTargeted, StatusIsFrozen,
    InSpellBook, Castable, SpellCharges
};
use rltk::{Point, RandomNumberGenerator};

//----------------------------------------------------------------------------
// System for determining if a Monster can take an action this turn.
//----------------------------------------------------------------------------
pub struct MonsterCanActSystem {}

#[derive(SystemData)]
pub struct MonsterCanActSystemData<'a> {
    entities: Entities<'a>,
    monsters: ReadStorage<'a, Monster>,
    status_is_frozen: ReadStorage<'a, StatusIsFrozen>,
    can_acts: WriteStorage<'a, CanAct>
}

impl<'a> System<'a> for MonsterCanActSystem {
    type SystemData = MonsterCanActSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {

        let MonsterCanActSystemData {entities, monsters, status_is_frozen, mut can_acts} = data;

        for (entity, _monster) in (&entities, &monsters).join() {

            // Guard for frozen monsters: they cannot act.
            if let Some(_) = status_is_frozen.get(entity) {
                continue;
            }
            can_acts.insert(entity, CanAct {})
                .expect("Failed to insert CanAct component.");
        }
    }
}


//----------------------------------------------------------------------------
// System for the most basic monster AI.
//
// Monsters with this AI type are simple Melee attaackers. They attempt to chase
// down the player and will Melee attack until someone is dead.
//----------------------------------------------------------------------------
pub struct MonsterBasicAISystem {}

#[derive(SystemData)]
pub struct MonsterBasicAISystemData<'a> {
    entities: Entities<'a>,
    map: WriteExpect<'a, Map>,
    ppos: ReadExpect<'a, Point>,
    player: ReadExpect<'a, Entity>,
    monsters: ReadStorage<'a, Monster>,
    viewsheds: WriteStorage<'a, Viewshed>,
    basic_ais: WriteStorage<'a, MonsterBasicAI>,
    can_acts: WriteStorage<'a, CanAct>,
    positions: WriteStorage<'a, Position>,
    wants_melee_attack: WriteStorage<'a, WantsToMeleeAttack>,
}

impl<'a> System<'a> for MonsterBasicAISystem {

    type SystemData = MonsterBasicAISystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let MonsterBasicAISystemData {
            entities,
            mut map,
            ppos,
            player,
            monsters,
            mut viewsheds,
            mut basic_ais,
            mut can_acts,
            mut positions,
            mut wants_melee_attack,
        } = data;

        let iter = (
            &entities,
            &monsters,
            &mut viewsheds,
            &mut basic_ais,
            &mut positions).join();

        for (entity, _m, mut viewshed, ai, mut pos) in iter {

            // If the entity cannot act, bail out.
            if can_acts.get(entity).is_none() {
                continue
            }

            // Our decision for what to do is conditional on this data.
            let in_viewshed = viewshed.visible_tiles.contains(&*ppos);
            let keep_following = ai.do_keep_following();
            let next_to_player = rltk::DistanceAlg::Pythagoras.distance2d(
                Point::new(pos.x, pos.y),
                *ppos
            ) < 1.5;

            // Monster next to player branch:
            //   If we're already next to player, we enter into melee combat.
            if next_to_player {
                wants_melee_attack
                    .insert(entity, WantsToMeleeAttack {target: *player})
                    .expect("Failed to insert player as melee target.");
            // Monster seeking player branch:
            //   This branch is taken if the monster is currently seeking the
            //   player, i.e., the monster is currently attempting to move towards
            //   the player until they are adjacent.
            } else if in_viewshed || keep_following {
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32,
                    map.xy_idx(ppos.x, ppos.y) as i32,
                    &RoutingMap::from_map(&*map, &ai.routing_options)
                );
                if path.success && path.steps.len() > 1 {
                    let new_x = path.steps[1] as i32 % map.width;
                    let new_y = path.steps[1] as i32 / map.width;
                    move_monster(&mut map, &mut pos, new_x, new_y, &mut viewshed);
                }
                // Update our monster's propensity to keep following the player
                // when they lose visual contact. After a specified amount of
                // time, the monster will switch to idling.
                if in_viewshed {
                    ai.reset_keep_following();
                } else {
                    ai.decrement_keep_following();
                }
            // Monster idling branch.
            //   This branch is taken if the monster can not currently see the
            //   player, and are flagged to wander when the player is out of
            //   visible range.
            } else if !in_viewshed && ai.no_visibility_wander {
                let new_pos = random_adjacent_position(&map, pos);
                move_monster(&mut map, &mut pos, new_pos.0, new_pos.1, &mut viewshed)
            }
            // We're done acting, so we've used up our action for the turn.
            can_acts.remove(entity).expect("Unable to remove CanAct component.");
        }
    }
}


pub struct MonsterAttackSpellcasterAISystem {}

#[derive(SystemData)]
pub struct MonsterAttackSpellcasterAISystemData<'a> {
    entities: Entities<'a>,
    map: WriteExpect<'a, Map>,
    ppos: ReadExpect<'a, Point>,
    player: ReadExpect<'a, Entity>,
    monsters: ReadStorage<'a, Monster>,
    viewsheds: WriteStorage<'a, Viewshed>,
    attack_spellcaster_ais: WriteStorage<'a, MonsterAttackSpellcasterAI>,
    can_acts: WriteStorage<'a, CanAct>,
    positions: WriteStorage<'a, Position>,
    wants_to_target: WriteStorage<'a, WantsToUseTargeted>,
    in_spellbooks: ReadStorage<'a, InSpellBook>,
    castables: ReadStorage<'a, Castable>,
    charges: ReadStorage<'a, SpellCharges>,
}

impl<'a> System<'a> for MonsterAttackSpellcasterAISystem {

    type SystemData = MonsterAttackSpellcasterAISystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let MonsterAttackSpellcasterAISystemData {
            entities,
            mut map,
            ppos,
            player,
            monsters,
            mut viewsheds,
            mut attack_spellcaster_ais,
            mut can_acts,
            mut positions,
            mut wants_to_target,
            in_spellbooks,
            castables,
            charges
        } = data;

        let iter = (
            &entities,
            &monsters,
            &mut viewsheds,
            &mut attack_spellcaster_ais,
            &mut positions).join();

        for (entity, _m, mut viewshed, ai, mut pos) in iter {

            // If the entity cannot act, bail out.
            if can_acts.get(entity).is_none() {
                continue
            }

            // Our decision for what to do is conditional on this data.
            let in_viewshed = viewshed.visible_tiles.contains(&*ppos);
            let l_infinity_distance_to_player = i32::max(
                i32::abs(pos.x - ppos.x),
                i32::abs(pos.y - ppos.y),
            );
            let mut spells = (&entities, &in_spellbooks, &castables, &charges)
                .join()
                .filter(|(_spell, book, _cast, charge)|
                    book.owner == entity && charge.charges > 0
                )
                .map(|(spell, _book, _cast, _charge)| spell);
            let spell_to_cast = spells.next();
            let has_spell_to_cast = spell_to_cast.is_some();

            // Monster seeking player branch:
            //   This branch is taken if the monster is currently seeking the
            //   player, i.e., the monster is currently attempting to move towards
            //   the player until they are adjacent.
            if l_infinity_distance_to_player == ai.distance_to_keep_away && has_spell_to_cast {
                println!("Gonna cast the spell!");
                if let Some(spell) = spell_to_cast {
                    wants_to_target
                        .insert(entity, WantsToUseTargeted {thing: spell, target: *ppos})
                        .expect("Could not insert WantsToUseTargeted from Monster Spellcaster AI.");
                }
            }
            // We're done acting, so we've used up our action for the turn.
            can_acts.remove(entity).expect("Unable to remove CanAct component.");
        }
    }
}

// Move a monster to a new postions.
// **THIS METHOD ASSUMES THE NEW POSITION IS SAFE TO MOVE INTO!**
fn move_monster(map: &mut Map, pos: &mut Position, newposx: i32, newposy: i32, viewshed: &mut Viewshed) {
    let new_idx = map.xy_idx(newposx, newposy);
    let old_idx = map.xy_idx(pos.x, pos.y);
    // We need to update the blocking information *now*, since we do
    // not want later monsters in the move queue to move into the
    // same position as this monster.
    map.blocked[old_idx] = false;
    map.blocked[new_idx] = true;
    pos.x = newposx;
    pos.y = newposy;
    viewshed.dirty = true;
}

// Return a random adjcaent position to pos that is not currently blocked.
// TODO: This should use the general functions we introduced in Map.
fn random_adjacent_position(map: &Map, pos: &Position) -> (i32, i32) {
    // TODO: This should use the game's internal RNG and probably belongs in
    // Map, not here.
    let mut rng = RandomNumberGenerator::new();
    let dx = rng.range(-1, 2);
    let dy = rng.range(-1, 2);
    let idx = map.xy_idx(pos.x + dx, pos.y + dy);
    if !map.blocked[idx] {
        return (pos.x + dx, pos.y + dy)
    } else {
        return (pos.x, pos.y)
    }
}