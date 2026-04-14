use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 12: "Marks & Navigation" — Set marks and navigate between 4 factory clusters.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // Cluster 1: top-left (belt_y=4)
    entities.push(LevelEntity {
        x: 2, y: 3,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 5..=10 {
        entities.push(LevelEntity {
            x, y: 4,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 21, y: 3,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Cluster 2: top-right (belt_y=4)
    entities.push(LevelEntity {
        x: 62, y: 3,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 65..=69 {
        entities.push(LevelEntity {
            x, y: 4,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 77, y: 3,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Cluster 3: bottom-left (belt_y=36)
    entities.push(LevelEntity {
        x: 2, y: 35,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 5..=9 {
        entities.push(LevelEntity {
            x, y: 36,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 21, y: 35,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Cluster 4: bottom-right (belt_y=36)
    entities.push(LevelEntity {
        x: 62, y: 35,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 65..=68 {
        entities.push(LevelEntity {
            x, y: 36,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 77, y: 35,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    LevelConfig {
        number: 12,
        name: "Marks & Navigation",
        map_width: 82,
        map_height: 40,
        entities,
        objective: "Use marks (ma/'a) to navigate between 4 corner clusters. Fix them all.",
        hints: vec![
            "Huge map! 4 partially-built clusters in each corner. Use marks to teleport between them.",
            "Move to the first cluster. Press ma to set mark 'a' at your cursor position.",
            "Visit each cluster corner and set marks: ma, mb, mc, md. Press 'a to jump to mark 'a'.",
            "Each cluster needs more belts and a smelter. Use i, c for belts, k then s for smelters.",
            "Use 'a/'b/'c/'d to hop between clusters as you build. Produce 4 widgets to win!",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ProduceWidgets(4),
    }
}
