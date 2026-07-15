use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 26: "Macro Empire" — the macro capstone: record ONE full widget
/// line (motions + insert placements + ~ corner fixes) into qa, then
/// replay it with @a, @@ and a counted 5@a to stamp out an empire.
///
/// Layout (52×52) — EIGHT identical bands, 6 rows apart:
///   band i (i = 0..8): OreDeposit(1, 2+6i) and OutputBin(48, 2+6i),
///   nothing in between.
///
/// The proven one-band macro (band top y, belt row r=y+1), recorded once
/// at (4, 3):
///   i cccccc k s j c×27 k a jj ccccc <Esc>  — belts, smelter(10,y),
///     main run, assembler(40,y), output run
///   h ~~~ k ic <Esc>                        — corner up into the bin
///   9h ~ j ic <Esc>                         — corner down into input B
///   36h 5j                                  — park on the NEXT band's start
/// Ending one band lower at the same column is what makes @a chainable.
///
/// Completion: 24 widgets (3 per band) — building eight bands by hand is
/// ~300 keystrokes; qa + @a + @@ + 5@a is ~60.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    for i in 0..8 {
        let y = 2 + i * 6;
        entities.push(LevelEntity {
            x: 1,
            y,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        });
        entities.push(LevelEntity {
            x: 48,
            y,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    LevelConfig {
        number: 26,
        name: "Macro Empire",
        map_width: 52,
        map_height: 52,
        entities,
        objective: "Eight empty bands. Record ONE line as qa, then @a, @@ and 5@a build the rest.",
        hints: vec![
            "Eight identical bands. Build the FIRST one inside a recording: start at (4,3), press qa.",
            "One band: i, 6 belts, k, s, j, 27 belts, k, a, jj, 5 belts, Esc — then fix the two corners with ~.",
            "End the recording where the NEXT band starts: 36h 5j, then q. Relative motions make it replayable.",
            "@a replays the macro once. @@ replays the LAST macro again — no register needed.",
            "Counts multiply macros: 5@a stamps the five remaining bands in one keystroke. 24 widgets to win.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ProduceWidgets(24),
    }
}
