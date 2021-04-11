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
//     from the use item menu, i.e., when used as an untargeted effect.
//   - [...]WhenTargeted components inficate effects of an item/spell when used as
//     a targeted effect.
//
//  Note that some entities (for example, potions) have different effects when
//  targeted vs. untargeted.
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

// Component for effects that inflict damage when thrown or cast.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsDamageWhenTargeted {
    pub damage: i32,
    pub kind: ElementalDamageKind
}

// Component for entities that inflict damage on any other entity occupying the
// same position.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsDamageWhenEncroachedUpon {
    pub damage: i32,
    pub kind: ElementalDamageKind
}

// Component for effects that inflict the frozen status.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsFreezingWhenTargeted {
    pub turns: i32
}

// Component for effects that inflict the burning status when used as a targeted
// effect.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsBurningWhenTargeted {
    pub turns: i32,
    pub tick_damage: i32
}

// Component for effects that buff melee attack damage for a number of turns.
#[derive(Component, ConvertSaveload, Clone)]
pub struct BuffsMeleeAttackWhenTargeted {
    pub turns: i32,
}

// Component for effects that buff melee physical defense for a number of
// turns.
#[derive(Component, ConvertSaveload, Clone)]
pub struct BuffsPhysicalDefenseWhenTargeted {
    pub turns: i32,
}

// Component for effects that inflict the burning status on any other entity
// occupyting the same space.
#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsBurningWhenEncroachedUpon {
    pub turns: i32,
    pub tick_damage: i32
}

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
pub struct GrantsMeleeAttackBonus {
    pub bonus: i32
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
pub struct MoveToPositionWhenTargeted {}