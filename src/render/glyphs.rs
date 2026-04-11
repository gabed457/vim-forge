use ratatui::style::{Color, Modifier, Style};

use crate::ecs::components::Processing;
use crate::resources::{EntityType, Facing, Resource};

/// Returns the display character for an entity type, respecting facing for conveyors.
pub fn entity_glyph(entity_type: EntityType, facing: Facing) -> char {
    match entity_type {
        EntityType::OreDeposit => 'O',
        EntityType::Smelter => 'S',
        EntityType::Assembler => 'A',
        EntityType::Conveyor => facing.arrow_glyph(), // arrow chars
        EntityType::Splitter => 'Y',
        EntityType::Merger => '\u{03BB}', // lambda
        EntityType::OutputBin => 'B',
        EntityType::Wall => '\u{2588}', // full block
    }
}

/// Returns the ratatui Style for an entity type.
pub fn entity_style(entity_type: EntityType) -> Style {
    match entity_type {
        EntityType::OreDeposit => Style::default()
            .fg(Color::Rgb(139, 119, 42))
            .add_modifier(Modifier::BOLD),
        EntityType::Smelter => Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD),
        EntityType::Assembler => Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        EntityType::Conveyor => Style::default().fg(Color::White),
        EntityType::Splitter => Style::default().fg(Color::Yellow),
        EntityType::Merger => Style::default().fg(Color::Yellow),
        EntityType::OutputBin => Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        EntityType::Wall => Style::default().fg(Color::DarkGray),
    }
}

/// Returns a dimmed conveyor style (when idle / not carrying a resource).
pub fn conveyor_idle_style() -> Style {
    Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::DIM)
}

/// Returns the display character for a resource floating on the grid.
pub fn resource_glyph(resource: Resource) -> char {
    match resource {
        Resource::Ore => 'o',
        Resource::Ingot => 'i',
        Resource::Widget => 'w',
    }
}

/// Returns the style for a resource glyph.
pub fn resource_style(resource: Resource) -> Style {
    match resource {
        Resource::Ore => Style::default().fg(Color::Rgb(180, 140, 60)),
        Resource::Ingot => Style::default().fg(Color::Rgb(200, 200, 200)),
        Resource::Widget => Style::default()
            .fg(Color::Rgb(100, 220, 100))
            .add_modifier(Modifier::BOLD),
    }
}

/// Returns the glyph for an empty tile.
pub fn empty_tile_glyph() -> char {
    '\u{00B7}' // middle dot
}

/// Returns the style for an empty tile.
pub fn empty_tile_style() -> Style {
    Style::default().fg(Color::Rgb(60, 60, 60))
}

/// Format a processing indicator for machines (e.g., "S2" for smelter with 2 ticks left,
/// "A5" for assembler with 5 ticks left). Returns None if the machine is idle.
pub fn processing_indicator(_entity_type: EntityType, processing: &Processing) -> Option<char> {
    if !processing.is_processing() {
        return None;
    }
    // Return the remaining tick count as a digit character (capped at 9)
    let ticks = processing.ticks_remaining.min(9) as u8;
    Some((b'0' + ticks) as char)
}
