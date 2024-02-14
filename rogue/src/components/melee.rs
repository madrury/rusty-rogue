use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{ConvertSaveload, Marker};
use serde::{Serialize, Deserialize};
use specs::error::NoError;

// Component holding the combat statistics of an entity.
// TODO: This should probably be broken into two comonents? HealthStats and
// MeleeStats?
#[derive(Component, ConvertSaveload, Clone)]
pub struct CombatStats {
    // Health.
    pub max_hp: i32,
    pub hp: i32,
    // Raw melee stats.
    pub defense: i32,
    pub power: i32
}
impl CombatStats {
    pub fn take_damage(&mut self, damage: i32) {
        self.hp = i32::max(0, self.hp - damage)
    }
    pub fn full_heal(&mut self) {
        self.hp = self.max_hp
    }
    pub fn heal_amount(&mut self, amount: i32) {
        self.hp = i32::min(self.max_hp, self.hp + amount)
    }
    pub fn increase_max_hp(&mut self, amount: i32) {
        self.max_hp += amount;
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub enum WeaponSpecialKind {
    ThrowWithoutExpending
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WeaponSpecial {
    pub regen_time: i32,
    pub time: i32,
    pub kind: WeaponSpecialKind
}
impl WeaponSpecial {
    // Returns value indicating if recharged.
    pub fn tick(&mut self) -> bool {
        if self.time == self.regen_time {
            return false
        }
        self.time = i32::min(self.time + 1, self.regen_time);
        return self.time == self.regen_time
    }
}