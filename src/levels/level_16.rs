use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 16: "Pages & Paragraphs" — long-range vertical navigation.
///
/// A HUGE map (60 x 120) with six "station" blocks — dense 3-row bands of
/// belts and walls — separated by ~17 empty rows. A viewport is only ~44
/// rows tall, so most of the map is off screen and j/k-spam is hopeless.
/// The taught keys:
///
///   } / {           — next / previous station block (paragraph motion)
///   Ctrl-d / Ctrl-u — half page down / up
///   Ctrl-f / Ctrl-b — full page down / up
///   zz / zt / zb    — scroll the view so the cursor is centered/top/bottom
///
/// Completion: use each of { } Ctrl-d Ctrl-u Ctrl-f Ctrl-b zz zt zb once.
///
/// NOTE: the curriculum also wanted H/M/L (viewport top/middle/bottom)
/// here, but the parser only accepts them with EMPTY modifiers while
/// shifted letters always arrive with the SHIFT modifier — so H/M/L are
/// currently unreachable in normal mode (engine bug, reported). This level
/// teaches the scroll family that actually works.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // Six station bands, three rows each, spread down the tall map.
    for &band in &[5usize, 25, 45, 65, 85, 105] {
        for row in band..band + 3 {
            // Wall bookends.
            for &x in &[4usize, 52] {
                entities.push(LevelEntity {
                    x,
                    y: row,
                    entity_type: EntityType::Wall,
                    facing: Facing::Right,
                    player_placed: false,
                });
            }
            // A stretch of belts with wall pylons every 8 tiles.
            for x in (6..=50).step_by(2) {
                let entity_type = if x % 8 == 0 {
                    EntityType::Wall
                } else {
                    EntityType::BasicBelt
                };
                entities.push(LevelEntity {
                    x,
                    y: row,
                    entity_type,
                    facing: Facing::Right,
                    player_placed: false,
                });
            }
        }
    }

    LevelConfig {
        number: 16,
        name: "Pages & Paragraphs",
        map_width: 60,
        map_height: 120,
        entities,
        objective: "120 rows tall! Use { } Ctrl-d/u Ctrl-f/b and zz zt zb to traverse it.",
        hints: vec![
            "This map is 120 rows tall — six station blocks separated by empty ground.",
            "} jumps to the NEXT station block, { to the previous one (paragraph motions).",
            "Ctrl-d / Ctrl-u scroll half a page down/up; Ctrl-f / Ctrl-b a full page.",
            "zz scrolls the view so the cursor is CENTERED — reorient after any big jump.",
            "zt / zb put the cursor row at the view's top / bottom. Use every scroll key once!",
        ],
        allowed_commands: None,
        completion: CompletionCondition::UseCommands(vec![
            "{".to_string(),
            "}".to_string(),
            "ctrl-d".to_string(),
            "ctrl-u".to_string(),
            "ctrl-f".to_string(),
            "ctrl-b".to_string(),
            "zz".to_string(),
            "zt".to_string(),
            "zb".to_string(),
        ]),
    }
}
