# VimForge — Visual Overhaul

## Codebase Context

VimForge is a terminal TUI factory/automation game (Rust, ratatui + crossterm + hecs ECS). The rendering pipeline works as follows:

- **`src/main.rs:360` `render_frame()`** calls `compute_layout()` then renders tutorial bar, game grid, sidebar, status bar, and popups
- **`src/ui/layout.rs:29` `compute_layout()`** splits the terminal into `LayoutAreas { tutorial_bar, game_grid, sidebar, status_bar }`
- **`src/ui/grid_render.rs:22` `render_grid()`** iterates screen tiles, each tile = 2 terminal chars. For each tile it calls `resolve_tile_entity()` to get a `TileEntityInfo`, then `glyphs::entity_art()` for the 2-char glyph, `glyphs::entity_style_for_state()` for the style, and `highlights::resolve_highlight()` for cursor/selection/flash overlays
- **`src/render/glyphs.rs`** defines `BuildingArt` structs (multi-tile ASCII art), `building_fg()` colors, `entity_style_for_state()` for idle/processing/blocked/broken states, and `belt_style()` / `resource_style()` for conveyors and resources
- **`src/render/colors.rs`** defines `BiomePalette` with per-region `floor_bg`/`floor_fg`/`accent` colors, plus `apply_day_night()` and `apply_pollution_tint()` tinting
- **`src/render/viewport.rs`** defines `Viewport { offset_x, offset_y, width, height }` with `follow_cursor()` and `clamp_to_map()`
- **`src/map/terrain.rs`** defines `Terrain::Ground` with glyph `'.'`, fg `(60,60,60)`, and `bg_color() -> None`

Buildings already use multi-tile ASCII art (3x2 extractors, 3x3 processors, 3x4 assemblers, etc.) via `BuildingArt` in `glyphs.rs`. The rendering pipeline already supports them through `PartOfBuilding` / `MultiTile` components and `compute_tile_coords()` in `grid_render.rs`.

---

## What's Wrong

### 1. Empty tiles have no background color
In `grid_render.rs:82-93`, empty tiles render terrain via `terrain_glyph_style()` which calls `Terrain::Ground.fg_color()` returning `(60,60,60)` with `bg_color() -> None`. No BG is set. The result: invisible dots on terminal-default black.

### 2. Map is pinned to top-left corner
`Viewport::clamp_to_map()` (viewport.rs:71-82) sets `offset_x = 0` and `offset_y = 0` when the map is smaller than the viewport. There is no centering logic. Level 1 is 30x15 tiles (60 terminal chars x 15 rows) but a typical terminal is ~190x45 — the factory sits in the upper-left with vast empty black space.

### 3. Conveyors are dim and have no lane background
`belt_style()` (glyphs.rs:1036-1049) sets `BasicBelt` fg to `(200,200,200)` and bg to `(20,20,22)` — barely distinguishable from black. `conveyor_idle_style()` (glyphs.rs:986-989) uses `(50,50,50)` with `DIM`. The production line is nearly invisible.

### 4. Resources on conveyors have no glow
In `grid_render.rs:74-76`, when a resource is on a tile, it's rendered with `resource_style()` which sets fg color only — no background glow. Resources blend into the dark conveyor/floor.

### 5. Idle machines are too dim
`entity_style_for_state()` (glyphs.rs:940-943) dims idle machines to 50% via `dim_color(base, 0.5)` with no BG. A Smelter at `(220,60,40)` becomes `(110,30,20)` on black — nearly invisible.

### 6. Sidebar is only 16 columns with no background
`SIDEBAR_WIDTH` in layout.rs:8 is `16`. The sidebar (sidebar.rs) has a `Borders::LEFT` border in `(60,60,70)` but no panel BG color — text floats on terminal-default black.

### 7. Tutorial bar uses `Color::Yellow`/`Color::Green` named colors
tutorial_bar.rs:29-35 uses `Color::Yellow`, `Color::Green`, `Color::Cyan`, `Color::Magenta`, `Color::DarkGray`, `Color::White` — named colors that look different on every terminal. BG is `Color::Rgb(30, 30, 60)` which is fine.

### 8. Status bar has no background
statusbar.rs renders mode indicators with BG (e.g., NORMAL gets `bg(30,35,45)`) but the overall bar has no BG — the rest of the row is terminal-default black.

### 9. No root-level background fill
`render_frame()` in main.rs:360-407 never fills the terminal background. Any area not covered by a widget (padding around the map, gaps) shows terminal-default black.

### 10. Early level maps are small
Level map sizes: L1=30x15, L2=22x8, L3=30x10, L5=34x12, L11=24x15. On a 190x45 terminal, these leave 70-80% of the screen empty.

---

## Fix 1: Floor & Terminal Background — No Pure Black

### 1a. Add floor BG to empty tiles

**File: `src/map/terrain.rs`**

Change `Terrain::Ground` to return a background color:

```rust
pub fn bg_color(&self) -> Option<(u8, u8, u8)> {
    match self {
        Terrain::Ground => Some((18, 22, 28)),    // dark blue-gray factory floor
        Terrain::Desert => Some((22, 18, 10)),
        Terrain::Ice => Some((14, 16, 22)),
        Terrain::Forest => Some((10, 18, 10)),
        Terrain::Swamp => Some((14, 16, 10)),
        Terrain::Water => Some((8, 14, 24)),
        Terrain::DeepWater => Some((4, 8, 18)),
        Terrain::Mountain => Some((16, 16, 18)),
        Terrain::Lava => Some((80, 20, 0)),
        Terrain::RadioactiveZone => Some((10, 30, 10)),
    }
}
```

Also brighten `Terrain::Ground` fg from `(60,60,60)` to `(35,42,52)` so the grid dots are visible against the new BG.

### 1b. Fill root terminal background

**File: `src/main.rs`** in `render_frame()` (line 360), before rendering any widgets:

```rust
use ratatui::widgets::Block;
frame.render_widget(
    Block::default().style(Style::default().bg(Color::Rgb(10, 12, 16))),
    frame.area(),
);
```

This ensures every pixel has at minimum `(10,12,16)` — near-black with subtle blue warmth.

### 1c. Give the sidebar a panel background

**File: `src/ui/sidebar.rs`** in `render_sidebar()` (line 26), change the Block:

```rust
let block = Block::default()
    .borders(Borders::LEFT)
    .border_style(Style::default().fg(Color::Rgb(40, 50, 65)))
    .style(Style::default().bg(Color::Rgb(14, 17, 22)))  // dark panel BG
    .title(Span::styled(
        " VimForge ",
        Style::default()
            .fg(Color::Rgb(255, 200, 60))    // gold title
            .add_modifier(Modifier::BOLD),
    ));
```

### 1d. Give the status bar a background

**File: `src/ui/statusbar.rs`** in `render_statusbar()`, wrap the paragraph:

```rust
let paragraph = Paragraph::new(line)
    .style(Style::default().bg(Color::Rgb(22, 26, 34)));
```

---

## Fix 2: Center The Map When Smaller Than Viewport

**File: `src/render/viewport.rs`**

The viewport needs to center the map content in the available screen area. Currently `clamp_to_map()` pins offset to 0 when the map is smaller. The rendering loop in `grid_render.rs:37-46` uses `viewport.offset_x/offset_y` to compute `map_x/map_y` — this is where tiles start at screen position 0.

Add a padding/centering system to `Viewport`:

```rust
pub struct Viewport {
    pub offset_x: usize,
    pub offset_y: usize,
    pub width: usize,
    pub height: usize,
    /// Terminal-cell padding for centering. Applied in render_grid.
    pub pad_left: u16,
    pub pad_top: u16,
}
```

Add to `clamp_to_map()`:

```rust
pub fn clamp_to_map(&mut self, map_width: usize, map_height: usize) {
    // Each tile = 2 terminal columns
    let map_term_width = map_width * 2;
    let viewport_term_width = self.width * 2;

    if map_width > self.width {
        self.offset_x = self.offset_x.min(map_width - self.width);
        self.pad_left = 0;
    } else {
        self.offset_x = 0;
        self.pad_left = ((viewport_term_width - map_term_width) / 2) as u16;
    }
    if map_height > self.height {
        self.offset_y = self.offset_y.min(map_height - self.height);
        self.pad_top = 0;
    } else {
        self.offset_y = 0;
        self.pad_top = ((self.height - map_height) / 2) as u16;
    }
}
```

**File: `src/ui/grid_render.rs`** — Apply padding in `render_grid()`:

In the cell position calculation (currently line 49-51):

```rust
let cell0_x = area.x + viewport.pad_left + screen_col_tile * 2;
let cell1_x = cell0_x + 1;
let cell_y = area.y + viewport.pad_top + screen_row;
```

The area outside the map (the padding zone) already gets the root BG from Fix 1b. If you want a subtler distinction, fill the game_grid area with a "void" color `(10,12,16)` before rendering tiles — this frames the factory floor.

---

## Fix 3: Bright Conveyor Lanes

**File: `src/render/glyphs.rs`**

### 3a. Brighten belt styles

Replace `belt_style()` (line 1036-1049):

```rust
pub fn belt_style(belt_type: EntityType) -> Style {
    match belt_type {
        EntityType::BasicBelt => Style::default()
            .fg(Color::Rgb(220, 220, 230))       // bright white arrow
            .bg(Color::Rgb(22, 26, 34))           // visible lane BG
            .add_modifier(Modifier::BOLD),
        EntityType::FastBelt => Style::default()
            .fg(Color::Rgb(255, 220, 50))         // gold
            .bg(Color::Rgb(30, 28, 14))
            .add_modifier(Modifier::BOLD),
        EntityType::ExpressBelt => Style::default()
            .fg(Color::Rgb(80, 160, 255))         // bright blue
            .bg(Color::Rgb(12, 22, 40))
            .add_modifier(Modifier::BOLD),
        _ => Style::default().fg(Color::Rgb(200, 200, 200)),
    }
}
```

### 3b. Remove conveyor_idle_style or brighten it

Replace `conveyor_idle_style()` (line 986-989) — idle conveyors should still be visible:

```rust
pub fn conveyor_idle_style() -> Style {
    Style::default()
        .fg(Color::Rgb(120, 120, 130))
        .bg(Color::Rgb(22, 26, 34))
}
```

### 3c. Apply belt_style in the rendering pipeline

Currently `entity_style_for_state()` handles belt styling generically. For belt entities specifically, `belt_style()` should be used instead. In `grid_render.rs:60-81`, when the resolved entity is a belt type, use `belt_style()` as the base style instead of `entity_style_for_state()`:

```rust
let base_style = if matches!(info.entity_type,
    EntityType::BasicBelt | EntityType::FastBelt | EntityType::ExpressBelt)
{
    glyphs::belt_style(info.entity_type)
} else {
    glyphs::entity_style_for_state(info.entity_type, state, frame_counter)
};
```

---

## Fix 4: Resource Glow on Conveyors

**File: `src/render/glyphs.rs`**

Add a resource glow BG function:

```rust
pub fn resource_glow_bg(resource: Resource) -> Color {
    let (r, g, b) = resource.color();
    // Dim glow: resource color at ~20% brightness as BG
    Color::Rgb(r / 5 + 8, g / 5 + 8, b / 5 + 8)
}
```

Update `resource_style()` (line 1061-1068) to include the glow BG:

```rust
pub fn resource_style(resource: Resource) -> Style {
    let (r, g, b) = resource.color();
    Style::default()
        .fg(Color::Rgb(r, g, b))
        .bg(resource_glow_bg(resource))
        .add_modifier(Modifier::BOLD)  // always bold for visibility
}
```

This means when `grid_render.rs:74-76` renders a resource on a tile, the BG automatically glows with the resource's color. A golden ore dot on a warm BG is immediately visible.

---

## Fix 5: Brighter Idle Machines

**File: `src/render/glyphs.rs`**

In `entity_style_for_state()` (line 940-943), idle is currently dimmed to 50%. Change to 65% and add a subtle BG:

```rust
MachineState::Idle => {
    let dimmed = dim_color(base, 0.65);
    let glow = building_glow_bg(entity_type);
    // Half-strength glow BG so idle machines are still visible
    let bg = (glow.0 / 2, glow.1 / 2, glow.2 / 2);
    Style::default()
        .fg(Color::Rgb(dimmed.0, dimmed.1, dimmed.2))
        .bg(Color::Rgb(bg.0.max(12), bg.1.max(12), bg.2.max(12)))
}
```

This gives idle machines a faint colored BG that distinguishes them from empty floor.

---

## Fix 6: Tutorial Bar — Replace Named Colors with Rgb

**File: `src/ui/tutorial_bar.rs`**

Replace all named Color constants (lines 28-39):

```rust
let title_style = Style::default()
    .fg(Color::Rgb(80, 200, 255))      // cyan — was Color::Yellow
    .bg(TUTORIAL_BG)
    .add_modifier(Modifier::BOLD);
let goal_style = Style::default()
    .fg(Color::Rgb(255, 220, 60))      // gold — was Color::Green
    .bg(TUTORIAL_BG)
    .add_modifier(Modifier::BOLD);
let progress_style = Style::default()
    .fg(Color::Rgb(80, 255, 120))      // bright green — was Color::Cyan
    .bg(TUTORIAL_BG)
    .add_modifier(Modifier::BOLD);
```

In the hint row (line 73-76):

```rust
let hint_style = Style::default()
    .fg(Color::Rgb(200, 160, 80))      // warm amber — was Color::Magenta
    .bg(TUTORIAL_BG)
    .add_modifier(Modifier::BOLD);
let counter_style = Style::default()
    .fg(Color::Rgb(80, 80, 100))       // muted — was Color::DarkGray
    .bg(TUTORIAL_BG);
```

In `entity_legend_spans()` (lines 117-174), replace `Color::Red`, `Color::Green`, `Color::White`, `Color::Cyan` with entity-accurate Rgb colors using `glyphs::building_fg()`:

```rust
1 => vec![
    glyph("O", Color::Rgb(160, 120, 60)),    // OreDeposit color
    label("=Ore  "),
    glyph("\u{2192}", Color::Rgb(220, 220, 230)),  // belt color
    label("=Conveyor  "),
    glyph("S", Color::Rgb(220, 60, 40)),     // Smelter color
    label("=Smelter  "),
    glyph("B", Color::Rgb(60, 200, 80)),     // OutputBin color
    label("=Output Bin"),
],
```

Also add a separator line (row 4) below the tutorial bar. Either increase `TUTORIAL_BAR_HEIGHT` in layout.rs from `3` to `4` and add a `───` separator row, or draw a bottom border on the tutorial bar Block.

---

## Fix 7: Wider Sidebar with Visual Hierarchy

**File: `src/ui/layout.rs`**

Increase `SIDEBAR_WIDTH` from `16` to `20`:

```rust
const SIDEBAR_WIDTH: u16 = 20;
```

**File: `src/ui/sidebar.rs`**

The sidebar already has good structure. Key color improvements:

1. Title "VimForge" — change from `(80,200,220)` cyan to `(255,200,60)` gold (more distinctive)
2. Zero values should be dim:

```rust
let value_style = if count == 0 {
    Style::default().fg(Color::Rgb(80, 80, 90))  // dim for zero
} else {
    Style::default()
        .fg(Color::Rgb(220, 225, 235))            // bright for nonzero
        .add_modifier(Modifier::BOLD)
};
```

3. Section headers — already yellow bold `(220,200,60)`, which works. Could shift to blue `(80,160,220)` for contrast against the gold title.

---

## Fix 8: Status Bar Background

**File: `src/ui/statusbar.rs`**

Add a bar-wide BG. In `render_statusbar()` (line 99-100):

```rust
let paragraph = Paragraph::new(line)
    .style(Style::default().bg(Color::Rgb(22, 26, 34)));
```

Also add dashes around the mode indicator for visual weight. In `mode_display()` (line 141):

```rust
Mode::Normal => (
    " -- NORMAL -- ".to_string(),
    Style::default()
        .fg(Color::Rgb(180, 190, 210))
        .bg(Color::Rgb(30, 35, 48)),
),
```

---

## Fix 9: Processing Animation Visibility

**File: `src/render/glyphs.rs`**

The processing pulse in `entity_style_for_state()` (line 945-955) adds only +/-35 to brightness channels. This is barely noticeable. Increase the pulse amplitude and ensure BG also pulses:

```rust
MachineState::Processing => {
    let pulse = ((frame % 6) as f32 / 6.0 * std::f32::consts::PI).sin();
    let pr = (base.0 as f32 + pulse * 50.0).clamp(0.0, 255.0) as u8;
    let pg = (base.1 as f32 + pulse * 50.0).clamp(0.0, 255.0) as u8;
    let pb = (base.2 as f32 + pulse * 50.0).clamp(0.0, 255.0) as u8;
    let glow = building_glow_bg(entity_type);
    // BG also pulses for stronger effect
    let bg_pulse = (pulse * 0.5 + 0.5).max(0.0);  // 0.0..1.0
    let bgr = (glow.0 as f32 * (1.0 + bg_pulse)).min(255.0) as u8;
    let bgg = (glow.1 as f32 * (1.0 + bg_pulse)).min(255.0) as u8;
    let bgb = (glow.2 as f32 * (1.0 + bg_pulse)).min(255.0) as u8;
    Style::default()
        .fg(Color::Rgb(pr, pg, pb))
        .bg(Color::Rgb(bgr, bgg, bgb))
        .add_modifier(Modifier::BOLD)
}
```

The processing countdown digit (rendered in `grid_render.rs:72-74` via `processing_indicator()` in glyphs.rs:1090-1096) already works — the center tile shows `3`, `2`, `1` etc. No change needed there.

---

## Fix 10: Larger Level Maps

Increase early tutorial map sizes so the factory fills more screen. Spread entities out with longer conveyor runs.

### Level 1 (`src/levels/level_01.rs`)

Current: `map_width: 30, map_height: 15`. Two production lines at rows 3 and 9, entities from x=2 to x=23.

Change to `map_width: 44, map_height: 22`. Spread the two lines to rows 5 and 16:

- OreDeposits at (3,4) and (3,15)
- Belts from x=6..=16 (11 tiles, longer visible conveyor run)
- Smelters at (17,4) and (17,15)
- Belts from x=20..=30 (11 tiles)
- OutputBins at (31,4) and (31,15)
- Update `CompletionCondition::NavigateToAll` targets to new OutputBin positions

### Level 2 (`src/levels/level_02.rs`)

Current: `map_width: 22, map_height: 8`. Change to `map_width: 34, map_height: 14`. Scale entity positions proportionally, add longer belt runs.

### Level 3 (`src/levels/level_03.rs`)

Current: `map_width: 30, map_height: 10`. Change to `map_width: 44, map_height: 18`.

### Other early levels (4-8, 11)

Increase by ~40-60%:
- L4: 38x14 -> 52x20
- L5: 34x12 -> 48x18
- L8: 28x22 -> 40x28
- L11: 24x15 -> 36x22

Levels 9, 10, 12, 13 are already large (52x32 to 82x42) and don't need changes.

For each resized level: spread entities apart with more empty space between production lines. The conveyors between machines should be 8-12 tiles long (currently 5-7) to create visible "roads" across the factory.

---

## Color Reference

| Element | FG | BG | Source |
|---|---|---|---|
| Terminal root | -- | `(10,12,16)` | main.rs render_frame |
| Ground floor tile | `(35,42,52)` dot | `(18,22,28)` | terrain.rs Ground |
| Conveyor lane (basic) | `(220,220,230)` BOLD | `(22,26,34)` | glyphs.rs belt_style |
| Conveyor lane (fast) | `(255,220,50)` BOLD | `(30,28,14)` | glyphs.rs belt_style |
| Conveyor lane (express) | `(80,160,255)` BOLD | `(12,22,40)` | glyphs.rs belt_style |
| Resource on belt | Rgb per resource, BOLD | resource color / 5 + 8 | glyphs.rs resource_style |
| Machine idle | base * 0.65 | glow / 2 | glyphs.rs entity_style_for_state |
| Machine processing | base +/- 50 pulse, BOLD | glow * (1..2) pulse | glyphs.rs entity_style_for_state |
| Sidebar panel | varied | `(14,17,22)` | sidebar.rs Block |
| Sidebar title | `(255,200,60)` BOLD | -- | sidebar.rs |
| Status bar | varied | `(22,26,34)` | statusbar.rs |
| Tutorial bar | varied Rgb | `(30,30,60)` | tutorial_bar.rs |
| Tutorial title | `(80,200,255)` BOLD | -- | tutorial_bar.rs |
| Tutorial goal | `(255,220,60)` BOLD | -- | tutorial_bar.rs |
| Cursor | -- | `(80,80,120)` REVERSED | highlights.rs |

---

## Files To Modify — Summary

| File | Changes |
|---|---|
| `src/map/terrain.rs` | Add BG color for all terrain types. Brighten Ground fg to `(35,42,52)`. |
| `src/main.rs` | Add root BG fill `(10,12,16)` in `render_frame()` before widgets. |
| `src/render/viewport.rs` | Add `pad_left`/`pad_top` fields to `Viewport`. Compute centering in `clamp_to_map()`. Update `new()` and `Viewport` initialization. |
| `src/ui/grid_render.rs` | Apply viewport padding to cell positions. Use `belt_style()` for belt entities. |
| `src/render/glyphs.rs` | Brighten `belt_style()`, `conveyor_idle_style()`. Add `resource_glow_bg()`. Adjust `entity_style_for_state()` idle (0.65 dim + subtle BG) and processing (stronger pulse). |
| `src/ui/sidebar.rs` | Add panel BG `(14,17,22)`. Gold title. Dim zero values. |
| `src/ui/statusbar.rs` | Add bar BG `(22,26,34)`. Add dashes around mode text. |
| `src/ui/tutorial_bar.rs` | Replace all named colors with Rgb. Cyan title, gold goal, green progress, amber hints. Fix entity legend colors to match actual `building_fg()` values. |
| `src/ui/layout.rs` | Increase `SIDEBAR_WIDTH` from 16 to 20. |
| `src/levels/level_01.rs` | Increase to 44x22. Spread entities, longer belts. |
| `src/levels/level_02.rs` | Increase to 34x14. |
| `src/levels/level_03.rs` | Increase to 44x18. |
| `src/levels/level_04.rs` | Increase to 52x20. |
| `src/levels/level_05.rs` | Increase to 48x18. |
| `src/levels/level_08.rs` | Increase to 40x28. |
| `src/levels/level_11.rs` | Increase to 36x22. |

## Priority Order

1. **Floor + root BG** (terrain.rs, main.rs) — transforms black void into colored factory floor
2. **Map centering** (viewport.rs, grid_render.rs) — stops factory sitting in the corner
3. **Conveyor brightness + lane BG** (glyphs.rs belt_style, grid_render.rs) — makes production lines visible
4. **Resource glow** (glyphs.rs resource_style) — makes flow visible
5. **Idle machine brightness** (glyphs.rs entity_style_for_state) — makes machines recognizable when idle
6. **UI panels** (sidebar.rs, statusbar.rs, tutorial_bar.rs, layout.rs) — polished chrome
7. **Processing pulse** (glyphs.rs) — brings factory to life
8. **Level map sizes** (level_01 through level_11) — fills the screen
