use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 24: "Search & Destroy" — ? backward search, * / # search under
/// cursor, n/N (with counts), and :noh.
///
/// Layout (44×28) — six ore→bin lines; each line has exactly ONE wall
/// bricked into the belt run at an unpredictable column:
///
///   Line y=2..22 step 4: OreDeposit(1,y), belts (4..=38, y+1) Right
///   except one WALL, OutputBin(39,y).
///   Wall positions: (12,3), (30,7), (8,11), (25,15), (18,19), (35,23).
///
/// The walls are scattered left/right so eyeballing and hjkl-ing to them
/// is slow — /wall<CR> finds the first, * re-searches from the one under
/// the cursor, # and ?wall<CR> hunt backward, n/N walk the match list,
/// and r c fixes each one on arrival. :noh clears the highlight when done.
///
/// Completion: every bin receives an item — all six walls must be found
/// and replaced.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    let belt = |x: usize, y: usize| LevelEntity {
        x,
        y,
        entity_type: EntityType::BasicBelt,
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

    // One wall per line, scattered columns (line index -> wall x).
    let wall_x = [12usize, 30, 8, 25, 18, 35];

    for (i, wx) in wall_x.iter().enumerate() {
        let y = 2 + i * 4;
        entities.push(big(1, y, EntityType::OreDeposit));
        entities.push(big(39, y, EntityType::OutputBin));
        for x in 4..=38 {
            if x == *wx {
                entities.push(wall(x, y + 1));
            } else {
                entities.push(belt(x, y + 1));
            }
        }
    }

    LevelConfig {
        number: 24,
        name: "Search & Destroy",
        map_width: 44,
        map_height: 28,
        entities,
        objective: "Six walls bricked into six belt lines. Hunt them with / ? * # n N, fix with r c.",
        hints: vec![
            "One wall is bricked into each of the 6 lines. /wall<CR> jumps straight to the first.",
            "Fix a wall under the cursor with r c — replace it with a belt. Then n = next match.",
            "* searches for whatever is UNDER the cursor; # does the same backward.",
            "?wall<CR> hunts backward; N reverses your last direction; counts work: 2n skips one.",
            ":noh clears the search highlight once every line is flowing again.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::Custom("all_5_clusters_producing".to_string()),
    }
}
