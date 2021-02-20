use specs::prelude::*;
use super::{Viewshed, Position, Map, Monster};
use rltk::{field_of_view, Point, console};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Monster>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (viewshed, pos, monster)  = data;
        for (viewshed, pos, monster) in (&viewshed, &pos, &monster).join() {
            console::log("Monster ponders their own existence.")
        }
    }
}