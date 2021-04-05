# Rusty-Rogue

![Gameplay Screenshot](img/screnshot.png)

This is a tradtional turn based Roguelike dungeon crawling game build in the Rust programming language. It started by following the excellent [Rust Roguelike Tutorial](https://bfnightly.bracketproductions.com/), but had diverged a large amount at this point.

## Building

The project is built with rusts `Cargo` tool:

```
cd rogue
cargo build
```

To run:

```
cargo run
```

## Controlls

Controls use the arrow keys, but vim keys offer more control:

  - `L`: Move right.
  - `K`: Move up.
  - `J`: Move down.
  - `H`: Move left.
  - `Y`: Move diagonally up and left.
  - `U`: Move diagonally up and right.
  - `B`: Move diagonally down and left.
  - `N`: Move diagonally down and right.
  - `Z`: Passes the turn without moving.

If you pass the turn with *no* monsters in view, you will gain a little bit of health. Don't abuse this, or you will starve!

Additionally, various menus are available to drink, throw potions, cast spells, and equip gear:

  - `I`: Opens the item menu for *using* items.
  - `T`: Opens an item menu for *throwing* items.

The same item may have different effects when used or thrown. For example, a fire potion will grant fire invulnrability when *used*, but will act like a firebomb when *thrown*.

  - `S`: Open the spell menu. Cast spells.

## Monsters

Various monsters inhabit the dungeon, and will attempt to stop you from descending.

### Goblins `g`

#### Basic Goblin 🟤
A basic goblin that will attamept to chase you down and bash you until you are dead.

#### Goblin Cleric 🔴
A support spellcaster that will heal any injured allies.

#### Goblin Enchanter ⚪
A support spellcaster that will apply buffs to allies.

#### Goblin Firemanser 🟠
An attack spellcaster that will cast fire spells on the player, inflicting the burning status effect.

#### Goblin Chillmanser 🔵
An attack spellcaster that will cast chill spells on the player, inflicting the frozen status effect.

### Orcs `O`

#### Basic Orc
A beefed up version of the basic Goblin.