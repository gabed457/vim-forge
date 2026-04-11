use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::Frame;

use crate::app::{AppState, Mode};
use crate::ecs::components::{EntityKind, FacingComponent, Processing};
use crate::render::glyphs;
use crate::render::highlights::{self, highlight_style};
use crate::render::viewport::Viewport;
use crate::resources::Facing;

/// Render the game grid into the given area.
///
/// Each tile occupies 2 character cells:
/// - Cell 0: entity glyph (or empty tile dot)
/// - Cell 1: resource glyph, processing indicator, or space
///
/// Highlights (cursor, selection, search, flash) are applied on top.
pub fn render_grid(frame: &mut Frame, area: Rect, app: &AppState, viewport: &Viewport) {
    let buf = frame.buffer_mut();

    // Precompute visual selection tiles
    let visual_tiles = app.visual_selection();

    // Precompute flash positions
    let flash_positions = app.animations.flash_positions();

    // Determine search state
    let search_matches: &[(usize, usize)] = &app.search.matches;
    let search_current = if app.search.has_pattern() && !app.search.matches.is_empty() {
        Some(app.search.current_match)
    } else {
        None
    };

    let is_insert = app.mode == Mode::Insert;

    for screen_row in 0..area.height {
        let map_y = viewport.offset_y + screen_row as usize;
        if map_y >= app.map.height {
            break;
        }

        for screen_col_tile in 0..(area.width / 2) {
            let map_x = viewport.offset_x + screen_col_tile as usize;
            if map_x >= app.map.width {
                break;
            }

            // Screen positions for the two character cells of this tile
            let cell0_x = area.x + screen_col_tile * 2;
            let cell1_x = cell0_x + 1;
            let cell_y = area.y + screen_row;

            // Ensure we are within buffer bounds
            if cell1_x >= area.x + area.width || cell_y >= area.y + area.height {
                break;
            }

            // Determine entity glyph and style
            let (glyph0, style0, processing_info) =
                if let Some(ent) = app.map.entity_at(map_x, map_y) {
                    let entity_type = app
                        .world
                        .get::<&EntityKind>(ent)
                        .map(|k| k.kind)
                        .unwrap_or(crate::resources::EntityType::Wall);

                    let facing = app
                        .world
                        .get::<&FacingComponent>(ent)
                        .map(|f| f.facing)
                        .unwrap_or(Facing::Right);

                    let g = glyphs::entity_glyph(entity_type, facing);
                    let s = glyphs::entity_style(entity_type);

                    // Check processing state for the second cell indicator
                    let proc_info = app
                        .world
                        .get::<&Processing>(ent)
                        .ok()
                        .and_then(|p| {
                            glyphs::processing_indicator(entity_type, &p).map(|c| (c, s))
                        });

                    (g, s, proc_info)
                } else {
                    (glyphs::empty_tile_glyph(), glyphs::empty_tile_style(), None)
                };

            // Determine second cell content: processing indicator > resource > space
            let (glyph1, style1) = if let Some((proc_char, proc_style)) = processing_info {
                (proc_char, proc_style)
            } else if let Some(resource) = app.map.resource_at(map_x, map_y) {
                (glyphs::resource_glyph(resource), glyphs::resource_style(resource))
            } else {
                (' ', Style::default())
            };

            // Determine highlight for this tile
            let highlight = highlights::resolve_highlight(
                map_x,
                map_y,
                app.cursor_x,
                app.cursor_y,
                is_insert,
                &visual_tiles,
                search_matches,
                search_current,
                &flash_positions,
            );

            // Apply styles to buffer cells
            let final_style0 = if let Some(ht) = highlight {
                merge_highlight(style0, highlight_style(ht))
            } else {
                style0
            };
            let final_style1 = if let Some(ht) = highlight {
                merge_highlight(style1, highlight_style(ht))
            } else {
                style1
            };

            let buf_cell0 = &mut buf[(cell0_x, cell_y)];
            buf_cell0.set_char(glyph0);
            buf_cell0.set_style(final_style0);

            let buf_cell1 = &mut buf[(cell1_x, cell_y)];
            buf_cell1.set_char(glyph1);
            buf_cell1.set_style(final_style1);
        }
    }
}

/// Merge a base style with a highlight style. The highlight's background takes priority;
/// the base's foreground is kept unless the highlight overrides it.
fn merge_highlight(base: Style, highlight: Style) -> Style {
    let mut result = base;
    // Apply highlight background
    if let Some(bg) = highlight.bg {
        result = result.bg(bg);
    }
    // If highlight has a foreground, use it; otherwise keep the base
    if let Some(fg) = highlight.fg {
        result = result.fg(fg);
    }
    // Merge modifiers
    result = result.add_modifier(highlight.add_modifier);
    result
}
