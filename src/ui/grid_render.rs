use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::Frame;

use crate::app::{AppState, Mode};
use crate::ecs::components::{
    EntityKind, FacingComponent, MultiTile, PartOfBuilding, Position, Processing,
};
use crate::render::glyphs::{self, MachineState};
use crate::render::highlights::{self, highlight_style};
use crate::render::viewport::Viewport;
use crate::resources::{EntityType, Facing};

/// Render the game grid into the given area.
///
/// Each tile occupies 2 character cells:
/// - Cell 0: building art column 0 (primary art character)
/// - Cell 1: building art column 1 (or resource glyph / processing indicator)
///
/// Multi-tile buildings show cohesive ASCII art across all their tiles.
/// Highlights (cursor, selection, search, flash) are applied on top.
pub fn render_grid(frame: &mut Frame, area: Rect, app: &AppState, viewport: &Viewport) {
    let buf = frame.buffer_mut();

    let visual_tiles = app.visual_selection();
    let flash_positions = app.animations.flash_positions();
    let search_matches: &[(usize, usize)] = &app.search.matches;
    let search_current = if app.search.has_pattern() && !app.search.matches.is_empty() {
        Some(app.search.current_match)
    } else {
        None
    };

    let is_insert = app.mode == Mode::Insert;
    let frame_counter = app.animations.frame_counter;

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

            let cell0_x = area.x + screen_col_tile * 2;
            let cell1_x = cell0_x + 1;
            let cell_y = area.y + screen_row;

            if cell1_x >= area.x + area.width || cell_y >= area.y + area.height {
                break;
            }

            // Resolve entity at this tile, handling multi-tile buildings
            let tile_info = resolve_tile_entity(app, map_x, map_y);

            let (glyph0, glyph1, style0, style1) = match tile_info {
                Some(info) => {
                    // Get 2-char art for this tile
                    let [art0, art1] =
                        glyphs::entity_art(info.entity_type, info.facing, info.tile_index);

                    // Determine machine state for styling
                    let state = info.machine_state;
                    let base_style =
                        glyphs::entity_style_for_state(info.entity_type, state, frame_counter);

                    // Column 1: processing indicator > resource > art character
                    let (g1, s1) = if let Some(proc_char) = info.processing_char {
                        (proc_char, base_style)
                    } else if let Some(resource) = app.map.resource_at(map_x, map_y) {
                        (glyphs::resource_glyph(resource), glyphs::resource_style(resource))
                    } else {
                        (art1, base_style)
                    };

                    (art0, g1, base_style, s1)
                }
                None => {
                    // Empty tile — show terrain or default dot
                    let terrain = app.map.terrain_at(map_x, map_y);
                    let (g, s) = terrain_glyph_style(terrain);
                    let (g1, s1) = if let Some(resource) = app.map.resource_at(map_x, map_y) {
                        (glyphs::resource_glyph(resource), glyphs::resource_style(resource))
                    } else {
                        ('\u{00B7}', s) // second dot for empty tile
                    };
                    (g, g1, s, s1)
                }
            };

            // Determine highlight for this tile
            let highlight = highlights::resolve_highlight(
                map_x, map_y, app.cursor_x, app.cursor_y, is_insert,
                &visual_tiles, search_matches, search_current, &flash_positions,
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

// ---------------------------------------------------------------------------
// Tile entity resolution
// ---------------------------------------------------------------------------

/// Information about the entity at a specific tile, resolved through multi-tile lookups.
struct TileEntityInfo {
    entity_type: EntityType,
    facing: Facing,
    tile_index: usize,
    machine_state: MachineState,
    /// Processing countdown character (only on the center/anchor tile).
    processing_char: Option<char>,
}

/// Resolve the entity at (map_x, map_y), following PartOfBuilding references
/// to the anchor entity for multi-tile buildings.
fn resolve_tile_entity(app: &AppState, map_x: usize, map_y: usize) -> Option<TileEntityInfo> {
    let ent = app.map.entity_at(map_x, map_y)?;

    // Check if this tile is a secondary tile of a multi-tile building
    let (anchor_ent, tile_index) =
        if let Ok(pob) = app.world.get::<&PartOfBuilding>(ent) {
            let anchor = pob.anchor;
            let anchor_pos = app
                .world
                .get::<&Position>(anchor)
                .map(|p| (p.x, p.y))
                .unwrap_or((map_x, map_y));
            let anchor_facing = app
                .world
                .get::<&FacingComponent>(anchor)
                .map(|f| f.facing)
                .unwrap_or(Facing::Right);
            let (w, h) = app
                .world
                .get::<&MultiTile>(anchor)
                .map(|m| (m.width, m.height))
                .unwrap_or((1, 1));
            let idx =
                compute_tile_index(anchor_pos.0, anchor_pos.1, map_x, map_y, anchor_facing, w, h);
            (anchor, idx)
        } else {
            (ent, 0)
        };

    let entity_type = app
        .world
        .get::<&EntityKind>(anchor_ent)
        .map(|k| k.kind)
        .unwrap_or(EntityType::Wall);

    let facing = app
        .world
        .get::<&FacingComponent>(anchor_ent)
        .map(|f| f.facing)
        .unwrap_or(Facing::Right);

    // Determine machine state and processing indicator
    let (machine_state, processing_char) = if let Ok(proc) =
        app.world.get::<&Processing>(anchor_ent)
    {
        if proc.is_processing() {
            let art = glyphs::building_art(entity_type);
            let center = art.rows.len() / 2;
            let indicator = if tile_index == center {
                glyphs::processing_indicator(entity_type, &proc)
            } else {
                None
            };
            (MachineState::Processing, indicator)
        } else {
            (MachineState::Idle, None)
        }
    } else {
        (MachineState::Idle, None)
    };

    Some(TileEntityInfo {
        entity_type,
        facing,
        tile_index,
        machine_state,
        processing_char,
    })
}

/// Compute the tile index (art row) for a tile of a multi-tile building.
///
/// Given the anchor position, the tile position, the facing, and the building footprint size,
/// returns which art row this tile corresponds to.
fn compute_tile_index(
    ax: usize, ay: usize,
    tx: usize, ty: usize,
    facing: Facing,
    w: usize, h: usize,
) -> usize {
    match facing {
        Facing::Right => ty.saturating_sub(ay),
        Facing::Down => (ax + w).saturating_sub(1).saturating_sub(tx),
        Facing::Left => (ay + h).saturating_sub(1).saturating_sub(ty),
        Facing::Up => tx.saturating_sub(ax),
    }
}

// ---------------------------------------------------------------------------
// Terrain & highlight helpers
// ---------------------------------------------------------------------------

/// Get the glyph and style for a terrain type. Uses ONLY Color::Rgb.
fn terrain_glyph_style(terrain: crate::map::terrain::Terrain) -> (char, Style) {
    let glyph = terrain.glyph();
    let (fr, fg, fb) = terrain.fg_color();
    let mut style = Style::default().fg(Color::Rgb(fr, fg, fb));

    if let Some((br, bg_c, bb)) = terrain.bg_color() {
        style = style.bg(Color::Rgb(br, bg_c, bb));
    }

    (glyph, style)
}

/// Merge a base style with a highlight style. The highlight's background takes priority;
/// the base's foreground is kept unless the highlight overrides it.
fn merge_highlight(base: Style, highlight: Style) -> Style {
    let mut result = base;
    if let Some(bg) = highlight.bg {
        result = result.bg(bg);
    }
    if let Some(fg) = highlight.fg {
        result = result.fg(fg);
    }
    result = result.add_modifier(highlight.add_modifier);
    result
}
