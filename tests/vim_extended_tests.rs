//! Exhaustive tests for the extended core-vim coverage: operator+motion
//! ranges, new motions, edits, text objects, visual-mode features,
//! registers, jumplist, and command-line substitution.
//!
//! Everything runs through `InputState::handle_key` — the same execution
//! path the game uses — against a real Map/World/UndoStack/Inventory.

#![allow(non_snake_case)]

use hecs::World;
use vimforge::commands::Command;
use vimforge::game::inventory::Inventory;
use vimforge::game::session::parse_key_notation;
use vimforge::game::undo::UndoStack;
use vimforge::input::handler::InputState;
use vimforge::map::grid::Map;
use vimforge::resources::{EntityType, Facing};
use vimforge::vim::parser::Mode;

/// Test rig: a full input pipeline over a real map.
struct Rig {
    input: InputState,
    map: Map,
    world: World,
    undo: UndoStack,
    inv: Inventory,
}

impl Rig {
    fn new(width: usize, height: usize) -> Self {
        Rig {
            input: InputState::new(),
            map: Map::new(width, height),
            world: World::new(),
            undo: UndoStack::new(),
            inv: Inventory::new(),
        }
    }

    fn place(&mut self, x: usize, y: usize, et: EntityType, facing: Facing) {
        self.map
            .place_entity_on_map(&mut self.world, x, y, et, facing, true)
            .expect("placement should succeed");
    }

    /// Feed vim-notation keys, returning every executed command.
    fn feed(&mut self, keys: &str) -> Vec<Command> {
        let mut out = Vec::new();
        for key in parse_key_notation(keys) {
            out.extend(self.input.handle_key(
                key,
                &mut self.map,
                &mut self.world,
                &mut self.undo,
                &mut self.inv,
            ));
        }
        out
    }

    fn et(&self, x: usize, y: usize) -> Option<EntityType> {
        self.map.entity_type_at(&self.world, x, y)
    }

    fn facing(&self, x: usize, y: usize) -> Option<Facing> {
        self.map.entity_facing_at(&self.world, x, y)
    }

    fn cursor(&self) -> (usize, usize) {
        (self.input.cursor_x, self.input.cursor_y)
    }

    /// Count all entities on the map (anchor tiles only).
    fn entity_count(&self) -> usize {
        let mut n = 0;
        for (e, _pos) in self
            .world
            .query::<&vimforge::ecs::components::Position>()
            .iter()
        {
            if self
                .world
                .get::<&vimforge::ecs::components::PartOfBuilding>(e)
                .is_err()
                && self
                    .world
                    .get::<&vimforge::ecs::components::EntityKind>(e)
                    .is_ok()
            {
                n += 1;
            }
        }
        n
    }
}

/// Belts at x0..=x1 on row y.
fn belt_row(rig: &mut Rig, x0: usize, x1: usize, y: usize) {
    for x in x0..=x1 {
        rig.place(x, y, EntityType::BasicBelt, Facing::Right);
    }
}

// ===========================================================================
// A. Operator + motion ranges
// ===========================================================================

#[test]
fn test_dw_deletes_word_and_gap() {
    let mut rig = Rig::new(20, 5);
    rig.place(0, 0, EntityType::Wall, Facing::Right); // "word" 1
    rig.place(5, 0, EntityType::Wall, Facing::Right); // next "word"
    rig.feed("dw");
    // w is exclusive: everything from cursor up to (not including) the
    // next entity is deleted.
    for x in 0..=4 {
        assert_eq!(rig.et(x, 0), None, "tile {x} should be deleted");
    }
    assert_eq!(rig.et(5, 0), Some(EntityType::Wall));
    assert_eq!(rig.cursor(), (0, 0), "delete leaves cursor at range start");
}

#[test]
fn test_d3l_deletes_three_tiles() {
    let mut rig = Rig::new(20, 5);
    belt_row(&mut rig, 0, 5, 0);
    rig.feed("d3l");
    assert_eq!(rig.et(0, 0), None);
    assert_eq!(rig.et(1, 0), None);
    assert_eq!(rig.et(2, 0), None);
    assert_eq!(rig.et(3, 0), Some(EntityType::BasicBelt), "l is exclusive");
}

#[test]
fn test_d2w_and_2dw_counts() {
    // d2w
    let mut rig = Rig::new(30, 5);
    rig.place(3, 0, EntityType::Smelter, Facing::Right);
    rig.place(6, 0, EntityType::Kiln, Facing::Right);
    rig.place(9, 0, EntityType::Press, Facing::Right);
    rig.feed("d2w");
    assert_eq!(rig.et(3, 0), None);
    assert_eq!(rig.et(6, 0), Some(EntityType::Kiln), "2w lands on kiln (exclusive)");
    assert_eq!(rig.et(9, 0), Some(EntityType::Press));

    // 2dw behaves the same (counts multiply)
    let mut rig2 = Rig::new(30, 5);
    rig2.place(3, 0, EntityType::Smelter, Facing::Right);
    rig2.place(6, 0, EntityType::Kiln, Facing::Right);
    rig2.place(9, 0, EntityType::Press, Facing::Right);
    rig2.feed("2dw");
    assert_eq!(rig2.et(3, 0), None);
    assert_eq!(rig2.et(6, 0), Some(EntityType::Kiln));
}

#[test]
fn test_dfs_deletes_through_smelter_inclusive() {
    let mut rig = Rig::new(20, 5);
    belt_row(&mut rig, 0, 2, 0);
    // Smelter is 3x3: anchored at (8,0), spans (8..=10, 0..=2)
    rig.place(8, 0, EntityType::Smelter, Facing::Right);
    rig.place(14, 0, EntityType::Wall, Facing::Right);
    rig.feed("dfs");
    for x in 0..=8 {
        assert_eq!(rig.et(x, 0), None, "f is inclusive — {x} deleted");
    }
    // Deleting the anchor tile removes the whole 3x3 smelter.
    assert_eq!(rig.et(9, 1), None);
    assert_eq!(rig.et(14, 0), Some(EntityType::Wall));
}

#[test]
fn test_dts_stops_before_smelter() {
    let mut rig = Rig::new(20, 5);
    belt_row(&mut rig, 0, 2, 0);
    rig.place(4, 0, EntityType::Smelter, Facing::Right);
    rig.feed("dts");
    assert_eq!(rig.et(3, 0), None);
    assert_eq!(rig.et(4, 0), Some(EntityType::Smelter), "t stops short");
}

#[test]
fn test_df_not_found_is_noop() {
    let mut rig = Rig::new(10, 3);
    belt_row(&mut rig, 0, 2, 0);
    rig.feed("dfs"); // no smelter anywhere — motion fails, operator aborts
    assert_eq!(rig.et(0, 0), Some(EntityType::BasicBelt));
    assert_eq!(rig.et(2, 0), Some(EntityType::BasicBelt));
}

#[test]
fn test_d_dollar_deletes_to_line_end() {
    let mut rig = Rig::new(10, 3);
    belt_row(&mut rig, 0, 9, 0);
    rig.feed("3l"); // cursor to x=3
    rig.feed("d$");
    for x in 0..=2 {
        assert_eq!(rig.et(x, 0), Some(EntityType::BasicBelt));
    }
    for x in 3..10 {
        assert_eq!(rig.et(x, 0), None, "$ inclusive to end of row");
    }
}

#[test]
fn test_dG_linewise_to_bottom() {
    let mut rig = Rig::new(10, 5);
    for y in 0..5 {
        rig.place(2, y, EntityType::BasicBelt, Facing::Right);
    }
    rig.feed("j"); // row 1
    rig.feed("dG");
    assert_eq!(rig.et(2, 0), Some(EntityType::BasicBelt), "row 0 kept");
    for y in 1..5 {
        assert_eq!(rig.et(2, y), None, "rows 1..5 deleted linewise");
    }
}

#[test]
fn test_dgg_linewise_to_top() {
    let mut rig = Rig::new(10, 5);
    for y in 0..5 {
        rig.place(2, y, EntityType::BasicBelt, Facing::Right);
    }
    rig.feed("2j"); // row 2
    rig.feed("dgg");
    for y in 0..=2 {
        assert_eq!(rig.et(2, y), None);
    }
    assert_eq!(rig.et(2, 3), Some(EntityType::BasicBelt));
}

#[test]
fn test_dj_linewise_two_rows() {
    let mut rig = Rig::new(10, 5);
    for y in 0..4 {
        rig.place(0, y, EntityType::BasicBelt, Facing::Right);
    }
    rig.feed("dj");
    assert_eq!(rig.et(0, 0), None);
    assert_eq!(rig.et(0, 1), None);
    assert_eq!(rig.et(0, 2), Some(EntityType::BasicBelt));
}

#[test]
fn test_db_backward_exclusive() {
    let mut rig = Rig::new(20, 3);
    rig.place(2, 0, EntityType::Wall, Facing::Right);
    rig.place(6, 0, EntityType::Turret, Facing::Right);
    rig.feed("6l"); // cursor on the turret
    rig.feed("db");
    assert_eq!(rig.et(2, 0), None, "b target deleted");
    assert_eq!(
        rig.et(6, 0),
        Some(EntityType::Turret),
        "cursor tile survives a backward exclusive delete"
    );
    assert_eq!(rig.cursor(), (2, 0));
}

#[test]
fn test_cw_enters_insert_mode() {
    let mut rig = Rig::new(20, 3);
    belt_row(&mut rig, 0, 2, 0);
    rig.place(5, 0, EntityType::Smelter, Facing::Right);
    rig.feed("cw");
    assert_eq!(rig.et(0, 0), None);
    assert_eq!(rig.input.parser.mode, Mode::Insert, "change enters insert");
    rig.feed("<Esc>");
    assert_eq!(rig.input.parser.mode, Mode::Normal);
}

#[test]
fn test_c_dollar_change_to_line_end() {
    let mut rig = Rig::new(10, 3);
    belt_row(&mut rig, 0, 9, 0);
    rig.feed("5l");
    rig.feed("c$");
    assert_eq!(rig.et(4, 0), Some(EntityType::BasicBelt));
    assert_eq!(rig.et(5, 0), None);
    assert_eq!(rig.et(9, 0), None);
    assert_eq!(rig.input.parser.mode, Mode::Insert);
}

#[test]
fn test_yw_yanks_without_deleting() {
    let mut rig = Rig::new(20, 6);
    belt_row(&mut rig, 0, 2, 0);
    rig.feed("yw");
    assert_eq!(rig.et(0, 0), Some(EntityType::BasicBelt), "yank keeps tiles");
    assert_eq!(rig.cursor(), (0, 0), "yank leaves cursor in place");
    // Paste it somewhere empty
    rig.feed("3jp");
    assert_eq!(rig.et(0, 3), Some(EntityType::BasicBelt));
}

#[test]
fn test_register_dw_and_paste() {
    let mut rig = Rig::new(20, 5);
    rig.place(0, 0, EntityType::Wall, Facing::Right);
    rig.place(4, 0, EntityType::Wall, Facing::Right);
    rig.feed("\"adw"); // deletes the first wall (and the gap) into "a
    assert_eq!(rig.et(0, 0), None);
    // Paste from register a at row 2
    rig.feed("2j\"ap");
    assert_eq!(rig.et(0, 2), Some(EntityType::Wall));
}

#[test]
fn test_2dd_deletes_two_rows() {
    let mut rig = Rig::new(10, 5);
    for y in 0..3 {
        rig.place(1, y, EntityType::BasicBelt, Facing::Right);
    }
    rig.feed("2dd");
    assert_eq!(rig.et(1, 0), None);
    assert_eq!(rig.et(1, 1), None);
    assert_eq!(rig.et(1, 2), Some(EntityType::BasicBelt));
}

#[test]
fn test_dot_repeat_after_dw() {
    let mut rig = Rig::new(30, 3);
    rig.place(0, 0, EntityType::Smelter, Facing::Right);
    rig.place(4, 0, EntityType::Kiln, Facing::Right);
    rig.place(8, 0, EntityType::Press, Facing::Right);
    rig.feed("dw"); // deletes smelter (up to kiln)
    assert_eq!(rig.et(0, 0), None);
    assert_eq!(rig.et(4, 0), Some(EntityType::Kiln));
    rig.feed("4l"); // move onto the kiln
    rig.feed("."); // repeat dw
    assert_eq!(rig.et(4, 0), None, "dot repeats the dw");
    assert_eq!(rig.et(8, 0), Some(EntityType::Press));
}

#[test]
fn test_operator_motion_undoable() {
    let mut rig = Rig::new(10, 3);
    belt_row(&mut rig, 0, 4, 0);
    rig.feed("d$");
    assert_eq!(rig.et(2, 0), None);
    rig.feed("u");
    assert_eq!(rig.et(2, 0), Some(EntityType::BasicBelt), "dw is undoable");
}

// ===========================================================================
// B. New motions
// ===========================================================================

#[test]
fn test_e_with_count() {
    let mut rig = Rig::new(30, 3);
    belt_row(&mut rig, 0, 2, 0);
    belt_row(&mut rig, 5, 7, 0);
    rig.feed("e");
    assert_eq!(rig.cursor(), (2, 0), "e goes to end of first cluster");
    rig.feed("gg");
    rig.feed("2e");
    assert_eq!(rig.cursor(), (7, 0), "2e goes to end of second cluster");
}

#[test]
fn test_ge_back_to_end_of_previous_cluster() {
    let mut rig = Rig::new(30, 3);
    belt_row(&mut rig, 0, 2, 0);
    belt_row(&mut rig, 6, 8, 0);
    rig.feed("7l"); // inside second cluster
    rig.feed("ge");
    assert_eq!(rig.cursor(), (2, 0), "ge lands on end of previous cluster");
}

#[test]
fn test_E_big_word_end() {
    let mut rig = Rig::new(30, 3);
    belt_row(&mut rig, 0, 3, 0);
    rig.feed("E");
    assert_eq!(rig.cursor(), (3, 0));
}

#[test]
fn test_ctrl_d_u_f_b_scrolling() {
    let mut rig = Rig::new(10, 100);
    // Default InputState viewport_height is 24 → half page is 12.
    rig.feed("<C-d>");
    assert_eq!(rig.cursor().1, 12, "Ctrl-d moves half a viewport down");
    rig.feed("<C-u>");
    assert_eq!(rig.cursor().1, 0, "Ctrl-u moves back up");
    rig.feed("<C-f>");
    assert_eq!(rig.cursor().1, 24, "Ctrl-f moves a full page down");
    rig.feed("<C-b>");
    assert_eq!(rig.cursor().1, 0, "Ctrl-b moves a full page up");
}

#[test]
fn test_ctrl_b_no_longer_opens_contracts() {
    let mut rig = Rig::new(10, 100);
    let cmds = rig.feed("<C-b>");
    assert!(
        !cmds.iter().any(|c| matches!(c, Command::CmdContracts)),
        "Ctrl-b must scroll, not open the contract board"
    );
    assert!(cmds
        .iter()
        .any(|c| matches!(c, Command::ScrollFullPageUp(_))));
}

#[test]
fn test_zz_zt_zb_viewport_positioning() {
    let mut rig = Rig::new(10, 200);
    rig.feed("100G"); // row 100
    assert_eq!(rig.cursor(), (0, 100));
    rig.feed("zz");
    assert_eq!(
        rig.input.viewport_top, 88,
        "zz centers: 100 - 24/2 = 88 (mirrored viewport)"
    );
    rig.feed("zt");
    assert_eq!(rig.input.viewport_top, 100, "zt puts cursor row at top");
    rig.feed("zb");
    assert_eq!(rig.input.viewport_top, 77, "zb: 100+1-24 = 77");
}

#[test]
fn test_paren_sentence_motions_jump_between_machines() {
    let mut rig = Rig::new(30, 5);
    belt_row(&mut rig, 0, 8, 0); // belts are skipped
    rig.place(10, 0, EntityType::Smelter, Facing::Right);
    rig.place(3, 2, EntityType::Kiln, Facing::Right);
    rig.feed(")");
    assert_eq!(rig.cursor(), (10, 0), ") jumps to the first machine");
    rig.feed(")");
    assert_eq!(rig.cursor(), (3, 2), ") jumps to the next machine");
    rig.feed("(");
    assert_eq!(rig.cursor(), (10, 0), "( jumps back");
}

#[test]
fn test_pipe_underscore_plus_minus() {
    let mut rig = Rig::new(30, 5);
    rig.place(4, 0, EntityType::Wall, Facing::Right);
    rig.place(7, 1, EntityType::Turret, Facing::Right);
    rig.feed("9|");
    assert_eq!(rig.cursor(), (8, 0), "9| goes to column 9 (1-indexed)");
    rig.feed("_");
    assert_eq!(rig.cursor(), (4, 0), "_ goes to first entity in row");
    rig.feed("+");
    assert_eq!(rig.cursor(), (7, 1), "+ first entity of row below");
    rig.feed("-");
    assert_eq!(rig.cursor(), (4, 0), "- first entity of row above");
}

#[test]
fn test_g_underscore_last_entity() {
    let mut rig = Rig::new(30, 3);
    rig.place(3, 0, EntityType::Wall, Facing::Right);
    rig.place(12, 0, EntityType::Turret, Facing::Right);
    rig.feed("g_");
    assert_eq!(rig.cursor(), (12, 0));
}

#[test]
fn test_jumplist_ctrl_o_ctrl_i() {
    let mut rig = Rig::new(10, 100);
    rig.feed("G"); // jump 1: 0 -> 99
    assert_eq!(rig.cursor(), (0, 99));
    rig.feed("gg"); // jump 2: 99 -> 0
    assert_eq!(rig.cursor(), (0, 0));
    rig.feed("<C-o>");
    assert_eq!(rig.cursor(), (0, 99), "Ctrl-o goes back one jump");
    rig.feed("<C-o>");
    assert_eq!(rig.cursor(), (0, 0), "Ctrl-o goes back again");
    rig.feed("<C-i>");
    assert_eq!(rig.cursor(), (0, 99), "Ctrl-i goes forward");
}

// ===========================================================================
// C. New edits
// ===========================================================================

#[test]
fn test_s_substitute_deletes_and_inserts() {
    let mut rig = Rig::new(10, 3);
    belt_row(&mut rig, 0, 3, 0);
    rig.feed("s");
    assert_eq!(rig.et(0, 0), None, "s deletes the tile under the cursor");
    assert_eq!(rig.input.parser.mode, Mode::Insert, "s enters insert");
    rig.feed("<Esc>");
}

#[test]
fn test_3s_deletes_three_tiles() {
    let mut rig = Rig::new(10, 3);
    belt_row(&mut rig, 0, 4, 0);
    rig.feed("3s");
    assert_eq!(rig.et(0, 0), None);
    assert_eq!(rig.et(1, 0), None);
    assert_eq!(rig.et(2, 0), None);
    assert_eq!(rig.et(3, 0), Some(EntityType::BasicBelt));
    assert_eq!(rig.input.parser.mode, Mode::Insert);
    rig.feed("<Esc>");
}

#[test]
fn test_S_changes_whole_line() {
    let mut rig = Rig::new(10, 3);
    belt_row(&mut rig, 0, 9, 0);
    rig.place(0, 1, EntityType::Wall, Facing::Right);
    rig.feed("S");
    for x in 0..10 {
        assert_eq!(rig.et(x, 0), None);
    }
    assert_eq!(rig.et(0, 1), Some(EntityType::Wall), "row 1 untouched");
    assert_eq!(rig.input.parser.mode, Mode::Insert);
    rig.feed("<Esc>");
}

#[test]
fn test_C_and_D_to_line_end() {
    let mut rig = Rig::new(10, 3);
    belt_row(&mut rig, 0, 9, 0);
    rig.feed("4l");
    rig.feed("D");
    assert_eq!(rig.et(3, 0), Some(EntityType::BasicBelt));
    assert_eq!(rig.et(4, 0), None);
    assert_eq!(rig.et(9, 0), None);
    assert_eq!(rig.input.parser.mode, Mode::Normal, "D stays in normal");

    let mut rig2 = Rig::new(10, 3);
    belt_row(&mut rig2, 0, 9, 0);
    rig2.feed("4l");
    rig2.feed("C");
    assert_eq!(rig2.et(4, 0), None);
    assert_eq!(rig2.input.parser.mode, Mode::Insert, "C enters insert");
    rig2.feed("<Esc>");
}

#[test]
fn test_Y_yanks_line() {
    let mut rig = Rig::new(10, 4);
    belt_row(&mut rig, 0, 2, 0);
    rig.feed("Y");
    assert_eq!(rig.et(0, 0), Some(EntityType::BasicBelt), "Y does not delete");
    rig.feed("2jp");
    assert!(
        rig.et(0, 2).is_some() || rig.et(0, 3).is_some(),
        "pasting the yanked line places belts"
    );
}

#[test]
fn test_R_replace_mode() {
    let mut rig = Rig::new(10, 3);
    belt_row(&mut rig, 0, 4, 0);
    rig.feed("R");
    assert_eq!(rig.input.mode_string(), "REPLACE");
    rig.feed("w"); // insert-mode quick key 'w' = Wall (1x1)
    assert_eq!(
        rig.et(0, 0),
        Some(EntityType::Wall),
        "R replaces the tile under the cursor"
    );
    assert_eq!(rig.cursor(), (1, 0), "R advances the cursor");
    rig.feed("w");
    assert_eq!(rig.et(1, 0), Some(EntityType::Wall));
    rig.feed("<Esc>");
    assert_eq!(rig.input.parser.mode, Mode::Normal);
    assert_eq!(rig.input.mode_string(), "NORMAL");
    assert_eq!(rig.et(2, 0), Some(EntityType::BasicBelt), "rest untouched");
}

#[test]
fn test_J_joins_clusters_with_belts() {
    let mut rig = Rig::new(20, 3);
    rig.place(2, 0, EntityType::Wall, Facing::Right);
    rig.place(3, 0, EntityType::Wall, Facing::Right);
    rig.place(8, 0, EntityType::Turret, Facing::Right);
    rig.feed("2l"); // onto the first cluster
    rig.feed("J");
    for x in 4..8 {
        assert_eq!(
            rig.et(x, 0),
            Some(EntityType::BasicBelt),
            "gap tile {x} filled with a belt"
        );
        assert_eq!(rig.facing(x, 0), Some(Facing::Right), "belts face right");
    }
    assert_eq!(rig.et(8, 0), Some(EntityType::Turret));
}

#[test]
fn test_J_no_gap_is_noop() {
    let mut rig = Rig::new(20, 3);
    belt_row(&mut rig, 0, 19, 0); // one solid row — nothing to join
    let before = rig.entity_count();
    rig.feed("J");
    assert_eq!(rig.entity_count(), before);
}

#[test]
fn test_ctrl_a_upgrades_belt_tier() {
    let mut rig = Rig::new(10, 3);
    rig.place(0, 0, EntityType::BasicBelt, Facing::Down);
    rig.feed("<C-a>");
    assert_eq!(rig.et(0, 0), Some(EntityType::FastBelt));
    assert_eq!(rig.facing(0, 0), Some(Facing::Down), "facing preserved");
    rig.feed("<C-a>");
    assert_eq!(rig.et(0, 0), Some(EntityType::ExpressBelt));
    rig.feed("<C-a>"); // top of ladder — no-op
    assert_eq!(rig.et(0, 0), Some(EntityType::ExpressBelt));
    rig.feed("2<C-x>");
    assert_eq!(rig.et(0, 0), Some(EntityType::BasicBelt), "count works down");
    // Undoable
    rig.feed("u");
    assert_eq!(rig.et(0, 0), Some(EntityType::ExpressBelt));
}

#[test]
fn test_ctrl_a_on_machine_without_ladder_is_noop() {
    let mut rig = Rig::new(10, 3);
    rig.place(0, 0, EntityType::Smelter, Facing::Right);
    rig.feed("<C-a>");
    assert_eq!(rig.et(0, 0), Some(EntityType::Smelter));
}

#[test]
fn test_gU_iw_upgrades_cluster() {
    let mut rig = Rig::new(10, 3);
    belt_row(&mut rig, 0, 2, 0);
    rig.feed("l"); // inside cluster
    rig.feed("gUiw");
    for x in 0..=2 {
        assert_eq!(rig.et(x, 0), Some(EntityType::FastBelt), "tile {x} upgraded");
    }
}

#[test]
fn test_gu_motion_downgrades() {
    let mut rig = Rig::new(10, 3);
    for x in 0..3 {
        rig.place(x, 0, EntityType::ExpressBelt, Facing::Right);
    }
    rig.feed("gu$");
    for x in 0..3 {
        assert_eq!(rig.et(x, 0), Some(EntityType::FastBelt));
    }
}

#[test]
fn test_gUU_and_guu_linewise() {
    let mut rig = Rig::new(10, 3);
    belt_row(&mut rig, 0, 4, 0);
    rig.place(0, 1, EntityType::BasicBelt, Facing::Right);
    rig.feed("gUU");
    for x in 0..=4 {
        assert_eq!(rig.et(x, 0), Some(EntityType::FastBelt));
    }
    assert_eq!(rig.et(0, 1), Some(EntityType::BasicBelt), "row 1 untouched");
    rig.feed("guu");
    for x in 0..=4 {
        assert_eq!(rig.et(x, 0), Some(EntityType::BasicBelt));
    }
}

#[test]
fn test_g_tilde_rotates_180() {
    let mut rig = Rig::new(10, 3);
    rig.place(0, 0, EntityType::BasicBelt, Facing::Right);
    rig.place(1, 0, EntityType::BasicBelt, Facing::Up);
    rig.feed("g~$");
    assert_eq!(rig.facing(0, 0), Some(Facing::Left));
    assert_eq!(rig.facing(1, 0), Some(Facing::Down));
}

#[test]
fn test_gi_returns_to_last_insert_position() {
    let mut rig = Rig::new(20, 5);
    rig.feed("3l2j"); // (3,2)
    rig.feed("ic<Esc>"); // place a belt at (3,2), cursor advanced to (4,2)
    rig.feed("gg"); // away to (0,0)
    rig.feed("gi");
    assert_eq!(rig.input.parser.mode, Mode::Insert, "gi re-enters insert");
    assert_eq!(rig.cursor().1, 2, "gi returns to the insert row");
    rig.feed("<Esc>");
}

// ===========================================================================
// D. Text objects
// ===========================================================================

/// Build a wall ring around (2,2)..(4,4) with a machine inside.
fn walled_rig() -> Rig {
    let mut rig = Rig::new(10, 10);
    for x in 1..=5 {
        rig.place(x, 1, EntityType::Wall, Facing::Right);
        rig.place(x, 5, EntityType::Wall, Facing::Right);
    }
    for y in 2..=4 {
        rig.place(1, y, EntityType::Wall, Facing::Right);
        rig.place(5, y, EntityType::Wall, Facing::Right);
    }
    rig.place(3, 3, EntityType::Turret, Facing::Right);
    rig
}

#[test]
fn test_all_bracket_text_objects_alias_block() {
    for obj in ["di[", "di{", "di<", "diB", "dis"] {
        let mut rig = walled_rig();
        rig.feed("3l3j"); // (3,3) inside the enclosure
        rig.feed(obj);
        assert_eq!(rig.et(3, 3), None, "{obj} deletes the enclosed machine");
        assert_eq!(
            rig.et(1, 1),
            Some(EntityType::Wall),
            "{obj} keeps the walls"
        );
    }
}

#[test]
fn test_da_bracket_includes_walls() {
    for obj in ["da[", "da{", "da<", "daB", "das"] {
        let mut rig = walled_rig();
        rig.feed("3l3j");
        rig.feed(obj);
        assert_eq!(rig.et(3, 3), None, "{obj} deletes the inside");
        assert_eq!(rig.et(1, 1), None, "{obj} deletes the walls too");
    }
}

#[test]
fn test_di_quote_deletes_belt_run() {
    let mut rig = Rig::new(20, 3);
    rig.place(1, 0, EntityType::Wall, Facing::Right);
    belt_row(&mut rig, 2, 6, 0);
    rig.place(7, 0, EntityType::Turret, Facing::Right);
    rig.feed("4l"); // on a belt in the run
    rig.feed("di\"");
    for x in 2..=6 {
        assert_eq!(rig.et(x, 0), None, "belt {x} deleted");
    }
    assert_eq!(rig.et(1, 0), Some(EntityType::Wall), "i\" keeps machines");
    assert_eq!(rig.et(7, 0), Some(EntityType::Turret));
}

#[test]
fn test_da_quote_includes_end_machines() {
    let mut rig = Rig::new(20, 3);
    rig.place(1, 0, EntityType::Wall, Facing::Right);
    belt_row(&mut rig, 2, 6, 0);
    rig.place(7, 0, EntityType::Turret, Facing::Right);
    rig.feed("4l");
    rig.feed("da'");
    assert_eq!(rig.et(1, 0), None, "a' takes the machine at the start");
    assert_eq!(rig.et(7, 0), None, "a' takes the machine at the end");
    assert_eq!(rig.et(4, 0), None);
}

#[test]
fn test_belt_run_only_same_direction() {
    let mut rig = Rig::new(20, 3);
    belt_row(&mut rig, 0, 3, 0); // facing Right
    rig.place(4, 0, EntityType::BasicBelt, Facing::Down); // different direction
    rig.feed("di\"");
    for x in 0..=3 {
        assert_eq!(rig.et(x, 0), None);
    }
    assert_eq!(
        rig.et(4, 0),
        Some(EntityType::BasicBelt),
        "differently-facing belt is not part of the run"
    );
}

#[test]
fn test_dit_deletes_machine_footprint() {
    let mut rig = Rig::new(20, 10);
    // Assembler at Facing::Right is 3 wide x 4 tall: (5..=7, 2..=5).
    // Its output port is at (8,4).
    rig.place(5, 2, EntityType::Assembler, Facing::Right);
    belt_row(&mut rig, 8, 10, 4); // output belts, must survive `it`
    rig.feed("6l3j"); // (6,3): a secondary tile of the machine
    rig.feed("dit");
    assert_eq!(rig.et(5, 2), None, "anchor gone");
    assert_eq!(rig.et(6, 3), None, "footprint gone");
    assert_eq!(rig.et(7, 5), None, "footprint gone");
    assert_eq!(rig.et(8, 4), Some(EntityType::BasicBelt), "it keeps ports");
}

#[test]
fn test_dat_includes_port_tiles() {
    let mut rig = Rig::new(20, 10);
    rig.place(5, 2, EntityType::Assembler, Facing::Right);
    belt_row(&mut rig, 8, 10, 4); // belt at (8,4) sits on the output port
    rig.feed("6l3j");
    rig.feed("dat");
    assert_eq!(rig.et(6, 3), None, "machine gone");
    assert_eq!(rig.et(8, 4), None, "at takes the port tile too");
    assert_eq!(rig.et(9, 4), Some(EntityType::BasicBelt), "beyond the port survives");
}

#[test]
fn test_yit_yanks_machine() {
    let mut rig = Rig::new(20, 10);
    rig.place(5, 2, EntityType::Assembler, Facing::Right);
    rig.feed("5l2j");
    rig.feed("yit");
    assert_eq!(rig.et(5, 2), Some(EntityType::Assembler), "yank keeps it");
    // Paste elsewhere (assembler spans (12..=14, 0..=3) — free space)
    rig.feed("gg12lp");
    assert_eq!(rig.et(12, 0), Some(EntityType::Assembler));
}

// ===========================================================================
// E. Visual mode richness
// ===========================================================================

#[test]
fn test_visual_count_motion() {
    let mut rig = Rig::new(20, 3);
    belt_row(&mut rig, 0, 9, 0);
    rig.feed("v5l");
    assert_eq!(rig.cursor(), (5, 0), "5l extends the selection by 5");
    rig.feed("d");
    for x in 0..=5 {
        assert_eq!(rig.et(x, 0), None);
    }
    assert_eq!(rig.et(6, 0), Some(EntityType::BasicBelt));
}

#[test]
fn test_visual_f_motion() {
    let mut rig = Rig::new(20, 3);
    belt_row(&mut rig, 0, 2, 0);
    rig.place(5, 0, EntityType::Smelter, Facing::Right);
    rig.feed("vfs");
    assert_eq!(rig.cursor(), (5, 0), "f extends selection to the smelter");
    rig.feed("d");
    assert_eq!(rig.et(5, 0), None);
    assert_eq!(rig.et(0, 0), None);
}

#[test]
fn test_visual_x_deletes_selection() {
    let mut rig = Rig::new(20, 3);
    belt_row(&mut rig, 0, 5, 0);
    rig.feed("v2lx");
    assert_eq!(rig.et(0, 0), None);
    assert_eq!(rig.et(2, 0), None);
    assert_eq!(rig.et(3, 0), Some(EntityType::BasicBelt));
    assert_eq!(rig.input.parser.mode, Mode::Normal);
}

#[test]
fn test_visual_r_replaces_selection() {
    let mut rig = Rig::new(20, 3);
    belt_row(&mut rig, 0, 4, 0);
    rig.feed("v2lrw"); // replace 3 selected tiles with walls (1x1)
    for x in 0..=2 {
        assert_eq!(rig.et(x, 0), Some(EntityType::Wall), "tile {x} replaced");
    }
    assert_eq!(rig.et(3, 0), Some(EntityType::BasicBelt));
    assert_eq!(rig.input.parser.mode, Mode::Normal);
}

#[test]
fn test_visual_tilde_rotates_selection() {
    let mut rig = Rig::new(20, 3);
    belt_row(&mut rig, 0, 2, 0);
    rig.feed("v2l~");
    for x in 0..=2 {
        assert_eq!(rig.facing(x, 0), Some(Facing::Left), "tile {x} rotated 180");
    }
}

#[test]
fn test_visual_text_object_viw() {
    let mut rig = Rig::new(20, 3);
    belt_row(&mut rig, 2, 5, 0);
    rig.feed("3l"); // inside the cluster
    rig.feed("viwd");
    for x in 2..=5 {
        assert_eq!(rig.et(x, 0), None, "viw selected the whole cluster");
    }
}

#[test]
fn test_gv_reselects_last_visual() {
    let mut rig = Rig::new(20, 3);
    belt_row(&mut rig, 0, 8, 0);
    rig.feed("v3l<Esc>"); // select (0,0)..(3,0), abandon
    assert_eq!(rig.input.parser.mode, Mode::Normal);
    rig.feed("8l"); // move away
    rig.feed("gv");
    assert_eq!(rig.input.parser.mode, Mode::Visual, "gv re-enters visual");
    assert_eq!(rig.input.visual_anchor, Some((0, 0)), "anchor restored");
    assert_eq!(rig.cursor(), (3, 0), "cursor restored");
    rig.feed("d");
    for x in 0..=3 {
        assert_eq!(rig.et(x, 0), None);
    }
    assert_eq!(rig.et(4, 0), Some(EntityType::BasicBelt));
}

#[test]
fn test_visual_block_I_column_insert() {
    let mut rig = Rig::new(20, 10);
    rig.feed("2l2j"); // (2,2)
    rig.feed("<C-v>3j2l"); // block (2,2)..(4,5)
    rig.feed("Ic"); // insert belts down the LEFT edge
    assert_eq!(rig.input.parser.mode, Mode::Normal);
    for y in 2..=5 {
        assert_eq!(
            rig.et(2, y),
            Some(EntityType::BasicBelt),
            "left edge row {y} filled"
        );
        assert_eq!(rig.et(3, y), None, "middle column untouched");
    }
}

#[test]
fn test_visual_block_A_column_append() {
    let mut rig = Rig::new(20, 10);
    rig.feed("2l2j");
    rig.feed("<C-v>2j2l");
    rig.feed("Aw"); // walls down the RIGHT edge
    for y in 2..=4 {
        assert_eq!(rig.et(4, y), Some(EntityType::Wall));
        assert_eq!(rig.et(2, y), None);
    }
}

#[test]
fn test_visual_block_O_swaps_corner() {
    let mut rig = Rig::new(20, 10);
    rig.feed("<C-v>3l2j"); // anchor (0,0), cursor (3,2)
    rig.feed("O");
    assert_eq!(rig.input.visual_anchor, Some((3, 0)), "corner swapped");
    assert_eq!(rig.cursor(), (0, 2));
    rig.feed("<Esc>");
}

// ===========================================================================
// F. Registers and jumplist
// ===========================================================================

#[test]
fn test_uppercase_register_appends() {
    let mut rig = Rig::new(20, 10);
    rig.place(0, 0, EntityType::Wall, Facing::Right);
    rig.place(0, 1, EntityType::Turret, Facing::Right);
    rig.feed("\"Ayy"); // append wall row into "a
    rig.feed("j\"Ayy"); // append turret row
    rig.feed("5j\"ap"); // paste combined blueprint at row 6
    assert_eq!(rig.et(0, 6), Some(EntityType::Wall));
    assert_eq!(rig.et(0, 7), Some(EntityType::Turret), "appended below");
}

#[test]
fn test_black_hole_register_preserves_unnamed() {
    let mut rig = Rig::new(20, 10);
    rig.place(0, 0, EntityType::Wall, Facing::Right);
    rig.place(0, 2, EntityType::Turret, Facing::Right);
    rig.feed("yy"); // unnamed = wall row
    rig.feed("2j\"_dd"); // delete turret row into the black hole
    assert_eq!(rig.et(0, 2), None);
    rig.feed("2jp"); // paste unnamed at row 4
    assert_eq!(
        rig.et(0, 4),
        Some(EntityType::Wall),
        "unnamed register survived the black-hole delete"
    );
}

#[test]
fn test_numbered_registers_shift() {
    let mut rig = Rig::new(20, 10);
    rig.place(0, 0, EntityType::Wall, Facing::Right);
    rig.place(0, 1, EntityType::Turret, Facing::Right);
    rig.feed("dd"); // "1 = wall
    rig.feed("jdd"); // "1 = turret, "2 = wall
    rig.feed("3j\"1p");
    assert_eq!(rig.et(0, 4), Some(EntityType::Turret), "\"1 is the last delete");
    rig.feed("2j\"2p");
    assert_eq!(
        rig.et(0, 6),
        Some(EntityType::Wall),
        "\"2 is the previous delete"
    );
}

#[test]
fn test_yank_register_0_still_works() {
    let mut rig = Rig::new(20, 10);
    rig.place(0, 0, EntityType::Wall, Facing::Right);
    rig.feed("yy"); // "0 = wall row
    rig.feed("dd"); // unnamed = delete
    rig.feed("3j\"0p");
    assert_eq!(rig.et(0, 3), Some(EntityType::Wall), "\"0 = last yank");
}

// ===========================================================================
// G. Command-line: :s, :g, &, @:
// ===========================================================================

#[test]
fn test_percent_substitute_whole_map() {
    let mut rig = Rig::new(20, 5);
    rig.place(0, 0, EntityType::BasicBelt, Facing::Down);
    rig.place(3, 2, EntityType::BasicBelt, Facing::Right);
    rig.place(5, 4, EntityType::Turret, Facing::Right);
    rig.feed(":%s/belt/fastbelt/g<CR>");
    assert_eq!(rig.et(0, 0), Some(EntityType::FastBelt));
    assert_eq!(rig.facing(0, 0), Some(Facing::Down), "facing preserved");
    assert_eq!(rig.et(3, 2), Some(EntityType::FastBelt));
    assert_eq!(rig.et(5, 4), Some(EntityType::Turret), "non-matching kept");
    assert_eq!(rig.input.status_message, "2 substitutions");
}

#[test]
fn test_substitute_current_row_first_only() {
    let mut rig = Rig::new(20, 5);
    rig.place(2, 0, EntityType::BasicBelt, Facing::Right);
    rig.place(6, 0, EntityType::BasicBelt, Facing::Right);
    rig.place(2, 1, EntityType::BasicBelt, Facing::Right);
    rig.feed(":s/belt/wall/<CR>"); // no /g: first match on cursor row
    assert_eq!(rig.et(2, 0), Some(EntityType::Wall));
    assert_eq!(rig.et(6, 0), Some(EntityType::BasicBelt), "no /g: only first");
    assert_eq!(rig.et(2, 1), Some(EntityType::BasicBelt), "other rows kept");
}

#[test]
fn test_substitute_row_range() {
    let mut rig = Rig::new(20, 6);
    for y in 0..5 {
        rig.place(0, y, EntityType::BasicBelt, Facing::Right);
    }
    rig.feed(":2,3s/belt/fastbelt/g<CR>"); // 1-indexed rows 2..3 = rows 1..2
    assert_eq!(rig.et(0, 0), Some(EntityType::BasicBelt));
    assert_eq!(rig.et(0, 1), Some(EntityType::FastBelt));
    assert_eq!(rig.et(0, 2), Some(EntityType::FastBelt));
    assert_eq!(rig.et(0, 3), Some(EntityType::BasicBelt));
}

#[test]
fn test_substitute_is_undoable() {
    let mut rig = Rig::new(20, 3);
    rig.place(0, 0, EntityType::BasicBelt, Facing::Right);
    rig.feed(":%s/belt/smelter/g<CR>");
    assert_eq!(rig.et(0, 0), Some(EntityType::Smelter));
    rig.feed("u");
    assert_eq!(rig.et(0, 0), Some(EntityType::BasicBelt));
}

#[test]
fn test_global_delete() {
    let mut rig = Rig::new(20, 5);
    belt_row(&mut rig, 0, 2, 0);
    rig.place(5, 2, EntityType::BasicBelt, Facing::Right);
    rig.place(8, 2, EntityType::Smelter, Facing::Right);
    rig.feed(":g/belt/d<CR>");
    assert_eq!(rig.et(0, 0), None);
    assert_eq!(rig.et(5, 2), None);
    assert_eq!(rig.et(8, 2), Some(EntityType::Smelter));
    assert_eq!(rig.input.status_message, "4 deleted");
    rig.feed("u");
    assert_eq!(rig.et(0, 0), Some(EntityType::BasicBelt), "undoable");
}

#[test]
fn test_ampersand_repeats_substitution_on_current_row() {
    let mut rig = Rig::new(20, 5);
    rig.place(0, 0, EntityType::BasicBelt, Facing::Right);
    rig.place(0, 2, EntityType::BasicBelt, Facing::Right);
    rig.feed(":s/belt/fastbelt/g<CR>"); // converts row 0
    assert_eq!(rig.et(0, 0), Some(EntityType::FastBelt));
    assert_eq!(rig.et(0, 2), Some(EntityType::BasicBelt));
    rig.feed("2j&"); // repeat on row 2
    assert_eq!(rig.et(0, 2), Some(EntityType::FastBelt));
}

#[test]
fn test_at_colon_repeats_last_command() {
    let mut rig = Rig::new(20, 5);
    rig.place(0, 0, EntityType::BasicBelt, Facing::Right);
    rig.place(0, 1, EntityType::BasicBelt, Facing::Right);
    rig.feed(":s/belt/fastbelt/g<CR>");
    assert_eq!(rig.et(0, 0), Some(EntityType::FastBelt));
    rig.feed("j@:"); // repeat :s on row 1
    assert_eq!(rig.et(0, 1), Some(EntityType::FastBelt));
}

#[test]
fn test_split_alias_commands() {
    let mut rig = Rig::new(10, 3);
    let cmds = rig.feed(":vsp<CR>");
    assert!(cmds.iter().any(|c| matches!(c, Command::SplitVertical)));
    let cmds = rig.feed(":sp<CR>");
    assert!(cmds.iter().any(|c| matches!(c, Command::SplitHorizontal)));
    let cmds = rig.feed(":only<CR>");
    assert!(cmds.iter().any(|c| matches!(c, Command::CloseOtherPanes)));
}

// ===========================================================================
// H. Ctrl-w extras
// ===========================================================================

#[test]
fn test_ctrl_w_extras_do_not_crash() {
    let mut rig = Rig::new(10, 3);
    let cmds = rig.feed("<C-w>w");
    assert!(cmds.iter().any(|c| matches!(c, Command::CyclePane)));
    let cmds = rig.feed("<C-w>c");
    assert!(cmds.iter().any(|c| matches!(c, Command::ClosePane)));
    let cmds = rig.feed("<C-w>x");
    assert!(cmds.iter().any(|c| matches!(c, Command::SwapPanes)));
    let cmds = rig.feed("<C-w>r");
    assert!(cmds.iter().any(|c| matches!(c, Command::RotatePanes)));
}

// ===========================================================================
// Misc: paragraph operator semantics, machine operators, e-count edge cases
// ===========================================================================

#[test]
fn test_d_brace_does_not_eat_next_paragraph() {
    let mut rig = Rig::new(10, 8);
    belt_row(&mut rig, 0, 2, 0);
    belt_row(&mut rig, 0, 2, 1);
    // rows 2..3 empty
    belt_row(&mut rig, 0, 2, 4); // next paragraph
    rig.feed("d}");
    assert_eq!(rig.et(0, 0), None);
    assert_eq!(rig.et(0, 1), None);
    assert_eq!(
        rig.et(0, 4),
        Some(EntityType::BasicBelt),
        "next paragraph survives d}}"
    );
}

#[test]
fn test_dot_repeat_of_x() {
    let mut rig = Rig::new(10, 3);
    belt_row(&mut rig, 0, 4, 0);
    rig.feed("x");
    assert_eq!(rig.et(0, 0), None);
    rig.feed("l.");
    assert_eq!(rig.et(1, 0), None, "dot repeats x");
    assert_eq!(rig.et(2, 0), Some(EntityType::BasicBelt));
}

#[test]
fn test_dot_repeat_of_insert_session() {
    let mut rig = Rig::new(20, 3);
    rig.feed("ic<Esc>"); // place one belt at (0,0), insert advances cursor
    assert_eq!(rig.et(0, 0), Some(EntityType::BasicBelt));
    rig.feed("5|"); // jump to column 5
    rig.feed(".");
    assert_eq!(
        rig.et(4, 0),
        Some(EntityType::BasicBelt),
        "dot replays the whole insert session"
    );
}

#[test]
fn test_undo_not_clobbered_by_motions() {
    // A plain motion between an edit and `.` must not clear the dot edit.
    let mut rig = Rig::new(10, 3);
    belt_row(&mut rig, 0, 4, 0);
    rig.feed("dl"); // delete one tile
    assert_eq!(rig.et(0, 0), None);
    rig.feed("wgg$0"); // wander around
    rig.feed(".");
    assert_eq!(rig.et(0, 0), None, "cursor back at 0 after 0; dot re-deletes");
}
