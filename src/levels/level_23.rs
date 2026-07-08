use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 23: "Jump History" — the jumplist (Ctrl-o / Ctrl-i), `` / ''
/// backflips, and a marks review, on a map big enough that hjkl-spam hurts.
///
/// Layout (80×36) — four ore→bin clusters, one in each corner, every one
/// with a 3-tile belt gap:
///
///   Each cluster: OreDeposit(dx,dy) 3×2, belts (dx+3..dx+8, dy+1) Right,
///   GAP (3 tiles), belts (dx+12..dx+17, dy+1) Right, OutputBin(dx+18,dy).
///   Corners: (2,2), (58,2), (2,30), (58,30).
///
///   A debris field of walls sits mid-map purely as w/e decoys — word
///   motions crawl through it, while /ore + n teleport corner to corner
///   (each search records a jumplist entry, so Ctrl-o walks history back
///   and Ctrl-i forward; `` flips between the last two spots).
///
/// Intended flow: /ore<CR> → corner 1, j onto the belt row, J closes the
/// gap; n → next corner, repeat, four times; then Ctrl-o / Ctrl-i / `` /
/// marks to hop the history like the hints teach.
///
/// Completion: every bin receives an item — all four distant clusters must
/// actually be visited and repaired.
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

    // Four corner clusters with a 3-tile gap each.
    for &(dx, dy) in &[(2usize, 2usize), (58, 2), (2, 30), (58, 30)] {
        entities.push(big(dx, dy, EntityType::OreDeposit));
        entities.push(big(dx + 18, dy, EntityType::OutputBin));
        for x in (dx + 3)..=(dx + 8) {
            entities.push(belt(x, dy + 1));
        }
        // gap: dx+9 ..= dx+11
        for x in (dx + 12)..=(dx + 17) {
            entities.push(belt(x, dy + 1));
        }
    }

    // Mid-map debris: decoys that make w/e crawling slow and search fast.
    for &(x, y) in &[
        (30usize, 14usize),
        (34, 15),
        (38, 16),
        (42, 17),
        (46, 18),
        (33, 19),
        (37, 20),
        (41, 21),
        (45, 13),
        (29, 18),
    ] {
        entities.push(wall(x, y));
    }

    LevelConfig {
        number: 23,
        name: "Jump History",
        map_width: 80,
        map_height: 36,
        entities,
        objective: "Four broken corners. Search-jump to each, fix it, and surf Ctrl-o/Ctrl-i home.",
        hints: vec![
            "Four clusters, four corners, each with a belt gap. Walking there with hjkl is pain.",
            "/ore<CR> teleports to the next deposit; n repeats the search. Each jump is remembered.",
            "At a cluster: j down onto the belt row, then J fills the gap to the next belt run.",
            "Ctrl-o walks BACK through your jump history; Ctrl-i walks forward again. Try a few!",
            "`` flips between the last two jump spots. ma marks a spot, `a returns exactly there.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::Custom("all_5_clusters_producing".to_string()),
    }
}
