use bevy::prelude::*;
use rand::{rngs::ThreadRng, thread_rng, Rng};

use crate::components::ants::{AntColorKind, AntStyle};

#[derive(Debug, Clone, Copy, Resource, Reflect)]
pub struct Clues {
    pub z0_primary_color: bool,
    pub z0_secondary_color: bool,
    pub pheromone_view_charges: usize,
    // FIXME: separate into UI resource
    pub ui_text: Entity,
    pub ui_container: Entity,
    pub ui_ant_clue: Entity,
    pub ant_clue: Entity,
}

impl Clues {
    pub fn reveal_next(&mut self, rng: &mut ThreadRng, ant_styles: &mut Query<&mut AntStyle>) {
        match (self.z0_primary_color, self.z0_secondary_color) {
            (true, true) => {
                self.pheromone_view_charges += 1;
            }
            (true, false) => {
                self.reveal_secondary_color(rng, ant_styles);
            }
            (false, true) => {
                self.reveal_primary_color(rng, ant_styles);
            }
            (false, false) => {
                if rng.gen_bool(0.5) {
                    self.reveal_primary_color(rng, ant_styles);
                } else {
                    self.reveal_secondary_color(rng, ant_styles);
                }
            }
        }
    }

    fn reveal_primary_color(&mut self, rng: &mut ThreadRng, ant_styles: &mut Query<&mut AntStyle>) {
        self.z0_primary_color = true;
        let Ok(mut ant_style) = ant_styles.get_mut(self.ant_clue) else {
            return;
        };
        ant_style.color_primary_kind = AntColorKind::new_random(rng); // FIXME: get Z0 color
        ant_style.color_secondary_kind =
            AntColorKind::new_random_from_primary(rng, &ant_style.color_primary_kind);
    }

    fn reveal_secondary_color(
        &mut self,
        rng: &mut ThreadRng,
        ant_styles: &mut Query<&mut AntStyle>,
    ) {
        self.z0_secondary_color = true;
        let Ok(mut ant_style) = ant_styles.get_mut(self.ant_clue) else {
            return;
        };
        ant_style.color_primary = ant_style.color_primary_kind.generate_color(rng);
        ant_style.color_secondary = ant_style.color_secondary_kind.generate_color(rng);
    }
}

#[derive(Event)]
pub enum ClueEvent {
    Found,
}

pub fn clues_receive_events(
    mut events: EventReader<ClueEvent>,
    mut clues: ResMut<Clues>,
    mut ant_styles: Query<&mut AntStyle>,
) {
    for event in events.read() {
        match event {
            ClueEvent::Found => clues.reveal_next(&mut thread_rng(), &mut ant_styles),
        }
    }
}

pub fn found_clue(mut event: EventWriter<ClueEvent>) {
    event.send(ClueEvent::Found);
}
