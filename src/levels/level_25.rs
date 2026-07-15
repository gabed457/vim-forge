use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 25: "Global Substitute" — :s/old/new/g on a row, & to repeat it,
/// :N,Ms/// row ranges, :%s/// whole-map, and :g/pat/d mass deletion.
///
/// Layout (42×24) — four ore→bin lines whose belt runs were built out of
/// PIPES (pipes don't carry solids!), a fifth line completely buried under
/// a wall field, and more wall debris below:
///
///   Lines y=2,6,10,14: OreDeposit(1,y), PIPES (4..=35, y+1),
///     OutputBin(36,y).
///   Line 5 (y=17): OreDeposit(1,17), WALLS (4..=35, 18), OutputBin(36,17).
///   Debris: walls dotted along rows 20-21.
///
/// Intended flow (exactly what the hints teach):
///   row 3:  :s/pipe/belt/g      — substitute on the current row
///   row 7:  &                   — repeat the last :s on this row
///   rows 11+15:  :12,16s/pipe/belt/g   — row-range form (1-indexed)
///   (or just :%s/pipe/belt/g for everything at once)
///   :g/wall/d               — delete EVERY wall on the map in one command
///   row 18: J               — one Join belts the excavated line 5
///
/// Replacing ~128 pipes and deleting ~45 walls by hand is the punishment
/// for skipping the command line.
///
/// Completion: every bin receives an item (all five lines flow).
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    let pipe = |x: usize, y: usize| LevelEntity {
        x,
        y,
        entity_type: EntityType::Pipe,
        facing: Facing::Right,
        player_placed: false,
    };
    let wall = |x: usize, y: usize| LevelEntity {
        x,
        y,
        entity_type: EntityType::Wall,
        facing: Facing::Right,
        player_placed: false,
    };
    let big = |x: usize, y: usize, entity_type: EntityType| LevelEntity {
        x,
        y,
        entity_type,
        facing: Facing::Right,
        player_placed: false,
    };

    // Four "plumbed" lines: some fool laid pipes instead of belts.
    for &y in &[2usize, 6, 10, 14] {
        entities.push(big(1, y, EntityType::OreDeposit));
        entities.push(big(36, y, EntityType::OutputBin));
        for x in 4..=35 {
            entities.push(pipe(x, y + 1));
        }
    }

    // Line 5: buried under the wall field.
    entities.push(big(1, 17, EntityType::OreDeposit));
    entities.push(big(36, 17, EntityType::OutputBin));
    for x in 4..=35 {
        entities.push(wall(x, 18));
    }

    // Wall debris south of the factory.
    for x in (6..=30).step_by(2) {
        entities.push(wall(x, 20));
    }
    for x in (7..=29).step_by(4) {
        entities.push(wall(x, 21));
    }

    LevelConfig {
        number: 25,
        name: "Global Substitute",
        map_width: 42,
        map_height: 24,
        entities,
        objective: "Pipes where belts belong and a wall-buried line. :s, &, ranges and :g/wall/d.",
        hints: vec![
            "Four lines were plumbed with PIPES — solids won't flow. On row 3 type :s/pipe/belt/g",
            "Move to the next bad row and press & — it repeats the last :s on the current row.",
            "Ranges scope substitutions: :12,16s/pipe/belt/g fixes rows 12-16. :%s/.../g does ALL rows.",
            "Line 5 is buried under walls, with more debris below. :g/wall/d deletes every wall at once.",
            "Stand on the excavated line 5 (row 18, on the deposit) and press J to belt the whole span.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::Custom("all_5_clusters_producing".to_string()),
    }
}
