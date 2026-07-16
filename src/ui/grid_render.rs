use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::Frame;

use crate::app::{AppState, Mode};
use crate::ecs::components::{
    EntityKind, FacingComponent, MultiTile, PartOfBuilding, Position, Processing,
};
use crate::render::animations::flash_spark;
use crate::render::colors::{
    apply_day_night, apply_day_night_fg, blend_trail, day_night_multiplier,
};
use crate::render::glyphs::{self, MachineState};
use crate::render::highlights::{
    self, flash_highlight, highlight_style, mark_badge_style, HighlightType,
};
use crate::render::sprites;
use crate::render::viewport::Viewport;
use crate::resources::{EntityType, Facing};

/// Render the game grid into the given area.
///
/// The renderer is scale-aware: the `Viewport` carries a tile scale S in
/// 1..=3 and one map tile covers (2S x S) terminal cells.
///
/// - S = 1 uses the compact per-tile path (2 cells per tile) that huge maps
///   need.
/// - S >= 2 uses the sprite painter in `render::sprites`: each building is
///   painted as ONE multi-cell sprite across its whole footprint (anchor
///   driven), belts get lane rails + marching chevrons, and cargo rides the
///   lanes as bold 2-cell chips.
///
/// At every scale the FULL rect is painted: terrain dither fills empty
/// tiles, centering pads and the area beyond the map bounds — zero untouched
/// cells. Rendering never mutates game state.
pub fn render_grid(frame: &mut Frame, area: Rect, app: &AppState, viewport: &Viewport) {
    if area.width < 2 || area.height == 0 {
        return;
    }
    let scale = viewport.scale.clamp(1, 3);
    let buf = frame.buffer_mut();

    // Pass 0 (all scales): textured ground across the ENTIRE rect so no cell
    // is ever left untouched, whatever the map/viewport geometry.
    fill_terrain(buf, area, app, viewport, scale);

    if scale == 1 {
        render_compact(buf, area, app, viewport);
    } else {
        render_scaled(buf, area, app, viewport, scale);
    }
}

// ===========================================================================
// Terrain dither (pass 0)
// ===========================================================================

/// Fill every cell of the rect with textured ground: in-map cells use the
/// tile's terrain palette, cells beyond the map (and centering pads) use a
/// dark void tone with the same subtle dither. When `scale == 1` the
/// day/night tint is applied here directly (the compact path re-tints the
/// cells it overdraws); at S >= 2 the scaled path runs a whole-rect tint
/// pass afterwards, so this stays raw.
fn fill_terrain(buf: &mut Buffer, area: Rect, app: &AppState, viewport: &Viewport, scale: u8) {
    let s = scale as i32;
    let ts_w = 2 * s;
    let ts_h = s;
    let pad_l = viewport.pad_left as i32;
    let pad_t = viewport.pad_top as i32;
    let tint_here = scale == 1;
    let day_tick = app.day_tick;

    for cy in area.y..area.y + area.height {
        for cx in area.x..area.x + area.width {
            let rx = cx as i32 - area.x as i32 - pad_l;
            let ry = cy as i32 - area.y as i32 - pad_t;

            let mut in_map = false;
            let mut terrain = crate::map::terrain::Terrain::Ground;
            let mut gx = cx as i32;
            let mut gy = cy as i32;
            if rx >= 0 && ry >= 0 {
                let tx = viewport.offset_x + (rx / ts_w) as usize;
                let ty = viewport.offset_y + (ry / ts_h) as usize;
                if tx < app.map.width && ty < app.map.height {
                    in_map = true;
                    terrain = app.map.terrain_at(tx, ty);
                    // Map-anchored cell coords so the texture scrolls with
                    // the world instead of swimming under it.
                    gx = rx + viewport.offset_x as i32 * ts_w;
                    gy = ry + viewport.offset_y as i32 * ts_h;
                }
            }

            let (glyph, mut fg, mut bg) = if in_map {
                let g = sprites::dither_glyph(gx, gy);
                let base_fg = terrain.fg_color();
                // Two-tone: half the specks are dimmer.
                let fg = if sprites::hash2(gx, gy) & 8 == 0 {
                    crate::render::colors::dim_color(base_fg, 0.65)
                } else {
                    base_fg
                };
                let bg = terrain.bg_color().unwrap_or((14, 16, 20));
                (g, fg, bg)
            } else {
                // Void beyond the map: darker and sparser.
                let g = match sprites::hash2(gx, gy) % 17 {
                    0 => '·',
                    1 => '.',
                    _ => ' ',
                };
                (g, (38, 42, 52), (11, 13, 17))
            };

            if tint_here {
                bg = apply_day_night(bg, day_tick);
                fg = apply_day_night_fg(fg, day_tick);
            }

            let cell = &mut buf[(cx, cy)];
            cell.set_char(glyph);
            cell.set_style(
                Style::default()
                    .fg(Color::Rgb(fg.0, fg.1, fg.2))
                    .bg(Color::Rgb(bg.0, bg.1, bg.2)),
            );
        }
    }
}

// ===========================================================================
// Scaled path (S >= 2): sprites + overlays + tint + crisp highlights
// ===========================================================================

fn render_scaled(buf: &mut Buffer, area: Rect, app: &AppState, viewport: &Viewport, scale: u8) {
    let s = scale as i32;
    let ts_w = 2 * s;
    let ts_h = s;
    let ox0 = area.x as i32 + viewport.pad_left as i32;
    let oy0 = area.y as i32 + viewport.pad_top as i32;
    let frame_counter = app.animations.frame_counter;

    // How many tile columns/rows can possibly intersect the rect (derived
    // from the rect itself so mismatched viewport dims can't leave holes).
    let tiles_x = (area.width as usize).div_ceil(ts_w as usize) + 1;
    let tiles_y = (area.height as usize).div_ceil(ts_h as usize) + 1;

    // ---- Pass 1: building/belt sprites, one per anchor -------------------
    let mut painted: std::collections::HashSet<hecs::Entity> = std::collections::HashSet::new();
    for ty in 0..tiles_y {
        let map_y = viewport.offset_y + ty;
        if map_y >= app.map.height {
            break;
        }
        for tx in 0..tiles_x {
            let map_x = viewport.offset_x + tx;
            if map_x >= app.map.width {
                break;
            }
            let Some(ent) = app.map.entity_at(map_x, map_y) else {
                continue;
            };
            // Resolve to the anchor entity for multi-tile buildings.
            let (anchor, ax, ay) = if let Ok(pob) = app.world.get::<&PartOfBuilding>(ent) {
                let anchor = pob.anchor;
                let pos = app
                    .world
                    .get::<&Position>(anchor)
                    .map(|p| (p.x, p.y))
                    .unwrap_or((map_x, map_y));
                (anchor, pos.0, pos.1)
            } else {
                (ent, map_x, map_y)
            };
            if !painted.insert(anchor) {
                continue;
            }

            let entity_type = app
                .world
                .get::<&EntityKind>(anchor)
                .map(|k| k.kind)
                .unwrap_or(EntityType::Wall);
            let facing = app
                .world
                .get::<&FacingComponent>(anchor)
                .map(|f| f.facing)
                .unwrap_or(Facing::Right);
            let (fw, fh) = app
                .world
                .get::<&MultiTile>(anchor)
                .map(|m| (m.width, m.height))
                .unwrap_or((1, 1));
            let state = match app.world.get::<&Processing>(anchor) {
                Ok(proc) if proc.is_processing() => MachineState::Processing,
                _ => MachineState::Idle,
            };

            let is_belt = matches!(
                entity_type,
                EntityType::BasicBelt | EntityType::FastBelt | EntityType::ExpressBelt
            );
            // Belts never draw sockets; skip the footprint allocation.
            let ports = if is_belt {
                Vec::new()
            } else {
                crate::map::multitile::building_footprint(entity_type)
                    .rotate_to(facing)
                    .ports
            };

            let ctx = sprites::SpriteCtx {
                entity_type,
                facing,
                scale,
                frame: frame_counter,
                state,
                fw,
                fh,
            };
            let origin_x = ox0 + (ax as i32 - viewport.offset_x as i32) * ts_w;
            let origin_y = oy0 + (ay as i32 - viewport.offset_y as i32) * ts_h;
            sprites::paint_building(buf, area, origin_x, origin_y, &ctx, &ports);
        }
    }

    // ---- Pass 2: per-tile overlays (cargo chips, trails, marks, particles)
    let marks = app.marks.list();
    for ty in 0..tiles_y {
        let map_y = viewport.offset_y + ty;
        if map_y >= app.map.height {
            break;
        }
        for tx in 0..tiles_x {
            let map_x = viewport.offset_x + tx;
            if map_x >= app.map.width {
                break;
            }
            let tox = ox0 + tx as i32 * ts_w;
            let toy = oy0 + ty as i32 * ts_h;

            if let Some(resource) = app.map.resource_at(map_x, map_y) {
                sprites::paint_cargo_chip(buf, area, tox, toy, scale, resource);
            } else if let Some(trail) = app.trails.get_at(map_x, map_y) {
                // Fading wake: blend the trail color into the tile's bg.
                let intensity = trail.intensity();
                tile_cells(buf, area, tox, toy, ts_w, ts_h, |cell| {
                    if let Color::Rgb(r, g, b) = cell.bg {
                        let (nr, ng, nb) = blend_trail((r, g, b), trail.color, intensity);
                        cell.bg = Color::Rgb(nr, ng, nb);
                    }
                });
            }

            // Mark badge in the tile's top-right corner cell.
            if let Some(&(name, _, _)) =
                marks.iter().find(|&&(_, mx, my)| mx == map_x && my == map_y)
            {
                put_cell(buf, area, tox + ts_w - 1, toy, name, mark_badge_style());
            }

            // Particle at the tile center.
            if let Some(p) = app.particles.get_at(map_x, map_y) {
                let style = Style::default()
                    .fg(Color::Rgb(p.fg.0, p.fg.1, p.fg.2))
                    .add_modifier(Modifier::BOLD);
                put_cell_keep_bg(buf, area, tox + s - 1, toy + s / 2, p.glyph, style);
            }
        }
    }

    // ---- Pass 3: day/night tint across the whole rect --------------------
    let (mr, mg, mb) = day_night_multiplier(app.day_tick);
    if (mr, mg, mb) != (1.0, 1.0, 1.0) {
        let soften = |m: f64| m + (1.0 - m) * 0.55;
        let (fr, fgm, fb) = (soften(mr), soften(mg), soften(mb));
        for cy in area.y..area.y + area.height {
            for cx in area.x..area.x + area.width {
                let cell = &mut buf[(cx, cy)];
                if let Color::Rgb(r, g, b) = cell.bg {
                    cell.bg = Color::Rgb(
                        (r as f64 * mr).min(255.0) as u8,
                        (g as f64 * mg).min(255.0) as u8,
                        (b as f64 * mb).min(255.0) as u8,
                    );
                }
                if let Color::Rgb(r, g, b) = cell.fg {
                    cell.fg = Color::Rgb(
                        (r as f64 * fr).min(255.0) as u8,
                        (g as f64 * fgm).min(255.0) as u8,
                        (b as f64 * fb).min(255.0) as u8,
                    );
                }
            }
        }
    }

    // ---- Pass 4: crisp highlights (applied after tint) --------------------
    let to_tile_origin = |x: usize, y: usize| -> Option<(i32, i32)> {
        if x < viewport.offset_x || y < viewport.offset_y {
            return None;
        }
        let tx = x - viewport.offset_x;
        let ty = y - viewport.offset_y;
        if tx >= tiles_x || ty >= tiles_y || x >= app.map.width || y >= app.map.height {
            return None;
        }
        Some((ox0 + tx as i32 * ts_w, oy0 + ty as i32 * ts_h))
    };

    // 4a. Search matches: gold corner frames (bright for the current one).
    if app.search.has_pattern() {
        let current = if app.search.matches.is_empty() {
            None
        } else {
            Some(app.search.current_match)
        };
        for (i, &(sx, sy)) in app.search.matches.iter().enumerate() {
            if Some(i) == current {
                continue;
            }
            if let Some((tox, toy)) = to_tile_origin(sx, sy) {
                frame_tile(buf, area, tox, toy, ts_w, ts_h, (170, 130, 25), false);
            }
        }
        if let Some(i) = current {
            if let Some(&(sx, sy)) = app.search.matches.get(i) {
                if let Some((tox, toy)) = to_tile_origin(sx, sy) {
                    frame_tile(buf, area, tox, toy, ts_w, ts_h, (255, 190, 40), true);
                }
            }
        }
    }

    // 4b. Visual selection: tile-sized tint.
    let sel_bg = match highlight_style(HighlightType::VisualSelection).bg {
        Some(Color::Rgb(r, g, b)) => (r, g, b),
        _ => (72, 62, 110),
    };
    for (vx, vy) in app.visual_selection() {
        if let Some((tox, toy)) = to_tile_origin(vx, vy) {
            tile_cells(buf, area, tox, toy, ts_w, ts_h, |cell| {
                let base = match cell.bg {
                    Color::Rgb(r, g, b) => (r, g, b),
                    _ => (14, 16, 20),
                };
                let (r, g, b) = blend_trail(base, sel_bg, 0.6);
                cell.bg = Color::Rgb(r, g, b);
            });
        }
    }

    // 4c. Flashes: bright tile fill + spark glyph.
    for flash in app.animations.flashes() {
        if let Some((tox, toy)) = to_tile_origin(flash.x, flash.y) {
            let hstyle = highlight_style(flash_highlight(flash.kind));
            if let Some(bg) = hstyle.bg {
                tile_cells(buf, area, tox, toy, ts_w, ts_h, |cell| {
                    cell.bg = bg;
                });
            }
            let (ch, fg) = flash_spark(flash.kind, flash.frames_remaining);
            let style = Style::default()
                .fg(Color::Rgb(fg.0, fg.1, fg.2))
                .add_modifier(Modifier::BOLD);
            put_cell_keep_bg(buf, area, tox + s - 1, toy + s / 2, ch, style);
        }
    }

    // 4d. Cursor: tile-sized bracket frame + inverse core, always on top.
    if let Some((tox, toy)) = to_tile_origin(app.cursor_x, app.cursor_y) {
        let is_insert = app.mode == Mode::Insert;
        let color = if is_insert { (120, 235, 130) } else { (240, 238, 225) };
        let bracket = Style::default()
            .fg(Color::Rgb(color.0, color.1, color.2))
            .add_modifier(Modifier::BOLD);
        put_cell_keep_bg(buf, area, tox, toy, '⌜', bracket);
        put_cell_keep_bg(buf, area, tox + ts_w - 1, toy, '⌝', bracket);
        put_cell_keep_bg(buf, area, tox, toy + ts_h - 1, '⌞', bracket);
        put_cell_keep_bg(buf, area, tox + ts_w - 1, toy + ts_h - 1, '⌟', bracket);
        // Inverse core: the 2 center cells take the cursor's ink-on-paper so
        // it reads on top of any sprite at any zoom.
        let core = highlight_style(if is_insert {
            HighlightType::CursorInsert
        } else {
            HighlightType::Cursor
        });
        for dx in 0..2 {
            let x = tox + s - 1 + dx;
            let y = toy + ts_h / 2;
            if in_area(area, x, y) {
                let cell = &mut buf[(x as u16, y as u16)];
                if let (Some(fg), Some(bg)) = (core.fg, core.bg) {
                    cell.fg = fg;
                    cell.bg = bg;
                }
                cell.modifier.insert(Modifier::BOLD);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scaled-path cell helpers
// ---------------------------------------------------------------------------

fn in_area(area: Rect, x: i32, y: i32) -> bool {
    x >= area.x as i32
        && y >= area.y as i32
        && x < (area.x + area.width) as i32
        && y < (area.y + area.height) as i32
}

fn put_cell(buf: &mut Buffer, area: Rect, x: i32, y: i32, ch: char, style: Style) {
    if in_area(area, x, y) {
        let cell = &mut buf[(x as u16, y as u16)];
        cell.set_char(ch);
        cell.set_style(style);
    }
}

/// Put a glyph but keep the cell's existing background (overlay on sprite).
fn put_cell_keep_bg(buf: &mut Buffer, area: Rect, x: i32, y: i32, ch: char, style: Style) {
    if in_area(area, x, y) {
        let cell = &mut buf[(x as u16, y as u16)];
        cell.set_char(ch);
        if let Some(fg) = style.fg {
            cell.fg = fg;
        }
        cell.modifier.insert(style.add_modifier);
    }
}

/// Run a closure over every cell of a tile block, clipped to the area.
fn tile_cells<F: FnMut(&mut ratatui::buffer::Cell)>(
    buf: &mut Buffer,
    area: Rect,
    tox: i32,
    toy: i32,
    ts_w: i32,
    ts_h: i32,
    mut f: F,
) {
    for y in toy..toy + ts_h {
        for x in tox..tox + ts_w {
            if in_area(area, x, y) {
                f(&mut buf[(x as u16, y as u16)]);
            }
        }
    }
}

/// Draw a corner frame around a tile block (search hit marker). `bright`
/// additionally underlays a tinted bed so the current match pops.
fn frame_tile(
    buf: &mut Buffer,
    area: Rect,
    tox: i32,
    toy: i32,
    ts_w: i32,
    ts_h: i32,
    color: (u8, u8, u8),
    bright: bool,
) {
    if bright {
        tile_cells(buf, area, tox, toy, ts_w, ts_h, |cell| {
            let base = match cell.bg {
                Color::Rgb(r, g, b) => (r, g, b),
                _ => (14, 16, 20),
            };
            let (r, g, b) = blend_trail(base, (120, 90, 15), 0.55);
            cell.bg = Color::Rgb(r, g, b);
        });
    }
    let style = Style::default()
        .fg(Color::Rgb(color.0, color.1, color.2))
        .add_modifier(Modifier::BOLD);
    put_cell_keep_bg(buf, area, tox, toy, '⌜', style);
    put_cell_keep_bg(buf, area, tox + ts_w - 1, toy, '⌝', style);
    put_cell_keep_bg(buf, area, tox, toy + ts_h - 1, '⌞', style);
    put_cell_keep_bg(buf, area, tox + ts_w - 1, toy + ts_h - 1, '⌟', style);
}

// ===========================================================================
// Compact path (S = 1) — today's 2-cells-per-tile renderer, unchanged
// ===========================================================================

fn render_compact(buf: &mut Buffer, area: Rect, app: &AppState, viewport: &Viewport) {
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
                        (proc_char, base_style.add_modifier(Modifier::BOLD))
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
                            glyphs::belt_animated_glyph(
                                info.entity_type,
                                info.facing,
                                frame_counter,
                            ),
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
                        (
                            glyphs::resource_glyph(resource),
                            ground_resource_style(resource, s),
                        )
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
                overlay1 =
                    Some(style1.fg(Color::Rgb(fg.0, fg.1, fg.2)).add_modifier(Modifier::BOLD));
            }
            if let Some(s) = overlay1 {
                style1 = s;
            }

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
                flashes,
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
