use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 20: "Join & Open" — J joins belt clusters across gaps.
///
/// Three ore rows, each broken by TWO gaps between belt clusters:
///
///   deposit -> belts(5..14) [GAP 15..19] belts(20..29) [GAP 30..34]
///   belts(35..44) -> bin
///
/// `J` (join) fills the gap between the cluster under the cursor and the
/// next cluster on the row with right-facing belts — exactly vim's J,
/// splicing "lines" together. With a count, `2J` bridges both gaps of a row
/// in a single keystroke. That is 30 tiles of conveyor from three keys per
/// row — dramatically faster than insert-mode c-spam.
///
/// Completion: DeliverOre(12) — the ore only flows across joined rows, so
/// J (or laboriously hand-laying every gap) is mandatory.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    for &b in &[3usize, 9, 15] {
        let y = b + 1; // belt row

        // OreDeposit (2,b): output port (4,y).
        entities.push(LevelEntity {
            x: 2,
            y: b,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        });

        // Three belt clusters with gaps at 15..=19 and 30..=34.
        for x in 5..=14 {
            entities.push(LevelEntity {
                x,
                y,
                entity_type: EntityType::BasicBelt,
                facing: Facing::Right,
                player_placed: false,
            });
        }
        for x in 20..=29 {
            entities.push(LevelEntity {
                x,
                y,
                entity_type: EntityType::BasicBelt,
                facing: Facing::Right,
                player_placed: false,
            });
        }
        for x in 35..=44 {
            entities.push(LevelEntity {
                x,
                y,
                entity_type: EntityType::BasicBelt,
                facing: Facing::Right,
                player_placed: false,
            });
        }

        // OutputBin (45,b): input port (45,y).
        entities.push(LevelEntity {
            x: 45,
            y: b,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    LevelConfig {
        number: 20,
        name: "Join & Open",
        map_width: 52,
        map_height: 22,
        entities,
        objective: "3 rows, 6 gaps. J joins clusters with belts (2J per row). 12 ore.",
        hints: vec![
            "Every ore row is broken by two GAPS between belt clusters.",
            "J (join) fills the gap from your cluster to the next one with right-facing belts!",
            "Stand anywhere on a row's first cluster and press J — the first gap splices itself.",
            "Counts work: 2J joins twice — both gaps of a row bridged with one keystroke.",
            "Insert variants: a=append right, A=append at row end, I=first entity, o/O=row below/above. Deliver 12 ore.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::DeliverOre(12),
    }
}
