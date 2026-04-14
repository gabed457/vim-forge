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
