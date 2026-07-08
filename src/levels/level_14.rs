use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 14: "Word Hops" — word-motion navigation across scattered clusters.
///
/// The map is two long rows of small entity clusters ("words") separated by
/// wide stretches of empty ground, so hjkl-spam is painful and the word
/// motions are dramatically faster:
///   w  — next entity (hops empty ground to the next cluster's start)
///   b  — previous entity
///   e  — end of the current cluster (or of the next one when at an end)
///   ge — end of the PREVIOUS cluster
///   W/B/E — "big word": next/prev entity of a DIFFERENT building type,
///           which hops whole same-type clusters in one keystroke
///   ^ / g_ — first / last entity in the row
///
/// Clusters alternate belt/wall so W and B are genuinely different from w/b.
fn cluster(
    entities: &mut Vec<LevelEntity>,
    xs: std::ops::RangeInclusive<usize>,
    y: usize,
    entity_type: EntityType,
) {
    for x in xs {
        entities.push(LevelEntity {
            x,
            y,
            entity_type,
            facing: Facing::Right,
            player_placed: false,
        });
    }
}

pub fn config() -> LevelConfig {
    let mut entities = Vec::new();
    let belt = EntityType::BasicBelt;
    let wall = EntityType::Wall;

    // Row 6: five clusters, alternating belt / wall.
    cluster(&mut entities, 5..=9, 6, belt); // A: belts (5..9, 6)
    cluster(&mut entities, 20..=25, 6, wall); // B: walls (20..25, 6)
    cluster(&mut entities, 40..=46, 6, belt); // C: belts (40..46, 6)
    cluster(&mut entities, 60..=68, 6, wall); // D: walls (60..68, 6)
    cluster(&mut entities, 80..=85, 6, belt); // E: belts (80..85, 6)

    // Row 12: four more clusters (decoys / extra practice ground).
    cluster(&mut entities, 10..=14, 12, wall);
    cluster(&mut entities, 30..=36, 12, belt);
    cluster(&mut entities, 55..=60, 12, wall);
    cluster(&mut entities, 75..=82, 12, belt);

    LevelConfig {
        number: 14,
        name: "Word Hops",
        map_width: 90,
        map_height: 18,
        entities,
        objective: "Hop the scattered clusters: use w, b, e, ge, E and ^.",
        hints: vec![
            "Clusters are 'words'. w jumps to the next entity — over empty ground that's the next cluster!",
            "e jumps to the END of your cluster; pressed again at an end, it hops to the next cluster's end.",
            "b jumps back to the previous entity; ge jumps back to the previous cluster's END.",
            "W, B and E are big-word hops: they skip to the next DIFFERENT building type in one press.",
            "^ snaps to the first entity in the row, g_ to the last. Checklist: w b e ge E ^.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::UseCommands(vec![
            "w".to_string(),
            "b".to_string(),
            "e".to_string(),
            "ge".to_string(),
            "E".to_string(),
            "^".to_string(),
        ]),
    }
}
