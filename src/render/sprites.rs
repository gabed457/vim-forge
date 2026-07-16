//! Multi-cell sprite painter for the adaptive-zoom renderer (scale >= 2).
//!
//! At tile scale S, one map tile covers 2S columns x S rows of terminal
//! cells. A building whose (rotated) footprint is fw x fh tiles is painted as
//! ONE sprite spanning fw*2S x fh*S cells, anchored at the footprint's
//! top-left tile. Painting goes straight into the ratatui `Buffer` through a
//! clipping `Painter`, so partially visible sprites at the grid edge work for
//! free and no intermediate strings are allocated.
//!
//! Everything here is a pure function of (entity type, facing, scale, frame,
//! state): rendering never mutates game state.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use crate::map::multitile::PortDefinition;
use crate::render::colors::dim_color;
use crate::render::glyphs::{building_fg, building_glow_bg, MachineState};
use crate::resources::{EntityType, Facing, Resource};

// ---------------------------------------------------------------------------
// Painter: clipped cell writer
// ---------------------------------------------------------------------------

/// Writes characters into a Buffer relative to a sprite origin, clipped to a
/// rect. The origin may lie outside the clip rect (sprite partially visible).
pub struct Painter<'a> {
    buf: &'a mut Buffer,
    clip: Rect,
    /// Absolute cell position of the sprite's top-left corner (can be
    /// negative when the sprite is partially scrolled off).
    ox: i32,
    oy: i32,
}

impl<'a> Painter<'a> {
    pub fn new(buf: &'a mut Buffer, clip: Rect, ox: i32, oy: i32) -> Self {
        Painter { buf, clip, ox, oy }
    }

    /// Put a single char at sprite-local (lx, ly).
    pub fn put(&mut self, lx: i32, ly: i32, ch: char, style: Style) {
        let x = self.ox + lx;
        let y = self.oy + ly;
        if x < self.clip.x as i32
            || y < self.clip.y as i32
            || x >= (self.clip.x + self.clip.width) as i32
            || y >= (self.clip.y + self.clip.height) as i32
        {
            return;
        }
        let cell = &mut self.buf[(x as u16, y as u16)];
        cell.set_char(ch);
        cell.set_style(style);
    }

    /// Fill a sprite-local box with one char.
    pub fn fill(&mut self, lx: i32, ly: i32, w: i32, h: i32, ch: char, style: Style) {
        for y in ly..ly + h {
            for x in lx..lx + w {
                self.put(x, y, ch, style);
            }
        }
    }

    /// Write a string horizontally starting at sprite-local (lx, ly).
    pub fn text(&mut self, lx: i32, ly: i32, s: &str, style: Style) {
        for (i, ch) in s.chars().enumerate() {
            self.put(lx + i as i32, ly, ch, style);
        }
    }
}

// ---------------------------------------------------------------------------
// Style helpers
// ---------------------------------------------------------------------------

fn rgb(c: (u8, u8, u8)) -> Color {
    Color::Rgb(c.0, c.1, c.2)
}

/// Tier accent colors for the generic chassis border (tier 0..=5).
fn tier_color(tier: u8) -> (u8, u8, u8) {
    match tier {
        0 => (150, 140, 120), // extractor: earthy bronze-gray
        1 => (170, 170, 180), // basic: steel
        2 => (90, 190, 210),  // t2: teal
        3 => (170, 120, 230), // t3: violet
        4 => (240, 150, 60),  // t4: orange
        _ => (255, 210, 80),  // t5+: gold
    }
}

/// Body background for a building: its color at ~12% + a floor tone.
fn body_bg(et: EntityType) -> (u8, u8, u8) {
    let g = building_glow_bg(et);
    (g.0.max(16), g.1.max(18), g.2.max(22))
}

/// Foreground color for the machine at a given state, pulsing while
/// processing (same rhythm as the compact renderer).
fn state_fg(et: EntityType, state: MachineState, frame: u32) -> (u8, u8, u8) {
    let base = building_fg(et);
    match state {
        MachineState::Processing => {
            let pulse = ((frame % 6) as f32 / 6.0 * std::f32::consts::PI).sin();
            (
                (base.0 as f32 + pulse * 50.0).clamp(0.0, 255.0) as u8,
                (base.1 as f32 + pulse * 50.0).clamp(0.0, 255.0) as u8,
                (base.2 as f32 + pulse * 50.0).clamp(0.0, 255.0) as u8,
            )
        }
        _ => dim_color(base, 0.85),
    }
}

fn body_style(et: EntityType) -> Style {
    Style::default().bg(rgb(body_bg(et)))
}

fn border_style(et: EntityType, state: MachineState, frame: u32) -> Style {
    let tier = tier_color(et.tier());
    let fg = match state {
        MachineState::Processing => state_fg(et, state, frame),
        _ => (
            ((tier.0 as u16 + building_fg(et).0 as u16) / 2) as u8,
            ((tier.1 as u16 + building_fg(et).1 as u16) / 2) as u8,
            ((tier.2 as u16 + building_fg(et).2 as u16) / 2) as u8,
        ),
    };
    Style::default().fg(rgb(fg)).bg(rgb(body_bg(et)))
}

fn glyph_style(et: EntityType, state: MachineState, frame: u32) -> Style {
    Style::default()
        .fg(rgb(state_fg(et, state, frame)))
        .bg(rgb(body_bg(et)))
        .add_modifier(Modifier::BOLD)
}

/// Bright socket style for ports.
fn socket_style(et: EntityType, out: bool) -> Style {
    let fg = if out { (140, 255, 170) } else { (255, 232, 150) };
    let glow = building_glow_bg(et);
    Style::default()
        .fg(rgb(fg))
        .bg(Color::Rgb(glow.0.max(14), glow.1.max(14), glow.2.max(14)))
        .add_modifier(Modifier::BOLD)
}

// ---------------------------------------------------------------------------
// Sprite context
// ---------------------------------------------------------------------------

/// Everything the painter needs to draw one building sprite.
pub struct SpriteCtx {
    pub entity_type: EntityType,
    pub facing: Facing,
    /// Tile scale (2 or 3; 1 uses the compact path, not this module).
    pub scale: u8,
    /// Animation frame counter.
    pub frame: u32,
    pub state: MachineState,
    /// ROTATED footprint size in tiles (as placed on the map).
    pub fw: usize,
    pub fh: usize,
}

impl SpriteCtx {
    /// Sprite size in cells.
    fn cell_dims(&self) -> (i32, i32) {
        let s = self.scale as i32;
        (self.fw as i32 * 2 * s, self.fh as i32 * s)
    }

    /// Whether this sprite gets a bottom name strip (S=3 and big enough).
    fn has_name_strip(&self) -> bool {
        let (w, h) = self.cell_dims();
        self.scale >= 3 && h >= 4 && w >= 6
    }

    /// Lowest interior row usable by bottom-anchored decorations (fireboxes,
    /// brick courses, smokestacks) without colliding with the name strip.
    fn interior_bottom(&self) -> i32 {
        let (_, h) = self.cell_dims();
        if self.has_name_strip() {
            h - 3
        } else {
            h - 2
        }
    }
}

/// Cheap deterministic 2D hash for texture dithering.
#[inline]
pub fn hash2(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B9) ^ (y as u32).wrapping_mul(0x85EB_CA6B);
    h ^= h >> 13;
    h = h.wrapping_mul(0xC2B2_AE35);
    h ^ (h >> 16)
}

// ---------------------------------------------------------------------------
// Entry point: paint one building sprite
// ---------------------------------------------------------------------------

/// Paint the full sprite for a building whose footprint's top-left tile sits
/// at absolute cell (origin_x, origin_y). `ports` must already be rotated to
/// the building's facing (offsets in rotated-footprint tile space).
pub fn paint_building(
    buf: &mut Buffer,
    clip: Rect,
    origin_x: i32,
    origin_y: i32,
    ctx: &SpriteCtx,
    ports: &[PortDefinition],
) {
    let mut p = Painter::new(buf, clip, origin_x, origin_y);
    use EntityType::*;
    match ctx.entity_type {
        BasicBelt | FastBelt | ExpressBelt => {
            paint_belt(&mut p, ctx);
            return; // belts have no chassis and no port sockets
        }
        Wall | ReinforcedWall => {
            paint_wall(&mut p, ctx);
            return;
        }
        Pipe | PipeJunction | GasPipeline => {
            paint_pipe(&mut p, ctx);
            return;
        }
        UndergroundEntrance | UndergroundExit => {
            paint_underground(&mut p, ctx);
            return;
        }
        _ => {}
    }

    // Chassis (border + body fill), then a type-specific interior, then the
    // name strip, then port sockets on top.
    paint_chassis(&mut p, ctx);
    paint_interior(&mut p, ctx);
    if ctx.scale >= 3 {
        paint_name_strip(&mut p, ctx);
    }
    paint_ports(&mut p, ctx, ports);
}

// ---------------------------------------------------------------------------
// Chassis
// ---------------------------------------------------------------------------

fn paint_chassis(p: &mut Painter, ctx: &SpriteCtx) {
    let (w, h) = ctx.cell_dims();
    let et = ctx.entity_type;
    let bs = border_style(et, ctx.state, ctx.frame);
    let body = body_style(et);

    // Body fill first.
    p.fill(0, 0, w, h, ' ', body);

    // Rounded border.
    for x in 1..w - 1 {
        p.put(x, 0, '─', bs);
        p.put(x, h - 1, '─', bs);
    }
    for y in 1..h - 1 {
        p.put(0, y, '│', bs);
        p.put(w - 1, y, '│', bs);
    }
    p.put(0, 0, '╭', bs);
    p.put(w - 1, 0, '╮', bs);
    p.put(0, h - 1, '╰', bs);
    p.put(w - 1, h - 1, '╯', bs);
}

/// Centered 2-char identity glyph for the generic chassis.
fn machine_glyph2(et: EntityType) -> [char; 2] {
    use EntityType::*;
    match et {
        Assembler | AdvancedAssembler | PrecisionAssembler | Megassembler => ['⚙', ' '],
        ResearchLab | AdvancedLab | QuantumLab | SingularityLab => ['⚗', ' '],
        Smelter | CokeFurnace => ['▲', ' '],
        Kiln | Boiler | GeothermalPlant | GeothermalTap => ['♨', ' '],
        OutputBin | Warehouse | SiloHopper => ['▣', ' '],
        Splitter => ['⋔', ' '],
        Merger => ['⋎', ' '],
        CoalGenerator | GasGenerator => ['⚡', ' '],
        SolarArray => ['☀', ' '],
        WindTurbine => ['╀', ' '],
        NuclearReactor | EnrichmentCascade | UraniumMine => ['☢', ' '],
        FusionReactor => ['✳', ' '],
        WaterPump | PumpStation => ['≋', ' '],
        FluidTank | CryoTank => ['◍', ' '],
        OilWell => ['⊼', ' '],
        TrainStation => ['▬', ' '],
        DronePort => ['✈', ' '],
        Turret => ['⊕', ' '],
        ShieldGenerator => ['◉', ' '],
        RecyclingPlant => ['♻', ' '],
        WasteDump => ['✗', ' '],
        SpaceElevatorBase | RocketAssembly => ['▲', '▲'],
        DysonSwarmLauncher => ['☀', '☀'],
        WarpGateFrame => ['◈', '◈'],
        _ => name_initials(et),
    }
}

/// Fallback glyph: initials of the first two words of the entity name.
fn name_initials(et: EntityType) -> [char; 2] {
    let name = et.name();
    let mut it = name.split_whitespace();
    let a = it
        .next()
        .and_then(|w| w.chars().next())
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or('?');
    let b = it
        .next()
        .and_then(|w| w.chars().next())
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or(' ');
    [a, b]
}

/// Paint the centered identity glyph (used by the generic interior).
fn paint_center_glyph(p: &mut Painter, ctx: &SpriteCtx) {
    let (w, h) = ctx.cell_dims();
    let g = machine_glyph2(ctx.entity_type);
    let gs = glyph_style(ctx.entity_type, ctx.state, ctx.frame);
    let cx = w / 2 - 1;
    let cy = h / 2;
    p.put(cx, cy, g[0], gs);
    p.put(cx + 1, cy, g[1], gs);
}

/// Bottom name strip (scale 3 only, and only when the sprite is tall enough
/// to keep an interior row free: needs H >= 4).
fn paint_name_strip(p: &mut Painter, ctx: &SpriteCtx) {
    let (w, h) = ctx.cell_dims();
    if h < 4 || w < 6 {
        return;
    }
    let name = ctx.entity_type.name();
    let avail = (w - 2) as usize;
    let strip = Style::default()
        .fg(rgb(dim_color(building_fg(ctx.entity_type), 0.75)))
        .bg(rgb(dim_color(body_bg(ctx.entity_type), 0.7)));
    let y = h - 2;
    // Strip bed.
    p.fill(1, y, w - 2, 1, ' ', strip);
    let len = name.chars().count().min(avail);
    let start = 1 + ((avail - len) / 2) as i32;
    for (i, ch) in name.chars().take(len).enumerate() {
        p.put(start + i as i32, y, ch.to_ascii_uppercase(), strip);
    }
}

// ---------------------------------------------------------------------------
// Port sockets
// ---------------------------------------------------------------------------

fn paint_ports(p: &mut Painter, ctx: &SpriteCtx, ports: &[PortDefinition]) {
    let s = ctx.scale as i32;
    for port in ports {
        let tx = port.offset_x as i32; // tile within rotated footprint
        let ty = port.offset_y as i32;
        // Cell block of that tile.
        let bx = tx * 2 * s;
        let by = ty * s;
        let out = port.port_type.is_output();
        let waste = port.port_type.is_waste();
        let style = if waste {
            Style::default()
                .fg(Color::Rgb(150, 130, 90))
                .bg(rgb(body_bg(ctx.entity_type)))
                .add_modifier(Modifier::BOLD)
        } else {
            socket_style(ctx.entity_type, out)
        };
        // Socket char: inputs point INTO the sprite; outputs are ●.
        let (cx, cy, ch) = match port.direction {
            Facing::Left => (bx, by + s / 2, if out { '●' } else { '▸' }),
            Facing::Right => (bx + 2 * s - 1, by + s / 2, if out { '●' } else { '◂' }),
            Facing::Up => (bx + s, by, if out { '●' } else { '▾' }),
            Facing::Down => (
                bx + s,
                by + s - 1,
                if waste {
                    '▿'
                } else if out {
                    '●'
                } else {
                    '▴'
                },
            ),
        };
        p.put(cx, cy, ch, style);
    }
}

// ---------------------------------------------------------------------------
// Type-specific interiors
// ---------------------------------------------------------------------------

fn paint_interior(p: &mut Painter, ctx: &SpriteCtx) {
    use EntityType::*;
    match ctx.entity_type {
        OreDeposit | CopperDeposit | CoalDeposit | StoneQuarry | UraniumMine | SandExtractor
        | SulfurMine | BauxiteMine | LithiumExtractor | RareEarthExtractor | BiomassHarvester => {
            paint_deposit(p, ctx)
        }
        Smelter | CokeFurnace => paint_smelter(p, ctx),
        Kiln | Boiler => paint_kiln(p, ctx),
        Assembler | AdvancedAssembler | PrecisionAssembler | Megassembler => {
            paint_assembler(p, ctx)
        }
        OutputBin | SiloHopper | Warehouse => paint_bin(p, ctx),
        Splitter => paint_splitter(p, ctx, false),
        Merger => paint_splitter(p, ctx, true),
        ResearchLab | AdvancedLab | QuantumLab | SingularityLab => paint_lab(p, ctx),
        CoalGenerator | GasGenerator => paint_generator(p, ctx),
        _ => paint_center_glyph(p, ctx),
    }
}

/// Rock face with ore flecks in the deposit's color.
fn paint_deposit(p: &mut Painter, ctx: &SpriteCtx) {
    let (w, h) = ctx.cell_dims();
    let base = building_fg(ctx.entity_type);
    let rock = Style::default()
        .fg(Color::Rgb(95, 88, 78))
        .bg(rgb(body_bg(ctx.entity_type)));
    let fleck = Style::default()
        .fg(rgb(state_fg(ctx.entity_type, ctx.state, ctx.frame)))
        .bg(rgb(body_bg(ctx.entity_type)))
        .add_modifier(Modifier::BOLD);
    let _ = base;
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let hsh = hash2(x, y.wrapping_mul(31));
            match hsh % 7 {
                0 => p.put(x, y, '▓', rock),
                1 | 2 => p.put(x, y, '▒', rock),
                3 => p.put(x, y, '░', rock),
                4 => p.put(x, y, '◦', fleck),
                5 => p.put(x, y, '•', fleck),
                _ => {}
            }
        }
    }
    paint_center_glyph(p, ctx);
}

/// Furnace body with a cycling flame row.
fn paint_smelter(p: &mut Painter, ctx: &SpriteCtx) {
    let (w, h) = ctx.cell_dims();
    let hot = Style::default()
        .fg(Color::Rgb(255, 150, 40))
        .bg(Color::Rgb(60, 18, 8))
        .add_modifier(Modifier::BOLD);
    let ember = Style::default()
        .fg(Color::Rgb(200, 70, 30))
        .bg(Color::Rgb(42, 14, 8));
    // Firebox: bottom interior row(s) glow.
    let fy = ctx.interior_bottom();
    for x in 1..w - 1 {
        let flame_on = matches!(ctx.state, MachineState::Processing);
        let phase = (ctx.frame / 3 + x as u32) % 2 == 0;
        let ch = if flame_on {
            if phase { '▲' } else { '△' }
        } else if (x + fy) % 2 == 0 {
            '▁'
        } else {
            ' '
        };
        p.put(x, fy, ch, if flame_on { hot } else { ember });
    }
    // Furnace mouth above the fire.
    let mouth = Style::default()
        .fg(rgb(state_fg(ctx.entity_type, ctx.state, ctx.frame)))
        .bg(Color::Rgb(30, 16, 12))
        .add_modifier(Modifier::BOLD);
    let my = h / 2 - 1;
    let mx = w / 2 - 1;
    p.put(mx, my, '▐', mouth);
    p.put(mx + 1, my, '▌', mouth);
}

/// Kiln: brick arch + heat shimmer.
fn paint_kiln(p: &mut Painter, ctx: &SpriteCtx) {
    let (w, h) = ctx.cell_dims();
    let brick = Style::default()
        .fg(Color::Rgb(160, 100, 50))
        .bg(rgb(body_bg(ctx.entity_type)));
    let heat = Style::default()
        .fg(Color::Rgb(255, 180, 70))
        .bg(rgb(body_bg(ctx.entity_type)))
        .add_modifier(Modifier::BOLD);
    let by = ctx.interior_bottom();
    for x in 1..w - 1 {
        p.put(x, by, if x % 2 == 0 { '▄' } else { '▆' }, brick);
    }
    let cy = h / 2 - 1;
    let shimmer = if matches!(ctx.state, MachineState::Processing) {
        if (ctx.frame / 4) % 2 == 0 { '≋' } else { '≈' }
    } else {
        '≈'
    };
    p.put(w / 2 - 1, cy, shimmer, heat);
    p.put(w / 2, cy, shimmer, heat);
}

/// Assembler: gear + sliding piston.
fn paint_assembler(p: &mut Painter, ctx: &SpriteCtx) {
    let (w, h) = ctx.cell_dims();
    let gs = glyph_style(ctx.entity_type, ctx.state, ctx.frame);
    let cy = h / 2;
    let cx = w / 2 - 1;
    // Gear "spins" while processing by alternating its satellite spokes.
    let spin = if matches!(ctx.state, MachineState::Processing) {
        if (ctx.frame / 3) % 2 == 0 { '✚' } else { '✖' }
    } else {
        '✚'
    };
    p.put(cx, cy, '⚙', gs);
    p.put(cx + 1, cy, spin, gs.add_modifier(Modifier::DIM));
    // Piston track along the row above the gear.
    if cy - 1 > 0 {
        let track = Style::default()
            .fg(Color::Rgb(120, 130, 145))
            .bg(rgb(body_bg(ctx.entity_type)));
        let head = Style::default()
            .fg(Color::Rgb(230, 235, 245))
            .bg(rgb(body_bg(ctx.entity_type)))
            .add_modifier(Modifier::BOLD);
        let x0 = 1;
        let x1 = w - 2;
        for x in x0..=x1 {
            p.put(x, cy - 1, '╌', track);
        }
        let span = (x1 - x0).max(1) as u32;
        let pos = if matches!(ctx.state, MachineState::Processing) {
            // Slide back and forth.
            let t = (ctx.frame / 2) % (2 * span);
            if t < span { t } else { 2 * span - t }
        } else {
            span / 2
        };
        p.put(x0 + pos as i32, cy - 1, '▪', head);
    }
}

/// Output bin / warehouse: slatted crate with a hatch.
fn paint_bin(p: &mut Painter, ctx: &SpriteCtx) {
    let (w, h) = ctx.cell_dims();
    let slat = Style::default()
        .fg(rgb(dim_color(building_fg(ctx.entity_type), 0.8)))
        .bg(rgb(body_bg(ctx.entity_type)));
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            if (x + y) % 2 == 0 {
                p.put(x, y, '▦', slat);
            }
        }
    }
    // Hatch on the top edge.
    let hatch = Style::default()
        .fg(Color::Rgb(255, 232, 150))
        .bg(rgb(body_bg(ctx.entity_type)))
        .add_modifier(Modifier::BOLD);
    p.put(w / 2 - 1, 0, '╾', hatch);
    p.put(w / 2, 0, '▼', hatch);
    p.put(w / 2 + 1, 0, '╼', hatch);
}

/// Splitter / merger: chevron fan.
fn paint_splitter(p: &mut Painter, ctx: &SpriteCtx, merge: bool) {
    let (w, h) = ctx.cell_dims();
    let gs = glyph_style(ctx.entity_type, ctx.state, ctx.frame);
    let dim = gs.add_modifier(Modifier::DIM);
    let cy = h / 2;
    let cx = w / 2 - 1;
    // Center hub.
    p.put(cx, cy, if merge { '⋎' } else { '⋔' }, gs);
    // Fan arms toward the corners (splitter) or from them (merger).
    let arm = if merge { '»' } else { '»' };
    let ty = h / 4;
    let by = h - 1 - h / 4;
    if ty > 0 && ty < h - 1 {
        p.put(cx + 2, ty, arm, dim);
        p.put(cx + 3, ty, arm, gs);
    }
    if by > 0 && by < h - 1 && by != ty {
        p.put(cx + 2, by, arm, dim);
        p.put(cx + 3, by, arm, gs);
    }
    if merge {
        p.put(cx - 2, cy, '»', dim);
        p.put(cx - 1, cy, '»', gs);
    } else {
        p.put(cx - 3, cy, '»', dim);
        p.put(cx - 2, cy, '»', gs);
    }
}

/// Research lab: flask with rising bubbles.
fn paint_lab(p: &mut Painter, ctx: &SpriteCtx) {
    let (w, h) = ctx.cell_dims();
    let gs = glyph_style(ctx.entity_type, ctx.state, ctx.frame);
    let bub = Style::default()
        .fg(Color::Rgb(210, 160, 255))
        .bg(rgb(body_bg(ctx.entity_type)));
    let cx = w / 2 - 1;
    let cy = h / 2;
    p.put(cx, cy, '⚗', gs);
    p.put(cx + 1, cy, '◆', gs.add_modifier(Modifier::DIM));
    // Bubbles drift upward while processing.
    if matches!(ctx.state, MachineState::Processing) && cy - 1 > 0 {
        let phase = (ctx.frame / 4) % 3;
        let bx = cx + 1 + (phase as i32 % 2);
        let by = (cy - 1 - phase as i32 % 2).max(1);
        p.put(bx, by, if phase == 1 { 'º' } else { '°' }, bub);
    }
}

/// Coal/gas generator: house body + smokestack with drifting smoke.
fn paint_generator(p: &mut Painter, ctx: &SpriteCtx) {
    let (w, h) = ctx.cell_dims();
    let gs = glyph_style(ctx.entity_type, ctx.state, ctx.frame);
    let cx = w / 2 - 1;
    let cy = h / 2;
    p.put(cx, cy, '⌂', gs);
    p.put(cx + 1, cy, '⚡', gs);
    // Smokestack near the top-right interior corner.
    let stack = Style::default()
        .fg(Color::Rgb(140, 140, 150))
        .bg(rgb(body_bg(ctx.entity_type)))
        .add_modifier(Modifier::BOLD);
    let smoke = Style::default()
        .fg(Color::Rgb(120, 120, 130))
        .bg(rgb(body_bg(ctx.entity_type)));
    let sx = w - 3;
    let by = ctx.interior_bottom();
    if h >= 3 {
        p.put(sx, by, '▮', stack);
        if matches!(ctx.state, MachineState::Processing) {
            // Smoke puffs drift up and to the right, cycling with the frame.
            let phase = (ctx.frame / 3) % 3;
            let puffs = ['░', '▒', '░'];
            for (i, ch) in puffs.iter().enumerate() {
                let py = by - 1 - i as i32;
                let px = sx + ((phase as i32 + i as i32) % 2);
                if py > 0 {
                    p.put(px, py, *ch, smoke);
                }
            }
        }
    }
}

/// Wall: full-tile brick pattern (no chassis border).
fn paint_wall(p: &mut Painter, ctx: &SpriteCtx) {
    let (w, h) = ctx.cell_dims();
    let base = building_fg(ctx.entity_type);
    let brick = Style::default()
        .fg(rgb(dim_color(base, 1.4)))
        .bg(rgb(dim_color(base, 0.55)));
    let mortar = Style::default()
        .fg(rgb(dim_color(base, 0.9)))
        .bg(rgb(dim_color(base, 0.55)));
    for y in 0..h {
        for x in 0..w {
            // Running-bond brick: offset every other row by 2 cells.
            let off = if y % 2 == 0 { 0 } else { 2 };
            let ch = if (x + off) % 4 == 3 { '▏' } else { '▬' };
            p.put(x, y, ch, if ch == '▏' { mortar } else { brick });
        }
    }
}

/// Pipe: a run through the middle of the tile, oriented by facing.
fn paint_pipe(p: &mut Painter, ctx: &SpriteCtx) {
    let (w, h) = ctx.cell_dims();
    let et = ctx.entity_type;
    let fg = Style::default()
        .fg(rgb(building_fg(et)))
        .bg(rgb(body_bg(et)))
        .add_modifier(Modifier::BOLD);
    let bed = Style::default().bg(rgb(body_bg(et)));
    p.fill(0, 0, w, h, ' ', bed);
    let horizontal = matches!(ctx.facing, Facing::Left | Facing::Right);
    if et == EntityType::PipeJunction {
        // Cross.
        let cy = h / 2;
        let cx = w / 2;
        for x in 0..w {
            p.put(x, cy, '━', fg);
        }
        for y in 0..h {
            p.put(cx, y, '┃', fg);
        }
        p.put(cx, cy, '╋', fg);
    } else if horizontal {
        let cy = h / 2;
        for x in 0..w {
            p.put(x, cy, '━', fg);
        }
        p.put(w / 2, cy, '◎', fg);
    } else {
        let cx = w / 2;
        for y in 0..h {
            p.put(cx, y, '┃', fg);
        }
        p.put(cx, h / 2, '◎', fg);
    }
    if et == EntityType::GasPipeline {
        p.put(w / 2 - 1, h / 2, '°', fg);
    }
}

/// Underground belt entrance/exit: ramp arrow + tunnel shading.
fn paint_underground(p: &mut Painter, ctx: &SpriteCtx) {
    let (w, h) = ctx.cell_dims();
    let entrance = ctx.entity_type == EntityType::UndergroundEntrance;
    let fg = Style::default()
        .fg(Color::Rgb(220, 220, 230))
        .bg(Color::Rgb(24, 26, 32))
        .add_modifier(Modifier::BOLD);
    let shade = Style::default()
        .fg(Color::Rgb(90, 90, 100))
        .bg(Color::Rgb(18, 19, 24));
    p.fill(0, 0, w, h, ' ', shade);
    // Tunnel mouth shading grows toward travel direction.
    for y in 0..h {
        for x in 0..w {
            let deep = if entrance {
                x >= w / 2 // darkens toward the far side (goes under)
            } else {
                x < w / 2 // emerges from dark
            };
            if deep {
                p.put(x, y, '▓', shade);
            } else if (x + y) % 2 == 0 {
                p.put(x, y, '░', shade);
            }
        }
    }
    // Direction arrow with a bracket mouth, oriented by facing.
    let cy = h / 2;
    let cx = w / 2 - 1;
    let arrow = match ctx.facing {
        Facing::Right => '▶',
        Facing::Left => '◀',
        Facing::Up => '▲',
        Facing::Down => '▼',
    };
    if entrance {
        p.put(cx, cy, arrow, fg);
        p.put(cx + 1, cy, '⊐', fg);
    } else {
        p.put(cx, cy, '⊏', fg);
        p.put(cx + 1, cy, arrow, fg);
    }
}

// ---------------------------------------------------------------------------
// Belts
// ---------------------------------------------------------------------------

/// Lane bed background per belt tier.
fn belt_bed(et: EntityType) -> (u8, u8, u8) {
    match et {
        EntityType::FastBelt => (30, 28, 14),
        EntityType::ExpressBelt => (12, 22, 40),
        _ => (22, 26, 34),
    }
}

/// Rail + chevron colors per belt tier.
fn belt_rail_fg(et: EntityType) -> (u8, u8, u8) {
    match et {
        EntityType::FastBelt => (200, 175, 60),
        EntityType::ExpressBelt => (70, 130, 220),
        _ => (150, 150, 165),
    }
}

fn belt_chevron_fg(et: EntityType) -> (u8, u8, u8) {
    match et {
        EntityType::FastBelt => (255, 220, 50),
        EntityType::ExpressBelt => (110, 180, 255),
        _ => (225, 225, 235),
    }
}

/// Marching speed divisor per belt tier (smaller = faster).
fn belt_speed_div(et: EntityType) -> u32 {
    match et {
        EntityType::FastBelt => 2,
        EntityType::ExpressBelt => 1,
        _ => 3,
    }
}

/// Paint a 1-tile belt sprite: lane rails + animated marching chevrons.
/// Cargo chips are painted by the grid overlay pass (they sit at the tile
/// center, which is exactly the lane center).
fn paint_belt(p: &mut Painter, ctx: &SpriteCtx) {
    let s = ctx.scale as i32;
    let (w, h) = (2 * s, s);
    let et = ctx.entity_type;
    let bed = Style::default().bg(rgb(belt_bed(et)));
    let rail = Style::default().fg(rgb(belt_rail_fg(et))).bg(rgb(belt_bed(et)));
    let chev = Style::default()
        .fg(rgb(belt_chevron_fg(et)))
        .bg(rgb(belt_bed(et)))
        .add_modifier(Modifier::BOLD);

    p.fill(0, 0, w, h, ' ', bed);

    let horizontal = matches!(ctx.facing, Facing::Left | Facing::Right);
    let step = ctx.frame / belt_speed_div(et);

    if horizontal {
        // Rails on top (and bottom at S>=3); the last row is the lane at S=2.
        for x in 0..w {
            p.put(x, 0, '═', rail);
            if h >= 3 {
                p.put(x, h - 1, '═', rail);
            }
        }
        let lane_y = h / 2;
        // Marching chevrons: one every 4 cells, phase-shifted by frame.
        // Use the ABSOLUTE sprite origin so adjacent belts form a continuous
        // marching line.
        let ch = if ctx.facing == Facing::Right { '»' } else { '«' };
        for x in 0..w {
            let world_x = p.ox + x; // absolute cell -> continuous phase
            let phase = if ctx.facing == Facing::Right {
                (world_x - step as i32).rem_euclid(4)
            } else {
                (world_x + step as i32).rem_euclid(4)
            };
            if phase == 0 {
                p.put(x, lane_y, ch, chev);
            } else if phase == 1 {
                p.put(x, lane_y, '·', rail);
            }
        }
    } else {
        // Vertical: rails on left/right columns, lane down the middle.
        for y in 0..h {
            p.put(0, y, '║', rail);
            p.put(w - 1, y, '║', rail);
        }
        let ch = if ctx.facing == Facing::Down { '∨' } else { '∧' };
        let lane_x = w / 2 - 1;
        for y in 0..h {
            let world_y = p.oy + y;
            let phase = if ctx.facing == Facing::Down {
                (world_y - step as i32).rem_euclid(2)
            } else {
                (world_y + step as i32).rem_euclid(2)
            };
            if phase == 0 {
                p.put(lane_x, y, ch, chev);
                p.put(lane_x + 1, y, ch, chev);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Cargo chip
// ---------------------------------------------------------------------------

/// Paint a bold 2-cell resource chip centered on the tile whose top-left
/// cell is at absolute (tile_ox, tile_oy).
pub fn paint_cargo_chip(
    buf: &mut Buffer,
    clip: Rect,
    tile_ox: i32,
    tile_oy: i32,
    scale: u8,
    resource: Resource,
) {
    let s = scale as i32;
    let (r, g, b) = resource.color();
    let chip = Style::default()
        .fg(Color::Rgb(r, g, b))
        .bg(Color::Rgb(r / 4 + 10, g / 4 + 10, b / 4 + 10))
        .add_modifier(Modifier::BOLD);
    let mut p = Painter::new(buf, clip, tile_ox, tile_oy);
    let cx = s - 1; // center-left of the 2s-wide tile
    let cy = s / 2;
    p.put(cx, cy, resource.glyph(), chip);
    p.put(cx + 1, cy, '▪', chip);
}

// ---------------------------------------------------------------------------
// Terrain dither
// ---------------------------------------------------------------------------

/// Dither glyph for a ground cell, seeded by cell coordinates. Two-tone,
/// mostly empty so the world stays calm.
pub fn dither_glyph(x: i32, y: i32) -> char {
    match hash2(x, y) % 11 {
        0 => '·',
        1 => '.',
        2 => '\'',
        3 => '`',
        _ => ' ',
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    fn render_sprite(et: EntityType, facing: Facing, scale: u8, fw: usize, fh: usize) -> Buffer {
        let area = Rect::new(0, 0, 60, 30);
        let mut buf = Buffer::empty(area);
        let ctx = SpriteCtx {
            entity_type: et,
            facing,
            scale,
            frame: 0,
            state: MachineState::Processing,
            fw,
            fh,
        };
        let ports = crate::map::multitile::building_footprint(et)
            .rotate_to(facing)
            .ports;
        paint_building(&mut buf, area, 0, 0, &ctx, &ports);
        buf
    }

    #[test]
    fn test_smelter_sprite_covers_footprint_and_has_flames() {
        let buf = render_sprite(EntityType::Smelter, Facing::Right, 2, 3, 3);
        // 3x3 tiles at S=2 -> 12x6 cells; every cell inside must be painted
        // (default cells have Color::Reset for both fg and bg).
        let mut flames = 0;
        for y in 0..6 {
            for x in 0..12u16 {
                let cell = &buf[(x, y)];
                assert!(
                    cell.style().bg != Some(Color::Reset) || cell.style().fg != Some(Color::Reset),
                    "unpainted cell at {},{}",
                    x,
                    y
                );
                let sym = cell.symbol();
                if sym == "▲" || sym == "△" {
                    flames += 1;
                }
            }
        }
        assert!(flames > 0, "processing smelter should show flame glyphs");
        // Corners are the rounded chassis.
        assert_eq!(buf[(0, 0)].symbol(), "╭");
        assert_eq!(buf[(11, 0)].symbol(), "╮");
        assert_eq!(buf[(0, 5)].symbol(), "╰");
        assert_eq!(buf[(11, 5)].symbol(), "╯");
    }

    #[test]
    fn test_ports_are_on_exact_port_tiles() {
        // Smelter 3x3, Facing::Right: input at tile (0,1), output (2,1),
        // waste (1,2). At S=2, input socket cell = (0, 1*2+1) = (0,3);
        // output = (2*4+3, 3) = (11,3).
        let buf = render_sprite(EntityType::Smelter, Facing::Right, 2, 3, 3);
        assert_eq!(buf[(0, 3)].symbol(), "▸", "input socket on left port tile");
        assert_eq!(buf[(11, 3)].symbol(), "●", "output socket on right port tile");
        // Waste at tile (1,2): cell x = 1*4+2 = 6, y = 2*2+1 = 5.
        assert_eq!(buf[(6, 5)].symbol(), "▿", "waste socket bottom center");
    }

    #[test]
    fn test_belt_sprite_has_rails_and_chevrons() {
        let area = Rect::new(0, 0, 20, 10);
        let mut buf = Buffer::empty(area);
        let ctx = SpriteCtx {
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            scale: 2,
            frame: 0,
            state: MachineState::Idle,
            fw: 1,
            fh: 1,
        };
        paint_building(&mut buf, area, 0, 0, &ctx, &[]);
        // Top row is rail.
        for x in 0..4u16 {
            assert_eq!(buf[(x, 0)].symbol(), "═", "rail on top row");
        }
        // Lane row contains a chevron somewhere.
        let lane: String = (0..4u16).map(|x| buf[(x, 1)].symbol().to_string()).collect();
        assert!(lane.contains('»'), "chevron in lane: {:?}", lane);
    }

    #[test]
    fn test_name_strip_at_scale_3() {
        let buf = render_sprite(EntityType::Smelter, Facing::Right, 3, 3, 3);
        // 3x3 at S=3 -> 18x9 cells. Name strip row = 7.
        let row: String = (0..18u16).map(|x| buf[(x, 7)].symbol().to_string()).collect();
        assert!(row.contains("SMELTER"), "name strip: {:?}", row);
    }

    #[test]
    fn test_generic_chassis_fallback_has_initials() {
        // Press is not in the custom set -> generic chassis with 'P' glyph.
        let buf = render_sprite(EntityType::Press, Facing::Right, 2, 3, 3);
        let mut found = false;
        for y in 0..6 {
            for x in 0..12u16 {
                if buf[(x, y)].symbol() == "P" {
                    found = true;
                }
            }
        }
        assert!(found, "generic chassis shows name initial");
    }

    #[test]
    fn test_sprite_clips_to_rect() {
        // Origin partially off the clip rect: must not panic, must not paint
        // outside.
        let area = Rect::new(2, 2, 8, 4);
        let mut buf = Buffer::empty(Rect::new(0, 0, 12, 8));
        let ctx = SpriteCtx {
            entity_type: EntityType::Smelter,
            facing: Facing::Right,
            scale: 2,
            frame: 5,
            state: MachineState::Idle,
            fw: 3,
            fh: 3,
        };
        paint_building(&mut buf, area, -3, -1, &ctx, &[]);
        for y in 0..8u16 {
            for x in 0..12u16 {
                let inside = x >= 2 && x < 10 && y >= 2 && y < 6;
                if !inside {
                    assert_eq!(buf[(x, y)].symbol(), " ", "painted outside clip at {},{}", x, y);
                }
            }
        }
    }
}
