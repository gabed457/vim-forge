use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 19: "The Upgrader" — belt tiers: Ctrl-a / Ctrl-x and the gU / gu
/// operators.
///
/// A complete, working smelting line runs on slow BasicBelts (the tier
/// ladder is BasicBelt -> FastBelt -> ExpressBelt; machines have no ladder,
/// so range upgrades skip them safely). Below it, a scrap row of expensive
/// ExpressBelts was installed by mistake.
///
///   Ctrl-a — upgrade the entity under the cursor one tier
///   Ctrl-x — downgrade one tier
///   gUU    — upgrade EVERYTHING upgradable on the row (gU+motion / gUiw
///            also work: gU is a full operator)
///   guu    — downgrade the row (fix the wasteful scrap row)
///
/// Completion: UseCommands([ctrl-a, ctrl-x, gU, gu]) — the ingots visibly
/// speed up as the line is upgraded.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // Working line: deposit (2,5) out (4,6) -> belts -> smelter (12,5)
    // in (12,6) out (14,6) -> belts -> bin (38,5) in (38,6).
    entities.push(LevelEntity {
        x: 2,
        y: 5,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 5..=11 {
        entities.push(LevelEntity {
            x,
            y: 6,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 12,
        y: 5,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 15..=37 {
        entities.push(LevelEntity {
            x,
            y: 6,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 38,
        y: 5,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Scrap row 11: over-tiered ExpressBelts to downgrade with guu.
    for x in 10..=30 {
        entities.push(LevelEntity {
            x,
            y: 11,
            entity_type: EntityType::ExpressBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    LevelConfig {
        number: 19,
        name: "The Upgrader",
        map_width: 50,
        map_height: 16,
        entities,
        objective: "Speed up the slow line: use Ctrl-a, Ctrl-x, gU and gu.",
        hints: vec![
            "This line crawls on BasicBelts. Ctrl-a upgrades the belt under the cursor one tier.",
            "Ctrl-x downgrades one tier. The ladder: Basic -> Fast -> Express (walls upgrade too).",
            "gU is the upgrade OPERATOR: gUU upgrades the whole row, gUiw a cluster, gU$ to row end.",
            "gu downgrades a range: the ExpressBelt scrap on row 11 is wasted — guu it down.",
            "Checklist: Ctrl-a, Ctrl-x, gU, gu. Machines have no tiers; range ops skip them.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::UseCommands(vec![
            "ctrl-a".to_string(),
            "ctrl-x".to_string(),
            "gU".to_string(),
            "gu".to_string(),
        ]),
    }
}
