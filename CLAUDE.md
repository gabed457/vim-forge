# VimForge

Terminal TUI factory/automation game with vim keybinding grammar as the control scheme.

## Architecture
- ECS: hecs
- TUI: ratatui + crossterm
- Serialization: serde + serde_json

## Key conventions
- All shared types (Command, EntityType, Resource, Facing, Range, Blueprint) live in `src/commands.rs` and `src/resources.rs`
- Entity behavior is defined by component composition in `src/ecs/components.rs`
- Vim parser is a state machine in `src/vim/parser.rs` — NOT a giant match block
- Rendering never mutates game state
- All editable actions go through the undo system

## Testing
- `cargo test` runs all tests
- Parser tests verify every key combination produces the correct Command variant
- Simulation tests verify resource flow timing and machine behavior
- Integration tests simulate full sequences (place entities, tick, check output)

## Dependencies
ONLY: ratatui, crossterm, hecs, serde, serde_json. Do not add any others.
