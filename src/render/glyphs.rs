use ratatui::style::{Color, Modifier, Style};

use crate::ecs::components::Processing;
use crate::render::colors::dim_color;
use crate::resources::{EntityType, Facing, Resource};

/// Machine state for per-state styling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineState {
    Idle,
    Processing,
    Blocked,
    Broken,
}

/// Returns the display character for an entity type, respecting facing for belts.
pub fn entity_glyph(entity_type: EntityType, facing: Facing) -> char {
    match entity_type {
        EntityType::BasicBelt => facing.arrow_glyph(),
        EntityType::FastBelt => match facing {
            Facing::Up => '\u{21D1}',    // double arrow up
            Facing::Down => '\u{21D3}',  // double arrow down
            Facing::Left => '\u{21D0}',  // double arrow left
            Facing::Right => '\u{21D2}', // double arrow right
        },
        EntityType::ExpressBelt => match facing {
            Facing::Up => '\u{21E7}',    // white arrow up
            Facing::Down => '\u{21E9}',  // white arrow down
            Facing::Left => '\u{21E6}',  // white arrow left
            Facing::Right => '\u{21E8}', // white arrow right
        },
        _ => entity_type.glyph(),
    }
}

/// Returns the ratatui Style for an entity type using ONLY Color::Rgb.
pub fn entity_style(entity_type: EntityType) -> Style {
    let (r, g, b) = entity_type.color();
    let mut style = Style::default().fg(Color::Rgb(r, g, b));
    // Bold for non-transport, non-wall entities
    match entity_type {
        EntityType::BasicBelt | EntityType::FastBelt | EntityType::ExpressBelt => {}
        EntityType::Wall | EntityType::ReinforcedWall => {}
        _ => {
            style = style.add_modifier(Modifier::BOLD);
        }
    }
    style
}

/// Returns style for an entity based on its machine state.
pub fn entity_style_for_state(entity_type: EntityType, state: MachineState) -> Style {
    let base_color = entity_type.color();
    match state {
        MachineState::Idle => {
            let dimmed = dim_color(base_color, 0.5);
            Style::default().fg(Color::Rgb(dimmed.0, dimmed.1, dimmed.2))
        }
        MachineState::Processing => {
            let (r, g, b) = base_color;
            Style::default()
                .fg(Color::Rgb(r, g, b))
                .add_modifier(Modifier::BOLD)
        }
        MachineState::Blocked => {
            Style::default()
                .fg(Color::Rgb(220, 180, 40))
                .add_modifier(Modifier::BOLD)
        }
        MachineState::Broken => {
            Style::default()
                .fg(Color::Rgb(200, 0, 0))
                .add_modifier(Modifier::BOLD)
        }
    }
}

/// Returns a dimmed conveyor style (when idle / not carrying a resource).
pub fn conveyor_idle_style() -> Style {
    Style::default()
        .fg(Color::Rgb(50, 50, 50))
        .add_modifier(Modifier::DIM)
}

/// Returns the display character for a resource floating on the grid.
pub fn resource_glyph(resource: Resource) -> char {
    resource.glyph()
}

/// Returns the style for a resource glyph using ONLY Color::Rgb.
pub fn resource_style(resource: Resource) -> Style {
    let (r, g, b) = resource.color();
    let mut style = Style::default().fg(Color::Rgb(r, g, b));
    if resource.tier() >= 2 {
        style = style.add_modifier(Modifier::BOLD);
    }
    style
}

/// Returns the glyph for an empty tile.
pub fn empty_tile_glyph() -> char {
    '\u{00B7}' // middle dot
}

/// Returns the style for an empty tile using ONLY Color::Rgb.
pub fn empty_tile_style() -> Style {
    Style::default().fg(Color::Rgb(60, 60, 60))
}

/// Belt animation: get the animated glyph for an empty belt based on frame counter.
pub fn belt_animated_glyph(belt_type: EntityType, facing: Facing, frame: u32) -> char {
    match belt_type {
        EntityType::BasicBelt => {
            // Cycle dot -> arrow every 4 frames
            if (frame / 4) % 2 == 0 {
                '\u{00B7}' // middle dot
            } else {
                match facing {
                    Facing::Right => '\u{203A}', // single right-pointing angle
                    Facing::Left => '\u{2039}',
                    Facing::Up => '\u{02C4}',
                    Facing::Down => '\u{02C5}',
                }
            }
        }
        EntityType::FastBelt => {
            // Cycle every 2 frames
            if (frame / 2) % 2 == 0 {
                '\u{00B7}'
            } else {
                match facing {
                    Facing::Right => '\u{00BB}', // >>
                    Facing::Left => '\u{00AB}',
                    Facing::Up => '\u{02C4}',
                    Facing::Down => '\u{02C5}',
                }
            }
        }
        EntityType::ExpressBelt => {
            // Cycle every frame
            if frame % 2 == 0 {
                '\u{00B7}'
            } else {
                match facing {
                    Facing::Right => '\u{21D2}',
                    Facing::Left => '\u{21D0}',
                    Facing::Up => '\u{21D1}',
                    Facing::Down => '\u{21D3}',
                }
            }
        }
        _ => '\u{00B7}',
    }
}

/// Returns the style for a belt type using ONLY Color::Rgb.
pub fn belt_style(belt_type: EntityType) -> Style {
    match belt_type {
        EntityType::BasicBelt => Style::default()
            .fg(Color::Rgb(200, 200, 200))
            .bg(Color::Rgb(20, 20, 22)),
        EntityType::FastBelt => Style::default()
            .fg(Color::Rgb(220, 200, 80))
            .bg(Color::Rgb(30, 26, 8)),
        EntityType::ExpressBelt => Style::default()
            .fg(Color::Rgb(100, 160, 255))
            .bg(Color::Rgb(10, 20, 40)),
        _ => Style::default().fg(Color::Rgb(200, 200, 200)),
    }
}

/// Format a processing indicator for machines.
/// Returns None if the machine is idle.
pub fn processing_indicator(_entity_type: EntityType, processing: &Processing) -> Option<char> {
    if !processing.is_processing() {
        return None;
    }
    // Return the remaining tick count as a digit character (capped at 9)
    let ticks = processing.ticks_remaining.min(9) as u8;
    Some((b'0' + ticks) as char)
}
