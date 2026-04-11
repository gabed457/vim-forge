use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 12: "Marks & Navigation" — Set marks and navigate between 4 factory clusters.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // Cluster 1: top-left corner (around 5,5)
    entities.push(LevelEntity {
        x: 2,
        y: 3,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    // Partial conveyor chain
    for x in 3..=8 {
        entities.push(LevelEntity {
            x,
            y: 3,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // Missing smelter and rest of chain
    entities.push(LevelEntity {
        x: 15,
        y: 3,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Cluster 2: top-right corner (around 65,5)
    entities.push(LevelEntity {
        x: 60,
        y: 3,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 61..=65 {
        entities.push(LevelEntity {
            x,
            y: 3,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 73,
        y: 3,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Cluster 3: bottom-left corner (around 5,35)
    entities.push(LevelEntity {
        x: 2,
        y: 35,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 3..=7 {
        entities.push(LevelEntity {
            x,
            y: 35,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 15,
        y: 35,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Cluster 4: bottom-right corner (around 65,35)
    entities.push(LevelEntity {
        x: 60,
        y: 35,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 61..=64 {
        entities.push(LevelEntity {
            x,
            y: 35,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 73,
        y: 35,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    LevelConfig {
        number: 12,
        name: "Marks & Navigation",
        map_width: 80,
        map_height: 40,
        entities,
        objective: "Set marks, navigate, fix all 4 clusters.",
        hints: vec![
            "Use ma to set mark 'a' at the current position.",
            "Use 'a to jump back to mark 'a'.",
            "Set marks at each cluster so you can quickly jump between them.",
            "Each cluster is partially built; complete the smelter chains.",
            "Marks save your position even when you scroll far away.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ProduceWidgets(4),
    }
}
