use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use serde::{Serialize, Deserialize};
use super::signaling::{ElementalDamageKind};

//------------------------------------------------------------------
// Game effects components
//
// These components tag entities as having certain effects when used withing
// various game contexts. Some of these fall into general classes:
//
//   - [...]WhenUsed components indicate effects of an item/spell when selected
//   from the use item menu, i.e., when used as an untargeted effect.
//   - [...]WhenThrown components indicate effects of an item/spell when thrown
//   (as from the throw menu, in the cast of the player).
//   - [...]WhenCast components inficate effects of an item/spell when cast (as
//   from the spell menu, in the cast of the player).
//
//  Note that some entities (for example, potions) have different effects when
//  thrown vs. untargeted. Some weapon specials allow the cast of two targeted
//  effects, thrown and cast.
//
//   - [...]WhenEncroachedUpon indicate effects that an entity has on any other
//     entity occupying the same tile.
//------------------------------------------------------------------

// Component for effects that increase the user's maximum hp.
#[derive(Component, ConvertSaveload, Clone)]
pub struct IncreasesMaxHpWhenUsed {
    pub amount: i32
}
// Component for effects that cause the user's spell to charge more quickly.
#[derive(Component, ConvertSaveload, Clone)]
pub struct DecreasesSpellRechargeWhenUsed {
    pub percentage: i32
}

// Components for effects that inflict damage.
#[derive(Clone, ConvertSaveload)]
pub struct InflictsDamageData {
    pub damage: i32,
    pub kind: ElementalDamageKind
}
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsDamageWhenThrown (pub InflictsDamageData);
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsDamageWhenCast (pub InflictsDamageData);


// Component for entities that inflict damage on any other entity occupying the
// same position. Examples are steam and chill.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsDamageWhenEncroachedUpon {
    pub damage: i32,
    pub kind: ElementalDamageKind
}


// Components for effects that inflict freezing and the chill status.
#[derive(Clone, ConvertSaveload)]
pub struct InflictsFreezingData {
    pub turns: i32
}
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsFreezingWhenCast (pub InflictsFreezingData);
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsFreezingWhenThrown (pub InflictsFreezingData);


// Components for effects that inflict the burning status.
#[derive(Clone, ConvertSaveload)]
pub struct InflictsBurningData {
    pub turns: i32,
    pub tick_damage: i32
}
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsBurningWhenThrown (pub InflictsBurningData);
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsBurningWhenCast (pub InflictsBurningData);


// Component for effects that buff melee attack damage for a number of turns.
#[derive(Component, ConvertSaveload, Clone)]
pub struct BuffsMeleeAttackWhenCast {
    pub turns: i32,
}

// Component for effects that buff melee physical defense for a number of
// turns.
#[derive(Component, ConvertSaveload, Clone)]
pub struct BuffsPhysicalDefenseWhenCast {
    pub turns: i32,
}

// Component for effects that inflict the burning status on any other entity
// occupyting the same space.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsBurningWhenEncroachedUpon {
    pub turns: i32,
    pub tick_damage: i32

}
// Component signals that any entity encroaching on the owner will have burning
// removed. Used on water.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct RemoveBurningWhenEncroachedUpon {}

// Component signals that any fire encroaching on the owner will be immediately
// dissapated.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct DissipateFireWhenEncroachedUpon {}

// Component signals that the burning status should be removed from the owner
// every upkeep turn.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct RemoveBurningOnUpkeep {}

// Component for entities that inflict (maybe) inflict burning on adjacent
// entities.
#[derive(Component, ConvertSaveload, Clone)]
pub struct ChanceToInflictBurningOnAdjacentEntities {
    pub chance: i32
}

// Component for effects that inflict the freezing status on any other entity
// occupyting the same space.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsFreezingWhenEncroachedUpon {
    pub turns: i32,
}

// Component for effects that grant a MeleeAttackBonus
#[derive(Component, ConvertSaveload, Clone)]
pub struct GrantsMeleeDefenseBonus {
    pub bonus: i32
}

// An entity with this component, when used, restores all of the users hp.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ProvidesFullHealing {}

// An entity with this component, when used, restores a single charge to all the
// user's spells.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ProvidesFullSpellRecharge {}

// An entity with this component, when used, restores the user to well fed.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ProvidesFullFood {}

// An entity with this component, when used, grants fire immunity.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ProvidesFireImmunityWhenUsed {
    pub turns: i32
}

// An entity with this component, when used, grants chill immunity.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ProvidesChillImmunityWhenUsed {
    pub turns: i32
}

// An entity with this component, when used, teleports the user to a random
// position.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct MovesToRandomPosition {}

// An eintity with this compoent move the using entity to a targeted position.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct MoveToPositionWhenCast {}