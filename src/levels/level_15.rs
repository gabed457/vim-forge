use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 15: "Precision Strikes" — f/t find-motions and ;/, repeat.
///
/// Four stacked smelting lines. Everything is wired correctly EXCEPT the
/// four smelters, which all face the wrong way (Up / Left / Down / Up).
/// The intended play, straight from the hints:
///
///   fs      — jump to the first (mis-rotated) smelter anchor
///   ~       — rotate it until it faces Right (1-3 presses)
///   ;       — repeat the find: jump to the NEXT smelter, fix, repeat
///   ,       — repeat backward to double-check earlier lines
///   tb / fb — hop (short of) the next output bin to watch deliveries
///
/// Completion is production-based (8 iron ingots) so the rotations must
/// actually happen — pressing the find keys alone completes nothing.
///
/// Line layout for band b (belt row b+1):
///   OreDeposit (2,b) out (4,b+1) -> belts (5..=13, b+1) ->
///   Smelter (14,b) [WRONG FACING] in (14,b+1) out (16,b+1) ->
///   belts (17..=29, b+1) -> OutputBin (30,b) in (30,b+1)
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // (band anchor row, wrong smelter facing)
    let bands = [
        (2usize, Facing::Up),
        (8, Facing::Left),
        (14, Facing::Down),
        (20, Facing::Up),
    ];

    for &(b, smelter_facing) in &bands {
        entities.push(LevelEntity {
            x: 2,
            y: b,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        });
        for x in 5..=13 {
            entities.push(LevelEntity {
                x,
                y: b + 1,
                entity_type: EntityType::BasicBelt,
                facing: Facing::Right,
                player_placed: false,
            });
        }
        // The fault: smelter rotated the wrong way.
        entities.push(LevelEntity {
            x: 14,
            y: b,
            entity_type: EntityType::Smelter,
            facing: smelter_facing,
            player_placed: false,
        });
        for x in 17..=29 {
            entities.push(LevelEntity {
                x,
                y: b + 1,
                entity_type: EntityType::BasicBelt,
                facing: Facing::Right,
                player_placed: false,
            });
        }
        entities.push(LevelEntity {
            x: 30,
            y: b,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    LevelConfig {
        number: 15,
        name: "Precision Strikes",
        map_width: 44,
        map_height: 26,
        entities,
        objective: "4 smelters face the wrong way. fs finds one, ~ fixes, ; repeats. 8 ingots.",
        hints: vec![
            "Four smelting lines, four smelters rotated wrong. Press fs to FIND the next Smelter.",
            "f jumps TO a building: fs smelter, fb bin, fo deposit, fc belt, fm merger. F looks backward.",
            "On each smelter press ~ until it faces Right, then ; repeats the last find — next smelter!",
            ", repeats the find in the OPPOSITE direction. t and T stop one tile short: try tb.",
            "Fix all four smelters so every line smelts. Deliver 8 iron ingots to finish.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::DeliverIngots(8),
    }
}
