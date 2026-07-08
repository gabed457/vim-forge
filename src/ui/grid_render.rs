use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::Frame;

use crate::app::{AppState, Mode};
use crate::ecs::components::{
    EntityKind, FacingComponent, MultiTile, PartOfBuilding, Position, Processing,
};
use crate::render::animations::flash_spark;
use crate::render::colors::{apply_day_night, apply_day_night_fg, blend_trail};
use crate::render::glyphs::{self, MachineState};
use crate::render::highlights::{self, highlight_style, mark_badge_style};
use crate::render::viewport::Viewport;
use crate::resources::{EntityType, Facing};

/// Render the game grid into the given area.
///
/// Each tile occupies 2 character cells:
/// - Cell 0: building art column 0 (primary art character / belt direction)
/// - Cell 1: building art column 1 (or item glyph / processing indicator /
///   particle / mark badge)
///
/// Multi-tile buildings show cohesive ASCII art across all their tiles.
/// Everything is bathed in a smooth day/night tint driven by `app.day_tick`,
/// then highlights (cursor, selection, search, flash) are applied on top so
/// they stay crisp at any time of day.
pub fn render_grid(frame: &mut Frame, area: Rect, app: &AppState, viewport: &Viewport) {
    if area.width < 2 || area.height == 0 {
        return;
    }
    let buf = frame.buffer_mut();

    let visual_tiles = app.visual_selection();
    let flashes = app.animations.flashes();
    let search_matches: &[(usize, usize)] = &app.search.matches;
    let search_current = if app.search.has_pattern() && !app.search.matches.is_empty() {
        Some(app.search.current_match)
    } else {
        None
    };
    let marks = app.marks.list();

    let is_insert = app.mode == Mode::Insert;
    let frame_counter = app.animations.frame_counter;
    let day_tick = app.day_tick;

    // Hard clip boundaries (exclusive) — the grid must never paint outside.
    let clip_right = area.x + area.width;
    let clip_bottom = area.y + area.height;

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

            let cell0_x = area.x + viewport.pad_left + screen_col_tile * 2;
            let cell1_x = cell0_x + 1;
            let cell_y = area.y + viewport.pad_top + screen_row;

            if cell0_x >= clip_right || cell_y >= clip_bottom {
                break;
            }
            let cell1_visible = cell1_x < clip_right;

            // Resolve entity at this tile, handling multi-tile buildings
            let tile_info = resolve_tile_entity(app, map_x, map_y);
            let tile_resource = app.map.resource_at(map_x, map_y);

            let (glyph0, glyph1, style0, style1) = match tile_info {
                Some(info) => {
                    let is_belt = matches!(
                        info.entity_type,
                        EntityType::BasicBelt | EntityType::FastBelt | EntityType::ExpressBelt
                    );

                    // Get 2-char art for this tile
                    let [art0, art1] = glyphs::entity_art(
                        info.entity_type,
                        info.facing,
                        info.tile_row,
                        info.tile_col,
                    );

                    let state = info.machine_state;
                    let base_style = if is_belt {
                        glyphs::belt_style(info.entity_type)
                    } else {
                        // Phase-shift the processing pulse per tile so big
                        // factories shimmer as a ripple, not a strobe.
                        let phase = frame_counter.wrapping_add((map_x * 3 + map_y * 5) as u32);
                        glyphs::entity_style_for_state(info.entity_type, state, phase)
                    };

                    // Cell 0: belt direction arrow / building art. Port arrows
                    // on machine edges get a bright accent so I/O edges pop.
                    let s0 = if !is_belt && glyphs::is_port_char(art0) {
                        glyphs::port_style(info.entity_type)
                    } else {
                        base_style
                    };

                    // Cell 1: processing indicator > item on belt > port > art
                    let (g1, s1) = if let Some(proc_char) = info.processing_char {
                        (
                            proc_char,
                            base_style.add_modifier(Modifier::BOLD),
                        )
                    } else if let Some(resource) = tile_resource {
                        // Item riding the tile: bright glyph over the belt bed
                        // so cargo is always visible against the lane color.
                        let mut s = glyphs::resource_style(resource);
                        if is_belt {
                            if let Some(bg) = style_bg_rgb(&base_style) {
                                s = s.bg(Color::Rgb(bg.0, bg.1, bg.2));
                            }
                        }
                        (glyphs::resource_glyph(resource), s)
                    } else if is_belt {
                        // Empty belt: animate the lane so flow direction reads
                        // at a glance even without cargo.
                        (
                            glyphs::belt_animated_glyph(info.entity_type, info.facing, frame_counter),
                            base_style,
                        )
                    } else if glyphs::is_port_char(art1) {
                        (art1, glyphs::port_style(info.entity_type))
                    } else {
                        (art1, base_style)
                    };

                    (art0, g1, s0, s1)
                }
                None => {
                    // Empty tile — show terrain or default dot
                    let terrain = app.map.terrain_at(map_x, map_y);
                    let (g, s) = terrain_glyph_style(terrain);
                    let (g1, s1) = if let Some(resource) = tile_resource {
                        // Resources sitting on open ground stay subtle —
                        // the bright treatment is reserved for belt cargo.
                        (glyphs::resource_glyph(resource), ground_resource_style(resource, s))
                    } else {
                        ('\u{00B7}', s) // second dot for empty tile
                    };
                    (g, g1, s, s1)
                }
            };

            // Item trails: fading wakes behind moving cargo, blended into
            // the tile background (skipped when an item currently sits here).
            let (mut style0, mut style1) = (style0, style1);
            if tile_resource.is_none() {
                if let Some(trail) = app.trails.get_at(map_x, map_y) {
                    let intensity = trail.intensity();
                    style0 = blend_bg(style0, trail.color, intensity);
                    style1 = blend_bg(style1, trail.color, intensity);
                }
            }

            // Ambient day/night tint (smooth lerp; backgrounds fully,
            // foregrounds gently so machines stay readable at night).
            style0 = tint_style(style0, day_tick);
            style1 = tint_style(style1, day_tick);

            // Overlays on cell 1: flash sparks > particles > mark badges.
            let (mut glyph1, mut overlay1) = (glyph1, None::<Style>);
            if tile_resource.is_none() {
                if let Some((ch, s)) = mark_badge_at(&marks, map_x, map_y) {
                    glyph1 = ch;
                    overlay1 = Some(s);
                }
            }
            if let Some(p) = app.particles.get_at(map_x, map_y) {
                glyph1 = p.glyph;
                let mut s = style1.fg(Color::Rgb(p.fg.0, p.fg.1, p.fg.2));
                s = s.add_modifier(Modifier::BOLD);
                overlay1 = Some(s);
            }
            if let Some(flash) = flashes.iter().find(|f| f.x == map_x && f.y == map_y) {
                let (ch, fg) = flash_spark(flash.kind, flash.frames_remaining);
                glyph1 = ch;
                overlay1 = Some(style1.fg(Color::Rgb(fg.0, fg.1, fg.2)).add_modifier(Modifier::BOLD));
            }
            if let Some(s) = overlay1 {
                style1 = s;
            }

            // Determine highlight for this tile
            let highlight = highlights::resolve_highlight(
                map_x, map_y, app.cursor_x, app.cursor_y, is_insert,
                &visual_tiles, search_matches, search_current, flashes,
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

            if cell1_visible {
                let buf_cell1 = &mut buf[(cell1_x, cell_y)];
                buf_cell1.set_char(glyph1);
                buf_cell1.set_style(final_style1);
            }
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
    tile_row: usize,
    tile_col: usize,
    machine_state: MachineState,
    /// Processing countdown character (only on the center/anchor tile).
    processing_char: Option<char>,
}

/// Resolve the entity at (map_x, map_y), following PartOfBuilding references
/// to the anchor entity for multi-tile buildings.
fn resolve_tile_entity(app: &AppState, map_x: usize, map_y: usize) -> Option<TileEntityInfo> {
    let ent = app.map.entity_at(map_x, map_y)?;

    // Check if this tile is a secondary tile of a multi-tile building
    let (anchor_ent, tile_row, tile_col) =
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
            let (tr, tc) =
                compute_tile_coords(anchor_pos.0, anchor_pos.1, map_x, map_y, anchor_facing, w, h);
            (anchor, tr, tc)
        } else {
            (ent, 0, 0)
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
            let center_row = art.height / 2;
            let center_col = art.width / 2;
            let indicator = if tile_row == center_row && tile_col == center_col {
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
        tile_row,
        tile_col,
        machine_state,
        processing_char,
    })
}

/// Compute the 2D tile coordinates (row, col) for a tile of a multi-tile building.
///
/// Given the anchor position, the tile position, the facing, and the rotated footprint size,
/// returns which (row, col) this tile corresponds to in screen space.
/// The row/col here are relative to the anchor in the rotated coordinate system.
fn compute_tile_coords(
    ax: usize, ay: usize,
    tx: usize, ty: usize,
    _facing: Facing,
    _w: usize, _h: usize,
) -> (usize, usize) {
    // In screen space, the anchor is always at (0, 0) of the rotated footprint.
    // dx = tx - ax (column offset), dy = ty - ay (row offset)
    let dx = tx.saturating_sub(ax);
    let dy = ty.saturating_sub(ay);
    // Screen row = dy, screen col = dx
    // These are in the rotated coordinate system; the art lookup will
    // inverse-rotate them via rotated_art_coords in entity_art().
    (dy, dx)
}

// ---------------------------------------------------------------------------
// Terrain, tint & highlight helpers
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

/// Muted style for a resource lying on open ground: colored but dimmed,
/// keeping the terrain's background so the world stays calm.
fn ground_resource_style(resource: crate::resources::Resource, terrain_style: Style) -> Style {
    let (r, g, b) = resource.color();
    let dim = crate::render::colors::dim_color((r, g, b), 0.7);
    let mut s = Style::default().fg(Color::Rgb(dim.0, dim.1, dim.2));
    if let Some(bg) = terrain_style.bg {
        s = s.bg(bg);
    }
    s
}

/// Extract the Rgb triple from a style's background, if it has one.
fn style_bg_rgb(style: &Style) -> Option<(u8, u8, u8)> {
    match style.bg {
        Some(Color::Rgb(r, g, b)) => Some((r, g, b)),
        _ => None,
    }
}

/// Blend a wake color into a style's background at the given intensity.
fn blend_bg(style: Style, color: (u8, u8, u8), intensity: f64) -> Style {
    let base = style_bg_rgb(&style).unwrap_or((14, 16, 20));
    let (r, g, b) = blend_trail(base, color, intensity);
    style.bg(Color::Rgb(r, g, b))
}

/// Apply the smooth day/night tint to a tile style: background fully, glyph
/// foreground gently (machines must stay readable at night).
fn tint_style(style: Style, day_tick: u32) -> Style {
    let mut result = style;
    if let Some(Color::Rgb(r, g, b)) = style.bg {
        let (tr, tg, tb) = apply_day_night((r, g, b), day_tick);
        result = result.bg(Color::Rgb(tr, tg, tb));
    }
    if let Some(Color::Rgb(r, g, b)) = style.fg {
        let (tr, tg, tb) = apply_day_night_fg((r, g, b), day_tick);
        result = result.fg(Color::Rgb(tr, tg, tb));
    }
    result
}

/// Small vim-mark badge for a tile, if a mark is set here.
fn mark_badge_at(
    marks: &[(char, usize, usize)],
    x: usize,
    y: usize,
) -> Option<(char, Style)> {
    marks
        .iter()
        .find(|&&(_, mx, my)| mx == x && my == y)
        .map(|&(name, _, _)| (name, mark_badge_style()))
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
