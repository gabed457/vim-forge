#![allow(non_snake_case)]

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use vimforge::commands::Command;
use vimforge::resources::{Direction, EntityType};
use vimforge::vim::parser::{Mode, VimParser};

fn key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

fn key_shift(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::SHIFT)
}

fn key_ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn key_esc() -> KeyEvent {
    KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)
}

#[test]
fn test_hjkl_movement() {
    let mut parser = VimParser::new();

    let cmds = parser.handle_key_event(key('h'));
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], Command::Move(Direction::Left, 1));

    let cmds = parser.handle_key_event(key('j'));
    assert_eq!(cmds[0], Command::Move(Direction::Down, 1));

    let cmds = parser.handle_key_event(key('k'));
    assert_eq!(cmds[0], Command::Move(Direction::Up, 1));

    let cmds = parser.handle_key_event(key('l'));
    assert_eq!(cmds[0], Command::Move(Direction::Right, 1));
}

#[test]
fn test_hjkl_with_count() {
    let mut parser = VimParser::new();

    // Type '3' then 'j'
    let cmds = parser.handle_key_event(key('3'));
    assert!(cmds.is_empty()); // count accumulation, no command yet

    let cmds = parser.handle_key_event(key('j'));
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], Command::Move(Direction::Down, 3));
}

#[test]
fn test_w_jump_next_entity() {
    let mut parser = VimParser::new();
    let cmds = parser.handle_key_event(key('w'));
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], Command::JumpNextEntity(1));
}

#[test]
fn test_b_jump_prev_entity() {
    let mut parser = VimParser::new();
    let cmds = parser.handle_key_event(key('b'));
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], Command::JumpPrevEntity(1));
}

#[test]
fn test_0_dollar_line_bounds() {
    let mut parser = VimParser::new();

    let cmds = parser.handle_key_event(key('0'));
    assert_eq!(cmds[0], Command::LineStart);

    let cmds = parser.handle_key_event(key('$'));
    assert_eq!(cmds[0], Command::LineEnd);
}

#[test]
fn test_gg_map_start() {
    let mut parser = VimParser::new();

    // First g: enters SecondG state
    let cmds = parser.handle_key_event(key('g'));
    assert!(cmds.is_empty());

    // Second g: produces MapStart
    let cmds = parser.handle_key_event(key('g'));
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], Command::MapStart(None));
}

#[test]
fn test_G_map_end() {
    let mut parser = VimParser::new();
    let cmds = parser.handle_key_event(key_shift('G'));
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], Command::MapEnd(None));
}

#[test]
fn test_enter_insert_mode() {
    let mut parser = VimParser::new();
    assert_eq!(parser.mode, Mode::Normal);

    let cmds = parser.handle_key_event(key('i'));
    assert_eq!(parser.mode, Mode::Insert);
    assert!(cmds.iter().any(|c| matches!(c, Command::EnterInsert(_))));
}

#[test]
fn test_insert_esc_returns_to_normal() {
    let mut parser = VimParser::new();
    parser.handle_key_event(key('i'));
    assert_eq!(parser.mode, Mode::Insert);

    let cmds = parser.handle_key_event(key_esc());
    assert_eq!(parser.mode, Mode::Normal);
    assert!(cmds.iter().any(|c| matches!(c, Command::ExitToNormal)));
}

#[test]
fn test_insert_quick_place() {
    let mut parser = VimParser::new();
    parser.handle_key_event(key('i'));

    // Quick-place: c → BasicBelt
    let cmds = parser.handle_key_event(key('c'));
    assert!(cmds
        .iter()
        .any(|c| matches!(c, Command::PlaceEntity(EntityType::BasicBelt))));

    // Quick-place: s → Smelter
    let cmds = parser.handle_key_event(key('s'));
    assert!(cmds
        .iter()
        .any(|c| matches!(c, Command::PlaceEntity(EntityType::Smelter))));

    // Quick-place: a → Assembler
    let cmds = parser.handle_key_event(key('a'));
    assert!(cmds
        .iter()
        .any(|c| matches!(c, Command::PlaceEntity(EntityType::Assembler))));

    // Quick-place: w → Wall
    let cmds = parser.handle_key_event(key('w'));
    assert!(cmds
        .iter()
        .any(|c| matches!(c, Command::PlaceEntity(EntityType::Wall))));

    // Quick-place: 1 → BasicBelt
    let cmds = parser.handle_key_event(key('1'));
    assert!(cmds
        .iter()
        .any(|c| matches!(c, Command::PlaceEntity(EntityType::BasicBelt))));
}

#[test]
fn test_insert_category_place() {
    let mut parser = VimParser::new();
    parser.handle_key_event(key('i'));

    // Shift+C → Conveyors category (no placement yet)
    let cmds = parser.handle_key_event(key_shift('C'));
    assert!(cmds.is_empty()); // category selected

    // Then 1 → BasicBelt
    let cmds = parser.handle_key_event(key('1'));
    assert!(cmds
        .iter()
        .any(|c| matches!(c, Command::PlaceEntity(EntityType::BasicBelt))));

    // Shift+S → ProcessingT1 category, then s → Smelter
    let cmds = parser.handle_key_event(key_shift('S'));
    assert!(cmds.is_empty());

    let cmds = parser.handle_key_event(key('s'));
    assert!(cmds
        .iter()
        .any(|c| matches!(c, Command::PlaceEntity(EntityType::Smelter))));
}

#[test]
fn test_insert_category_esc_returns_to_stage1() {
    let mut parser = VimParser::new();
    parser.handle_key_event(key('i'));
    assert_eq!(parser.mode, Mode::Insert);

    // Enter a category via Shift+C
    parser.handle_key_event(key_shift('C'));

    // Esc in stage 2 → back to stage 1 (still insert mode)
    let cmds = parser.handle_key_event(key_esc());
    assert_eq!(parser.mode, Mode::Insert);
    assert!(cmds.is_empty()); // no ExitToNormal

    // Esc in stage 1 → normal mode
    let cmds = parser.handle_key_event(key_esc());
    assert_eq!(parser.mode, Mode::Normal);
    assert!(cmds.iter().any(|c| matches!(c, Command::ExitToNormal)));
}

#[test]
fn test_dd_delete_line() {
    let mut parser = VimParser::new();

    let cmds = parser.handle_key_event(key('d'));
    assert!(cmds.is_empty()); // waiting for motion

    let cmds = parser.handle_key_event(key('d'));
    assert_eq!(cmds.len(), 1);
    assert!(matches!(cmds[0], Command::DemolishLine(1)));
}

#[test]
fn test_yy_yank_line() {
    let mut parser = VimParser::new();

    parser.handle_key_event(key('y'));
    let cmds = parser.handle_key_event(key('y'));
    assert!(matches!(cmds[0], Command::YankLine(1, None)));
}

#[test]
fn test_visual_mode_enter_exit() {
    let mut parser = VimParser::new();

    let cmds = parser.handle_key_event(key('v'));
    assert_eq!(parser.mode, Mode::Visual);
    assert!(cmds.iter().any(|c| matches!(c, Command::EnterVisual)));

    let cmds = parser.handle_key_event(key_esc());
    assert_eq!(parser.mode, Mode::Normal);
    assert!(cmds.iter().any(|c| matches!(c, Command::ExitToNormal)));
}

#[test]
fn test_command_mode_enter() {
    let mut parser = VimParser::new();

    let cmds = parser.handle_key_event(key(':'));
    assert_eq!(parser.mode, Mode::Command);
    assert!(cmds.iter().any(|c| matches!(c, Command::EnterCommand)));
}

#[test]
fn test_search_mode_enter() {
    let mut parser = VimParser::new();

    let cmds = parser.handle_key_event(key('/'));
    assert_eq!(parser.mode, Mode::Search);
    assert!(cmds
        .iter()
        .any(|c| matches!(c, Command::EnterSearch(true))));
}

#[test]
fn test_undo_redo() {
    let mut parser = VimParser::new();

    let cmds = parser.handle_key_event(key('u'));
    assert_eq!(cmds[0], Command::Undo);

    let cmds = parser.handle_key_event(key_ctrl('r'));
    assert_eq!(cmds[0], Command::Redo);
}

#[test]
fn test_x_delete_under_cursor() {
    let mut parser = VimParser::new();
    let cmds = parser.handle_key_event(key('x'));
    assert_eq!(cmds[0], Command::DeleteUnderCursor(1));
}

#[test]
fn test_3x_delete_three() {
    let mut parser = VimParser::new();
    parser.handle_key_event(key('3'));
    let cmds = parser.handle_key_event(key('x'));
    assert_eq!(cmds[0], Command::DeleteUnderCursor(3));
}

#[test]
fn test_tilde_rotate_entity_under_cursor() {
    let mut parser = VimParser::new();
    let cmds = parser.handle_key_event(key('~'));
    assert_eq!(cmds[0], Command::RotateEntityUnderCursor);
}

#[test]
fn test_p_paste() {
    let mut parser = VimParser::new();
    let cmds = parser.handle_key_event(key('p'));
    assert!(matches!(cmds[0], Command::Paste(None, 1, false)));
}

#[test]
fn test_P_paste_before() {
    let mut parser = VimParser::new();
    let cmds = parser.handle_key_event(key_shift('P'));
    assert!(matches!(cmds[0], Command::Paste(None, 1, true)));
}

#[test]
fn test_set_mark() {
    let mut parser = VimParser::new();
    parser.handle_key_event(key('m'));
    let cmds = parser.handle_key_event(key('a'));
    assert_eq!(cmds[0], Command::SetMark('a'));
}

#[test]
fn test_n_N_search_next_prev() {
    let mut parser = VimParser::new();

    let cmds = parser.handle_key_event(key('n'));
    assert_eq!(cmds[0], Command::SearchNext(1));

    let cmds = parser.handle_key_event(key_shift('N'));
    assert_eq!(cmds[0], Command::SearchPrev(1));
}

#[test]
fn test_star_search_word() {
    let mut parser = VimParser::new();
    let cmds = parser.handle_key_event(key('*'));
    assert!(matches!(cmds[0], Command::SearchWordUnderCursor(true)));
}

#[test]
fn test_dot_repeat() {
    let mut parser = VimParser::new();
    let cmds = parser.handle_key_event(key('.'));
    assert_eq!(cmds[0], Command::DotRepeat);
}

#[test]
fn test_ctrl_g_sidebar() {
    let mut parser = VimParser::new();
    let cmds = parser.handle_key_event(key_ctrl('g'));
    assert_eq!(cmds[0], Command::ToggleSidebar);
}

#[test]
fn test_insert_c_direct_belt_multiple() {
    // Multiple c presses in insert mode should each produce PlaceEntity(BasicBelt)
    let mut parser = VimParser::new();
    parser.handle_key_event(key('i'));

    for _ in 0..5 {
        let cmds = parser.handle_key_event(key('c'));
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], Command::PlaceEntity(EntityType::BasicBelt));
    }
}

#[test]
fn test_insert_uppercase_u_underground_exit() {
    let mut parser = VimParser::new();
    parser.handle_key_event(key('i'));

    let cmds = parser.handle_key_event(key_shift('U'));
    assert!(cmds
        .iter()
        .any(|c| matches!(c, Command::PlaceEntity(EntityType::UndergroundExit))));
}

#[test]
fn test_c_in_normal_mode_is_change_operator() {
    // c in normal mode is still the Change operator, not belt placement
    let mut parser = VimParser::new();
    assert_eq!(parser.mode, Mode::Normal);

    let cmds = parser.handle_key_event(key('c'));
    assert!(cmds.is_empty()); // waiting for motion (operator pending)
    assert_eq!(parser.mode, Mode::Normal); // still in normal mode, not insert

    // c + w = Change + motion (operator applied to range)
    let cmds = parser.handle_key_event(key('w'));
    assert!(cmds.len() >= 1); // produces motion + operator commands
}

// ===========================================================================
// Extended core-vim coverage (parser-level): operator+motion, new keys
// ===========================================================================

use vimforge::commands::{MotionKind, Operator, SubstScope};

fn feed(parser: &mut VimParser, keys: &str) -> Vec<Command> {
    let mut out = Vec::new();
    for k in vimforge::game::session::parse_key_notation(keys) {
        out.extend(parser.handle_key_event(k));
    }
    out
}

#[test]
fn test_dw_emits_operator_motion() {
    let mut parser = VimParser::new();
    let cmds = feed(&mut parser, "dw");
    assert_eq!(
        cmds,
        vec![Command::OperatorMotion(
            Operator::Delete,
            MotionKind::WordForward,
            1,
            None
        )]
    );
}

#[test]
fn test_d3l_and_3dl_counts() {
    let mut parser = VimParser::new();
    let cmds = feed(&mut parser, "d3l");
    assert_eq!(
        cmds,
        vec![Command::OperatorMotion(Operator::Delete, MotionKind::Right, 3, None)]
    );
    let cmds = feed(&mut parser, "3dl");
    assert_eq!(
        cmds,
        vec![Command::OperatorMotion(Operator::Delete, MotionKind::Right, 3, None)]
    );
    // 2d3l = 6
    let cmds = feed(&mut parser, "2d3l");
    assert_eq!(
        cmds,
        vec![Command::OperatorMotion(Operator::Delete, MotionKind::Right, 6, None)]
    );
}

#[test]
fn test_registered_operator_motion() {
    let mut parser = VimParser::new();
    let cmds = feed(&mut parser, "\"adw");
    assert_eq!(
        cmds,
        vec![Command::OperatorMotion(
            Operator::Delete,
            MotionKind::WordForward,
            1,
            Some('a')
        )]
    );
}

#[test]
fn test_dfs_emits_find_motion() {
    let mut parser = VimParser::new();
    let cmds = feed(&mut parser, "dfs");
    assert_eq!(
        cmds,
        vec![Command::OperatorMotion(
            Operator::Delete,
            MotionKind::Find(EntityType::Smelter, true),
            1,
            None
        )]
    );
}

#[test]
fn test_dG_and_dgg_emit_linewise_motions() {
    let mut parser = VimParser::new();
    let cmds = feed(&mut parser, "dG");
    assert_eq!(
        cmds,
        vec![Command::OperatorMotion(Operator::Delete, MotionKind::MapEnd(None), 1, None)]
    );
    let cmds = feed(&mut parser, "dgg");
    assert_eq!(
        cmds,
        vec![Command::OperatorMotion(Operator::Delete, MotionKind::MapStart(None), 1, None)]
    );
}

#[test]
fn test_c_dollar_and_C_equivalent() {
    let mut parser = VimParser::new();
    let expected = Command::OperatorMotion(Operator::Change, MotionKind::LineEnd, 1, None);
    assert_eq!(feed(&mut parser, "c$"), vec![expected.clone()]);
    assert_eq!(feed(&mut parser, "C"), vec![expected]);
    assert_eq!(parser.mode, Mode::Normal, "mode change happens in handler");
}

#[test]
fn test_D_delete_to_line_end() {
    let mut parser = VimParser::new();
    assert_eq!(
        feed(&mut parser, "D"),
        vec![Command::OperatorMotion(Operator::Delete, MotionKind::LineEnd, 1, None)]
    );
}

#[test]
fn test_Y_yanks_line() {
    let mut parser = VimParser::new();
    assert_eq!(feed(&mut parser, "Y"), vec![Command::YankLine(1, None)]);
    assert_eq!(feed(&mut parser, "3Y"), vec![Command::YankLine(3, None)]);
}

#[test]
fn test_s_substitute_enters_insert() {
    let mut parser = VimParser::new();
    let cmds = feed(&mut parser, "2s");
    assert_eq!(
        cmds,
        vec![Command::DeleteUnderCursor(2), Command::EnterInsert(1)]
    );
    assert_eq!(parser.mode, Mode::Insert);
    feed(&mut parser, "<Esc>");
}

#[test]
fn test_S_change_line() {
    let mut parser = VimParser::new();
    assert_eq!(feed(&mut parser, "S"), vec![Command::ChangeLine(1)]);
}

#[test]
fn test_R_enters_replace_mode() {
    let mut parser = VimParser::new();
    let cmds = feed(&mut parser, "R");
    assert!(cmds.iter().any(|c| matches!(c, Command::EnterInsert(_))));
    assert_eq!(parser.mode, Mode::Insert);
    assert!(parser.replace_mode);
    // A placement key becomes ReplaceTile in replace mode
    let cmds = feed(&mut parser, "w");
    assert_eq!(cmds, vec![Command::ReplaceTile(EntityType::Wall)]);
    feed(&mut parser, "<Esc>");
    assert!(!parser.replace_mode);
    assert_eq!(parser.mode, Mode::Normal);
}

#[test]
fn test_J_join_and_count() {
    let mut parser = VimParser::new();
    assert_eq!(feed(&mut parser, "J"), vec![Command::JoinClusters(1)]);
    assert_eq!(feed(&mut parser, "3J"), vec![Command::JoinClusters(3)]);
}

#[test]
fn test_ctrl_a_x_tier_change() {
    let mut parser = VimParser::new();
    assert_eq!(feed(&mut parser, "<C-a>"), vec![Command::TierUp(1)]);
    assert_eq!(feed(&mut parser, "2<C-x>"), vec![Command::TierDown(2)]);
}

#[test]
fn test_g_operators() {
    let mut parser = VimParser::new();
    assert_eq!(
        feed(&mut parser, "gUw"),
        vec![Command::OperatorMotion(Operator::Upgrade, MotionKind::WordForward, 1, None)]
    );
    assert_eq!(
        feed(&mut parser, "gu$"),
        vec![Command::OperatorMotion(Operator::Downgrade, MotionKind::LineEnd, 1, None)]
    );
    assert_eq!(
        feed(&mut parser, "g~l"),
        vec![Command::OperatorMotion(Operator::Rotate180, MotionKind::Right, 1, None)]
    );
    // Doubled linewise forms
    assert_eq!(
        feed(&mut parser, "gUU"),
        vec![Command::OperatorLines(Operator::Upgrade, 1, None)]
    );
    assert_eq!(
        feed(&mut parser, "guu"),
        vec![Command::OperatorLines(Operator::Downgrade, 1, None)]
    );
    assert_eq!(
        feed(&mut parser, "g~~"),
        vec![Command::OperatorLines(Operator::Rotate180, 1, None)]
    );
}

#[test]
fn test_ge_and_E_motions() {
    let mut parser = VimParser::new();
    assert_eq!(feed(&mut parser, "ge"), vec![Command::JumpEndClusterBack(1)]);
    assert_eq!(feed(&mut parser, "E"), vec![Command::JumpEndClusterBig(1)]);
    assert_eq!(feed(&mut parser, "2e"), vec![Command::JumpEndCluster(2)]);
}

#[test]
fn test_scroll_keys() {
    let mut parser = VimParser::new();
    assert_eq!(feed(&mut parser, "<C-d>"), vec![Command::ScrollHalfPageDown(1)]);
    assert_eq!(feed(&mut parser, "<C-u>"), vec![Command::ScrollHalfPageUp(1)]);
    assert_eq!(feed(&mut parser, "<C-f>"), vec![Command::ScrollFullPageDown(1)]);
    assert_eq!(feed(&mut parser, "<C-b>"), vec![Command::ScrollFullPageUp(1)]);
    assert_eq!(feed(&mut parser, "zz"), vec![Command::ScrollCenterCursor]);
    assert_eq!(feed(&mut parser, "zt"), vec![Command::ScrollCursorTop]);
    assert_eq!(feed(&mut parser, "zb"), vec![Command::ScrollCursorBottom]);
}

#[test]
fn test_ctrl_b_is_not_contracts_anymore() {
    let mut parser = VimParser::new();
    let cmds = feed(&mut parser, "<C-b>");
    assert!(!cmds.iter().any(|c| matches!(c, Command::CmdContracts)));
}

#[test]
fn test_jumplist_keys() {
    let mut parser = VimParser::new();
    assert_eq!(feed(&mut parser, "<C-o>"), vec![Command::JumpListBack(1)]);
    assert_eq!(feed(&mut parser, "<C-i>"), vec![Command::JumpListForward(1)]);
}

#[test]
fn test_sentence_and_column_motions() {
    let mut parser = VimParser::new();
    assert_eq!(feed(&mut parser, "("), vec![Command::JumpPrevMachine(1)]);
    assert_eq!(feed(&mut parser, ")"), vec![Command::JumpNextMachine(1)]);
    assert_eq!(feed(&mut parser, "5|"), vec![Command::JumpColumn(5)]);
    assert_eq!(feed(&mut parser, "_"), vec![Command::FirstEntityInRow]);
    assert_eq!(feed(&mut parser, "+"), vec![Command::FirstEntityRowDown(1)]);
    assert_eq!(feed(&mut parser, "-"), vec![Command::FirstEntityRowUp(1)]);
}

#[test]
fn test_gi_and_gv() {
    let mut parser = VimParser::new();
    let cmds = feed(&mut parser, "gi");
    assert_eq!(cmds, vec![Command::JumpLastInsert, Command::EnterInsert(1)]);
    assert_eq!(parser.mode, Mode::Insert);
    feed(&mut parser, "<Esc>");
    assert_eq!(feed(&mut parser, "gv"), vec![Command::ReselectVisual]);
}

#[test]
fn test_extended_text_object_whitelist() {
    for (keys, obj) in [
        ("di[", '['),
        ("da{", '{'),
        ("di<", '<'),
        ("daB", 'B'),
        ("di\"", '"'),
        ("da'", '\''),
        ("dit", 't'),
        ("dis", 's'),
    ] {
        let mut parser = VimParser::new();
        let cmds = feed(&mut parser, keys);
        assert_eq!(
            cmds,
            vec![Command::TextObjectOp(
                Operator::Delete,
                keys.contains('i'),
                obj,
                None
            )],
            "sequence {keys}"
        );
    }
}

#[test]
fn test_visual_counts_and_text_objects() {
    let mut parser = VimParser::new();
    feed(&mut parser, "v");
    assert_eq!(feed(&mut parser, "5l"), vec![Command::Move(Direction::Right, 5)]);
    assert_eq!(
        feed(&mut parser, "iw"),
        vec![Command::VisualTextObject(true, 'w')]
    );
    // x acts as delete
    let cmds = feed(&mut parser, "x");
    assert_eq!(cmds, vec![Command::VisualOperator(Operator::Delete)]);
    assert_eq!(parser.mode, Mode::Normal);
}

#[test]
fn test_visual_f_and_replace() {
    let mut parser = VimParser::new();
    feed(&mut parser, "v");
    assert_eq!(
        feed(&mut parser, "fs"),
        vec![Command::FindEntity(EntityType::Smelter, 1, true)]
    );
    assert_eq!(
        feed(&mut parser, "rw"),
        vec![Command::VisualReplace(EntityType::Wall)]
    );
    assert_eq!(parser.mode, Mode::Normal);
}

#[test]
fn test_visual_block_insert_keys() {
    let mut parser = VimParser::new();
    feed(&mut parser, "<C-v>");
    assert_eq!(parser.mode, Mode::VisualBlock);
    let cmds = feed(&mut parser, "Ic");
    assert_eq!(
        cmds,
        vec![Command::VisualBlockInsert(EntityType::BasicBelt, false)]
    );
    assert_eq!(parser.mode, Mode::Normal);

    feed(&mut parser, "<C-v>");
    let cmds = feed(&mut parser, "Aw");
    assert_eq!(
        cmds,
        vec![Command::VisualBlockInsert(EntityType::Wall, true)]
    );
}

#[test]
fn test_visual_block_O_swaps_corner() {
    let mut parser = VimParser::new();
    feed(&mut parser, "<C-v>");
    assert_eq!(feed(&mut parser, "O"), vec![Command::VisualSwapCorner]);
    feed(&mut parser, "<Esc>");
}

#[test]
fn test_command_line_substitute_parsing() {
    let mut parser = VimParser::new();
    let cmds = feed(&mut parser, ":%s/belt/fastbelt/g<CR>");
    assert_eq!(
        cmds,
        vec![Command::EnterCommand, Command::CmdSubstitute {
            scope: SubstScope::WholeMap,
            pattern: "belt".to_string(),
            replacement: "fastbelt".to_string(),
            global: true,
        }]
    );
    let cmds = feed(&mut parser, ":s/belt/wall/<CR>");
    assert_eq!(
        cmds,
        vec![Command::EnterCommand, Command::CmdSubstitute {
            scope: SubstScope::CurrentRow,
            pattern: "belt".to_string(),
            replacement: "wall".to_string(),
            global: false,
        }]
    );
    let cmds = feed(&mut parser, ":2,5s/belt/pipe/g<CR>");
    assert_eq!(
        cmds,
        vec![Command::EnterCommand, Command::CmdSubstitute {
            scope: SubstScope::Rows(1, 4),
            pattern: "belt".to_string(),
            replacement: "pipe".to_string(),
            global: true,
        }]
    );
}

#[test]
fn test_command_line_global_delete_and_aliases() {
    let mut parser = VimParser::new();
    assert_eq!(
        feed(&mut parser, ":g/belt/d<CR>"),
        vec![Command::EnterCommand, Command::CmdGlobalDelete("belt".to_string())]
    );
    assert_eq!(
        feed(&mut parser, ":only<CR>"),
        vec![Command::EnterCommand, Command::CloseOtherPanes]
    );
    assert_eq!(
        feed(&mut parser, ":sp<CR>"),
        vec![Command::EnterCommand, Command::SplitHorizontal]
    );
    assert_eq!(
        feed(&mut parser, ":vsp<CR>"),
        vec![Command::EnterCommand, Command::SplitVertical]
    );
}

#[test]
fn test_ampersand_and_at_colon() {
    let mut parser = VimParser::new();
    assert_eq!(feed(&mut parser, "&"), vec![Command::RepeatSubstitute]);
    // @: repeats the last command line
    feed(&mut parser, ":pause<CR>");
    assert_eq!(feed(&mut parser, "@:"), vec![Command::CmdPause]);
    // (typed :pause emits EnterCommand + CmdPause; @: replays only the command)
}

#[test]
fn test_ctrl_w_extras_parser() {
    let mut parser = VimParser::new();
    assert_eq!(feed(&mut parser, "<C-w>w"), vec![Command::CyclePane]);
    assert_eq!(feed(&mut parser, "<C-w>c"), vec![Command::ClosePane]);
    assert_eq!(feed(&mut parser, "<C-w>x"), vec![Command::SwapPanes]);
    assert_eq!(feed(&mut parser, "<C-w>r"), vec![Command::RotatePanes]);
}

#[test]
fn test_black_hole_register_select() {
    let mut parser = VimParser::new();
    let cmds = feed(&mut parser, "\"_dd");
    assert_eq!(
        cmds,
        vec![Command::OperatorLines(Operator::Delete, 1, Some('_'))]
    );
}
