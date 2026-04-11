# VimForge: Complete Game Guide

A terminal-based factory-building game where Vim's entire grammar is the control scheme. Build production lines, smelt ore into ingots, assemble widgets, and master Vim along the way.

```
cargo run
```

---

## Table of Contents

1. [Game Overview](#game-overview)
2. [Core Concepts](#core-concepts)
3. [Entities](#entities)
4. [Resources](#resources)
5. [Resource Pipeline](#resource-pipeline)
6. [Simulation System](#simulation-system)
7. [Controls: Normal Mode](#controls-normal-mode)
8. [Controls: Insert Mode](#controls-insert-mode)
9. [Controls: Visual Mode](#controls-visual-mode)
10. [Controls: Command Mode](#controls-command-mode)
11. [Controls: Search Mode](#controls-search-mode)
12. [Operators & Motions](#operators--motions)
13. [Text Objects](#text-objects)
14. [Registers](#registers)
15. [Marks](#marks)
16. [Macros](#macros)
17. [Dot Repeat](#dot-repeat)
18. [Undo / Redo](#undo--redo)
19. [Split Panes](#split-panes)
20. [UI Layout](#ui-layout)
21. [Tutorial Levels](#tutorial-levels)
22. [Freeplay Mode](#freeplay-mode)
23. [Progression System](#progression-system)
24. [Save / Load](#save--load)
25. [Technical Reference](#technical-reference)

---

## Game Overview

VimForge is a single-binary terminal TUI game built with Rust. You navigate a tile-based grid, place factory machines using Vim keybindings, and build production chains that convert raw ore into widgets. Every Vim concept -- motions, operators, text objects, registers, macros, marks, visual mode, splits -- maps to a factory-building action.

**Dependencies:** ratatui 0.29, crossterm 0.28, hecs 0.10, serde 1, serde_json 1

**Minimum terminal size:** 80 columns x 24 rows

**Frame rate:** ~30 FPS (33ms frame duration)

---

## Core Concepts

### The Grid

The game world is a 2D tile grid. Each tile can hold:
- One **entity** (a machine, conveyor, wall, etc.)
- One **resource** (ore, ingot, or widget)

Tiles are rendered as 2 character cells wide:
- Cell 1: Entity glyph (or `·` for empty)
- Cell 2: Resource glyph, processing indicator, or space

### Facing

Entities that process or transport resources have a **facing direction** (Up, Right, Down, Left). Facing determines:
- Which side accepts input
- Which side emits output
- Which direction a conveyor pushes resources

### The Cursor

You control a cursor that moves across the grid. In Normal mode you navigate; in Insert mode you place entities at the cursor position.

---

## Entities

There are 8 entity types. Each has a glyph, color, and specific input/output behavior.

| Entity | Glyph | Color | Placeable | Has Facing | Description |
|--------|-------|-------|-----------|------------|-------------|
| Ore Deposit | `O` | Brown RGB(139,119,42) | No | No | Emits raw ore periodically |
| Smelter | `S` | Red | Yes | Yes | Converts Ore into Ingot |
| Assembler | `A` | Cyan | Yes | Yes | Converts 2 Ingots into Widget |
| Conveyor | Arrow (`→←↑↓`) | White | Yes | Yes | Moves resources in facing direction |
| Splitter | `Y` | Yellow | Yes | Yes | Splits 1 input into 2 alternating outputs |
| Merger | `λ` | Yellow | Yes | Yes | Merges 2 inputs into 1 output |
| Output Bin | `B` | Green | No | No | Collects finished products, tracks counts |
| Wall | `█` | Dark Gray | Yes | No | Blocks movement; used as enclosure for text objects |

### Insert-Mode Placement Keys

| Key | Entity |
|-----|--------|
| `s` | Smelter |
| `a` | Assembler |
| `c` | Conveyor |
| `p` | Splitter |
| `e` | Merger |
| `w` | Wall |

### Find-Mode Characters (for `f`/`F`/`t`/`T`)

| Key | Entity |
|-----|--------|
| `s` | Smelter |
| `a` | Assembler |
| `c` | Conveyor |
| `p` | Splitter |
| `m` | Merger |
| `o` | Ore Deposit |
| `b` | Output Bin |
| `w` | Wall |

### Input / Output Sides

Each entity type defines which sides accept input and which sides emit output, relative to facing direction:

| Entity | Input Sides | Output Sides |
|--------|------------|--------------|
| Ore Deposit | None | All 4 sides |
| Smelter | Back (opposite of facing) | Front (facing direction) |
| Assembler | Left & Right (perpendicular) | Front (facing direction) |
| Conveyor | Back + both sides | Front (facing direction) |
| Splitter | Back (opposite of facing) | Left & Right (perpendicular) |
| Merger | Left & Right (perpendicular) | Front (facing direction) |
| Output Bin | All 4 sides | None |
| Wall | None | None |

---

## Resources

| Resource | Glyph | Color | Source |
|----------|-------|-------|--------|
| Ore | `o` | Tan RGB(180,140,60) | Emitted by Ore Deposits |
| Ingot | `i` | Silver RGB(200,200,200) | Produced by Smelters (1 Ore -> 1 Ingot) |
| Widget | `w` | Green RGB(100,220,100) | Produced by Assemblers (2 Ingots -> 1 Widget) |

---

## Resource Pipeline

The full production chain:

```
Ore Deposit ──Ore──> Conveyor(s) ──Ore──> Smelter ──Ingot──> Conveyor(s) ─┐
                                                                           ├──> Assembler ──Widget──> Conveyor(s) ──> Output Bin
Ore Deposit ──Ore──> Conveyor(s) ──Ore──> Smelter ──Ingot──> Conveyor(s) ─┘
```

- **Ore Deposit** emits 1 Ore every N ticks (default: 4) to an adjacent entity with a matching input side
- **Conveyor** pushes its resource 1 tile per tick in its facing direction
- **Smelter** consumes Ore from its input side, processes for 3 ticks, outputs Ingot on its front side
- **Assembler** consumes 1 Ingot from each perpendicular side, processes for 5 ticks, outputs Widget on its front side
- **Splitter** takes 1 input and alternates output between its two perpendicular sides
- **Merger** takes from two perpendicular inputs (alternating priority) and outputs forward
- **Output Bin** consumes any resource on its own tile or pushed from an adjacent entity with matching output side; increments counters

---

## Simulation System

The simulation runs in a fixed tick order. Each tick executes 7 steps:

| Step | System | Description |
|------|--------|-------------|
| 1 | `ore_deposit_emit` | Ore deposits check interval and emit Ore to valid adjacent inputs |
| 2 | `machine_output` | Smelters/Assemblers with completed output push to adjacent entity with matching input |
| 3 | `conveyor_movement` | All conveyors simultaneously push resources forward (deterministic, sorted by entity ID) |
| 4 | `splitter_process` | Splitters route resources to alternating perpendicular outputs |
| 5 | `merger_process` | Mergers pull from alternating perpendicular inputs |
| 6 | `output_bin_consume` | Output bins consume resources on their own tile or adjacent tiles |
| 7 | `machine_process_tick` | Decrement processing timers; on completion: Smelter produces Ingot, Assembler produces Widget |

### Simulation Timing

| Parameter | Default |
|-----------|---------|
| Ore emit interval | 4 ticks |
| Smelter processing | 3 ticks |
| Assembler processing | 5 ticks |
| Simulation speed | 5 ticks/second |
| Max speed | 20 ticks/second |
| Speed 0 | Paused |

### Conveyor Blocking

If a conveyor's destination tile already has a resource, the conveyor holds its resource until the destination clears. No resource is ever lost or overwritten.

---

## Controls: Normal Mode

Normal mode is the default mode for navigation and issuing commands.

### Movement

| Key | Action | Accepts Count |
|-----|--------|---------------|
| `h` or `Left` | Move cursor left | Yes |
| `j` or `Down` | Move cursor down | Yes |
| `k` or `Up` | Move cursor up | Yes |
| `l` or `Right` | Move cursor right | Yes |
| `w` | Jump to next entity (same type, wraps rows) | Yes |
| `W` | Jump to next entity (different type) | Yes |
| `b` | Jump to previous entity (same type, wraps rows) | Yes |
| `B` | Jump to previous entity (different type) | Yes |
| `e` | Jump to end of horizontal entity cluster | No |
| `0` | Jump to column 0 (line start) | No |
| `$` or `End` | Jump to last column (line end) | No |
| `^` | Jump to first entity in current row | No |
| `gg` | Jump to map top (or line N with count) | Yes (as line number) |
| `G` | Jump to map bottom (or line N with count) | Yes (as line number) |
| `H` | Jump to viewport top row | No |
| `M` | Jump to viewport middle row | No |
| `L` | Jump to viewport bottom row | No |
| `}` | Jump to next paragraph (next non-empty row after gap) | Yes |
| `{` | Jump to previous paragraph | Yes |
| `%` | Jump to connected entity (follow conveyor chain) | No |

### Find / Til

| Key | Action |
|-----|--------|
| `f<char>` | Jump forward to next entity of type matching `<char>` |
| `F<char>` | Jump backward to previous entity of type |
| `t<char>` | Jump forward to tile before entity of type |
| `T<char>` | Jump backward to tile after entity of type |
| `;` | Repeat last find in same direction |
| `,` | Repeat last find in opposite direction |

### Single-Key Edits

| Key | Action | Accepts Count |
|-----|--------|---------------|
| `x` | Delete entity under cursor | Yes |
| `~` | Toggle/rotate facing of entity under cursor | No |
| `r<char>` | Replace entity under cursor with type matching `<char>` | No |

### Mode Switches

| Key | Action |
|-----|--------|
| `i` | Enter Insert mode (count = repeat) |
| `v` | Enter Visual (character-wise) mode |
| `V` | Enter Visual Line mode |
| `Ctrl-v` | Enter Visual Block mode |
| `:` | Enter Command mode |
| `/` | Enter Search mode (forward) |
| `?` | Enter Search mode (backward) |

### Search Navigation

| Key | Action | Accepts Count |
|-----|--------|---------------|
| `n` | Go to next search match | Yes |
| `N` | Go to previous search match | Yes |
| `*` | Search forward for entity type under cursor | No |
| `#` | Search backward for entity type under cursor | No |

### Paste

| Key | Action | Accepts Count |
|-----|--------|---------------|
| `p` | Paste from register after cursor | Yes |
| `P` | Paste from register before cursor | Yes |

### Marks

| Key | Action |
|-----|--------|
| `m<a-z>` | Set mark at current cursor position |
| `'<a-z>` | Jump to mark (first entity in that row) |
| `` `<a-z> `` | Jump to mark exact position |
| `''` | Jump to previous jump position (row) |
| ` `` ` | Jump to previous jump position (exact) |

### Macros

| Key | Action |
|-----|--------|
| `q<a-z>` | Start recording macro into register |
| `q` (while recording) | Stop recording |
| `@<a-z>` | Play macro from register (accepts count) |
| `@@` | Replay last played macro (accepts count) |

### Undo / Redo / Repeat

| Key | Action |
|-----|--------|
| `u` | Undo last edit |
| `Ctrl-r` | Redo |
| `.` | Repeat last edit command |

### Window / Split

| Key | Action |
|-----|--------|
| `Ctrl-w v` | Split viewport vertically |
| `Ctrl-w s` | Split viewport horizontally |
| `Ctrl-w h/j/k/l` | Move focus to pane in direction |
| `Ctrl-w q` | Close current pane |
| `Ctrl-w o` | Close all other panes |
| `Ctrl-w =` | Equalize pane sizes |

### UI Toggles

| Key | Action |
|-----|--------|
| `Ctrl-g` | Toggle sidebar |

### Quit Shortcuts

| Key | Action |
|-----|--------|
| `ZZ` | Save and quit |
| `ZQ` | Quit without saving |
| `Ctrl-C` | Force quit |

### Count Prefix

Most commands accept a numeric prefix. Type a number before a command to repeat it:
- `3j` = move down 3 times
- `5x` = delete 5 entities
- `2dd` = delete 2 lines
- `3@a` = play macro `a` three times

Counts can also precede operators: `2d3j` = delete 6 lines down (2 * 3).

---

## Controls: Insert Mode

Insert mode is for placing entities on the grid. Enter with `i` from Normal mode.

### Entity Placement

| Key | Action |
|-----|--------|
| `s` | Place Smelter at cursor and advance |
| `a` | Place Assembler at cursor and advance |
| `c` | Place Conveyor at cursor and advance |
| `p` | Place Splitter at cursor and advance |
| `e` | Place Merger at cursor and advance |
| `w` | Place Wall at cursor and advance |

Entities are placed with the current **insert facing** direction (default: Right).

### Movement (no placement)

| Key | Action |
|-----|--------|
| `h` | Move cursor left (no placement) |
| `j` | Move cursor down (no placement) |
| `k` | Move cursor up (no placement) |
| `l` | Move cursor right (no placement) |

### Change Facing + Move

| Key | Action |
|-----|--------|
| `H` or `Left` | Set facing Left, move left |
| `J` or `Down` | Set facing Down, move down |
| `K` or `Up` | Set facing Up, move up |
| `L` or `Right` | Set facing Right, move right |

### Other

| Key | Action |
|-----|--------|
| `Backspace` | Move cursor back (undo last cursor advance) |
| `Esc` | Exit to Normal mode |

---

## Controls: Visual Mode

Visual mode selects a range of tiles for batch operations. Three sub-modes:

| Mode | Entry Key | Selection Shape |
|------|-----------|-----------------|
| Visual | `v` | Character-wise (reading order from anchor to cursor) |
| Visual Line | `V` | Full rows from anchor row to cursor row |
| Visual Block | `Ctrl-v` | Rectangle from anchor corner to cursor corner |

### Selection Adjustment

All Normal mode movement keys work to extend the selection (hjkl, w, b, 0, $, gg, G, H, M, L, }, {, %).

| Key | Action |
|-----|--------|
| `o` | Swap anchor and cursor (toggle which end is active) |

### Operators on Selection

| Key | Action |
|-----|--------|
| `d` | Delete all entities in selection |
| `y` | Yank (copy) all entities in selection to register |
| `c` | Change (delete selection, enter Insert mode) |
| `>` | Rotate all entities in selection clockwise |
| `<` | Rotate all entities in selection counter-clockwise |
| `p` | Paste register contents over selection |

### Exit

| Key | Action |
|-----|--------|
| `Esc` | Exit to Normal mode |
| `v` | Exit if already in Visual mode |
| `V` | Exit if already in Visual Line mode |
| `Ctrl-v` | Exit if already in Visual Block mode |

---

## Controls: Command Mode

Enter with `:` from Normal mode. Type a command and press Enter to execute.

| Command | Aliases | Description |
|---------|---------|-------------|
| `:w [path]` | `:write` | Save game (optionally to specific path) |
| `:q` | `:quit` | Quit (must save first or use `:q!`) |
| `:q!` | `:quit!` | Force quit without saving |
| `:wq` | `:x` | Save and quit |
| `:e <path>` | `:edit` | Load save file |
| `:speed <n>` | | Set simulation speed (0-20 ticks/sec) |
| `:pause` | | Pause simulation |
| `:resume` | `:run` | Resume simulation |
| `:step` | | Single simulation tick |
| `:stats` | | Show statistics popup |
| `:registers` | `:reg` | Show registers popup |
| `:marks` | | Show marks popup |
| `:map` | `:mapinfo` | Show map info popup |
| `:help [topic]` | `:h` | Show help popup (topics: insert, visual, or general) |
| `:level [n]` | | Jump to tutorial level N |
| `:restart` | | Restart current level |
| `:freeplay` | | Enter freeplay mode |
| `:menu` | | Return to main menu |
| `:noh` | `:nohlsearch` | Clear search highlighting |
| `:version` | `:ver` | Show version info |

### Command Mode Keys

| Key | Action |
|-----|--------|
| Any character | Append to command line |
| `Backspace` | Delete last character (exit if empty) |
| `Tab` | Autocomplete command name |
| `Up` | Previous command in history |
| `Down` | Next command in history |
| `Enter` | Execute command |
| `Esc` | Cancel and return to Normal mode |

---

## Controls: Search Mode

Enter with `/` (forward) or `?` (backward) from Normal mode.

Type an entity name prefix to search. Matching is done via `EntityType::from_search_prefix()`.

| Key | Action |
|-----|--------|
| Any character | Append to search pattern |
| `Backspace` | Delete last character (exit if empty) |
| `Up` | Previous search in history |
| `Down` | Next search in history |
| `Enter` | Execute search, jump to first match |
| `Esc` | Cancel search, return to Normal mode |

After searching, use `n`/`N` to cycle through matches. Use `:noh` to clear highlights.

---

## Operators & Motions

Operators follow the Vim grammar: `[count] operator [count] motion`

### Operators

| Key | Operator | Description |
|-----|----------|-------------|
| `d` | Delete | Remove entities (saved to register) |
| `y` | Yank | Copy entities to register |
| `c` | Change | Delete entities and enter Insert mode |
| `>` | Rotate CW | Rotate entity facings clockwise |
| `<` | Rotate CCW | Rotate entity facings counter-clockwise |

### Linewise Operator Shortcuts

| Keys | Action |
|------|--------|
| `dd` | Delete entire row(s) |
| `yy` | Yank entire row(s) |
| `cc` | Change entire row(s) |
| `>>` | Rotate entire row(s) clockwise |
| `<<` | Rotate entire row(s) counter-clockwise |

### Motions (used after operator)

Any movement command can serve as a motion:
- `dw` = delete to next entity
- `d$` = delete to end of line
- `dgg` = delete to map top
- `y3j` = yank 3 rows down
- `>}` = rotate clockwise to next paragraph

---

## Text Objects

Text objects select regions when used after an operator + `i` (inner) or `a` (around):

| Text Object | `i` (inner) | `a` (around) |
|-------------|-------------|--------------|
| `w` (word) | Contiguous entity cluster on current row | Cluster plus adjacent empty space |
| `p` (paragraph) | Contiguous rows containing entities | Paragraph plus adjacent empty row |
| `b` or `(` (block) | Tiles enclosed by walls (flood fill, excluding walls) | Enclosed tiles plus surrounding walls |

Examples:
- `diw` = delete the entity cluster under cursor
- `yap` = yank the paragraph (group of entity rows) including trailing blank row
- `dib` = delete everything inside wall enclosure

---

## Registers

Registers store blueprints (copied entity layouts) or macro keystrokes.

### Register Types

| Register | Name | Description |
|----------|------|-------------|
| `"a` - `"z` | Named | User-assignable, persist until overwritten |
| `""` | Unnamed | Default for all operations |
| `"0` | Yank | Last yanked content (yy, yw, etc.) |
| `"1` | Delete | Last deleted content (dd, dw, x, etc.) |

### Usage

- `"ayy` = yank current line into register `a`
- `"ap` = paste from register `a`
- `"Ayy` = append to register `a` (uppercase appends)
- `yy` without register prefix = uses unnamed register `""` and also updates `"0`
- `dd` without register prefix = uses unnamed register `""` and also updates `"1`

### Blueprint Storage

When you yank or delete entities, they are stored as a **Blueprint** containing:
- Relative positions of all entities
- Entity types and facings
- Width and height of the blueprint
- Whether the selection was linewise

View registers with `:registers` or `:reg`.

---

## Marks

Marks save cursor positions for quick navigation.

| Mark | Description |
|------|-------------|
| `a` - `z` | User-settable named marks |
| `'` (prev jump) | Automatic; stores position before last jump |

### Commands

- `ma` = set mark `a` at current position
- `'a` = jump to row of mark `a` (first entity in that row)
- `` `a `` = jump to exact position of mark `a`
- `''` = jump to row of previous jump
- ` `` ` = jump to exact position of previous jump

View marks with `:marks`.

---

## Macros

Macros record a sequence of keystrokes for replay.

### Recording

1. Press `qa` to start recording into register `a`
2. Perform any sequence of actions
3. Press `q` to stop recording

### Playback

- `@a` = play macro from register `a`
- `5@a` = play macro 5 times
- `@@` = replay last played macro

### Safety

- Maximum recursion depth: 100 (prevents infinite loops from recursive macros)
- Macros are stored in named registers (`a-z`), shared with blueprint storage

---

## Dot Repeat

The `.` key repeats the last edit command. This includes:

- **Normal mode edits**: `x`, `dd`, `dw`, `>>`, `r<char>`, etc.
- **Insert sessions**: The entire sequence from `i` to `Esc` (all placements in one insert session)

Example workflow:
1. `~` to rotate a conveyor
2. `l` to move right
3. `.` to rotate the next conveyor
4. Repeat `l.` for each conveyor

---

## Undo / Redo

| Key | Action |
|-----|--------|
| `u` | Undo last edit (restores full map snapshot) |
| `Ctrl-r` | Redo (re-applies undone change) |

### Details

- Undo captures a **full map snapshot** (all entities, resources, processing state, counters)
- Maximum undo depth: **100 snapshots**
- New edits clear the redo stack
- Undo/redo restores: entity positions, facings, processing timers, emitter counters, output bin counts, splitter/merger state, and all resources on the map

---

## Split Panes

Split the viewport to view multiple map areas simultaneously.

| Key | Action |
|-----|--------|
| `Ctrl-w v` | Split vertically (side by side) |
| `Ctrl-w s` | Split horizontally (stacked) |
| `Ctrl-w h/j/k/l` | Move focus to pane in direction |
| `Ctrl-w q` | Close current pane |
| `Ctrl-w o` | Close all other panes |
| `Ctrl-w =` | Equalize pane sizes |

- Maximum panes: **4** (2x2 grid at maximum)
- 2 panes: 50/50 split
- 3 panes: each gets 1/3
- 4 panes: 2x2 grid
- Each pane has its own viewport/camera that scrolls independently
- The cursor and game state are shared across all panes

---

## UI Layout

### Screen Layout

```
+--------------------------------------------------+
| Tutorial Bar (2 rows, optional)                   |
| Level name | Objective | Hint                     |
+--------------------------------------+-----------+
|                                      |           |
|           Game Grid                  | Sidebar   |
|        (2 chars per tile)            | (16 cols) |
|                                      |           |
+--------------------------------------+-----------+
| Status Bar (1 row)                               |
| [MODE] status message    pending    [col,row]    |
+--------------------------------------------------+
```

### Sidebar Sections

The sidebar (toggled with `Ctrl-g`) shows:

1. **VimForge** title (Cyan)
2. **Output** section: Widget/Ingot/Ore counts from output bins
3. **Sim** section: Tick count and speed (green = running, red = paused)
4. **Regs** section: Up to 5 most recently used registers
5. **Marks** section: Up to 8 set marks with positions

### Status Bar

| Position | Content |
|----------|---------|
| Left | Mode indicator with color: NORMAL (white), INSERT (green bg), VISUAL (orange bg) |
| Center | Pending keystrokes (yellow, e.g., `d` waiting for motion) |
| Right | Recording indicator (`recording @a` in red) and cursor position `[col,row]` |

In Command/Search mode, the status bar shows the command line with `:` or `/`/`?` prefix.

### Popups

Popups overlay the game screen (centered, 60% width, 70% height):

| Popup | Trigger | Content |
|-------|---------|---------|
| Help | `:help` | Keybinding reference (topics: general, insert, visual) |
| Stats | `:stats` | Map size, tick count, entity counts, output totals |
| Registers | `:reg` | All non-empty registers with content summaries |
| Marks | `:marks` | All set marks with positions |

Popup controls: `j`/`k` to scroll, `Esc`/`q` to close.

### Main Menu

Shown on startup. Options:

| Key | Option | Condition |
|-----|--------|-----------|
| `1` | Tutorial | Always available |
| `2` | Freeplay | Unlocked after completing levels |
| `3` | Load Save | Available if save file exists |
| `4` / `q` | Quit | Always available |

### Highlight System

Tiles can have colored highlights with this priority (highest first):

| Highlight | Background Color | When Active |
|-----------|-----------------|-------------|
| Error Flash | RGB(80,0,0) | Invalid placement (3 frames) |
| Placement Flash | RGB(60,100,60) | Successful placement (3 frames) |
| Cursor (Normal) | RGB(80,80,120) reversed | Always in Normal mode |
| Cursor (Insert) | RGB(60,120,60) reversed | Always in Insert mode |
| Search Current | RGB(200,150,0) | Active search, current match |
| Visual Selection | RGB(100,80,40) | Visual mode active |
| Search Other | RGB(80,60,0) | Active search, other matches |

### Animations

- **Placement flash**: 3 frames (~100ms) green flash on successful entity placement
- **Error flash**: 3 frames (~100ms) red flash on invalid placement

---

## Tutorial Levels

The game includes 13 progressive tutorial levels, each teaching specific Vim mechanics. Levels unlock sequentially (completing level N unlocks level N+1).

### Level 1: Movement (20x10)

**Objective:** Navigate to each output bin and watch resources flow.

**Pre-built factory:** Two complete ore-to-output production lines (rows 2 and 7). Ore deposits at (2,2) and (2,7), conveyors spanning across, output bins at (15,2) and (15,7).

**Completion:** Visit both output bin positions: (15,2) and (15,7).

**Allowed commands:** h, j, k, l, 0, $, gg, G, H, M, L (movement only)

**Hints:**
1. Use h/j/k/l to move left/down/up/right.
2. Use 0 to jump to start of row, $ to jump to end.
3. Use gg to go to top, G to go to bottom.
4. Use H/M/L to jump to top/middle/bottom of the visible area.
5. Navigate to both output bins at (15,2) and (15,7).

**Teaches:** Basic hjkl movement, line navigation (0, $), map navigation (gg, G), viewport positioning (H, M, L).

---

### Level 2: First Placement (15x8)

**Objective:** Connect the ore deposit to the output bin using conveyors.

**Pre-placed:** Ore Deposit at (1,4), Output Bin at (13,4). Gap of 11 tiles between them.

**Completion:** Deliver 3 ore to the output bin.

**Hints:**
1. Press i to enter Insert mode.
2. In Insert mode, press c to place a conveyor.
3. Press Esc to return to Normal mode.
4. Conveyors face right by default. Place them from left to right.
5. Ore will flow automatically once the path is complete.

**Teaches:** Insert mode (i), conveyor placement (c), exiting insert (Esc).

---

### Level 3: Smelting (20x8)

**Objective:** Build Ore -> Conveyors -> Smelter -> Conveyors -> Output. Deliver 3 ingots.

**Pre-placed:** Ore Deposit at (1,4), Output Bin at (18,4).

**Completion:** Deliver 3 ingots.

**Hints:**
1. Place conveyors from the ore deposit toward the middle.
2. Place a smelter (s in Insert mode) to convert ore into ingots.
3. Smelters take input from behind and output forward.
4. Continue with conveyors from the smelter to the output bin.

**Teaches:** Smelter placement (s), understanding input/output sides, multi-entity chains.

---

### Level 4: Full Production (25x12)

**Objective:** Two ore lines -> smelters -> assembler -> output bin. Deliver 3 widgets.

**Pre-placed:** Ore Deposits at (1,3) and (1,9), Output Bin at (23,6).

**Completion:** Produce 3 widgets.

**Hints:**
1. Build two parallel conveyor lines from each ore deposit.
2. Place a smelter on each line to produce ingots.
3. Route both ingot lines to an assembler (a in Insert mode).
4. Assemblers take inputs from the sides and output forward.
5. Connect the assembler output to the bin with conveyors.

**Teaches:** Assembler placement (a), multi-input entities, routing from multiple sources.

---

### Level 5: Demolish & Rebuild (25x10)

**Objective:** Fix the broken factory. Deliver 5 ingots.

**Pre-placed:** A partially built production line with deliberate defects:
- Correct conveyors at (2-4, 5) facing Right
- Wrong-facing conveyor at (5,5) facing Left
- Wrong-facing conveyor at (6,5) facing Up
- Missing conveyor at x=7
- Wrong-facing smelter at (8,5) facing Up (should be Right)
- Wrong-facing conveyor at (9,5) facing Left
- Correct conveyors at (10-14, 5) facing Right
- Missing conveyors at x=15-16
- Correct conveyors at (17-22, 5) facing Right

**Completion:** Deliver 5 ingots.

**Hints:**
1. Use x to delete the entity under the cursor.
2. Use d followed by a motion to delete a range.
3. Delete wrong-facing conveyors and replace them.
4. Fix the smelter by deleting it and placing a new one facing Right.
5. Fill in any gaps with new conveyors.

**Teaches:** Delete command (x, d+motion), debugging broken factories.

---

### Level 6: Copy That (30x15)

**Objective:** Build one line, yy to copy, p to paste. Produce 9 widgets total.

**Pre-placed:** 3 Ore Deposits at (1,3), (1,7), (1,11) and 3 Output Bins at (28,3), (28,7), (28,11).

**Completion:** Produce 9 widgets.

**Hints:**
1. Build a full production line on row 3 first.
2. Use yy to yank (copy) the current row of entities.
3. Move to the target row and use p to paste.
4. Each line needs: ore -> conveyors -> smelter -> conveyors -> assembler -> conveyors -> bin.
5. All three lines must produce widgets for a total of 9.

**Teaches:** Yank (yy) and paste (p), efficient duplication.

---

### Level 7: Blueprints (40x20)

**Objective:** Use named registers to save and paste blueprints. Produce 10 widgets.

**Pre-placed:** 4 Ore Deposits at (1,3), (1,7), (1,11), (1,15) and 2 Output Bins at (38,5), (38,13).

**Completion:** Produce 10 widgets.

**Hints:**
1. Build a two-row production cluster that merges into one output.
2. Use "a2yy to yank 2 rows into register 'a'.
3. Navigate to where you want the next cluster.
4. Use "ap to paste from register 'a'.
5. Named registers (a-z) persist until overwritten.

**Teaches:** Named registers ("a, "b), multi-line yank (2yy), register-targeted paste ("ap).

---

### Level 8: Block Select (40x20)

**Objective:** Use Visual-Block mode to copy a rectangular factory section. Produce 6 widgets.

**Pre-placed:** A working 4-row production cluster (rows 2-5) with ore deposits, conveyors, smelters, assembler, and output bin. A second partially set up area (rows 12-15) with ore deposits and an output bin.

**Completion:** Produce 6 widgets.

**Hints:**
1. Use Ctrl-v to enter Visual-Block mode.
2. Select the rectangular area of the working cluster.
3. Use y to yank the selection.
4. Navigate to the target area and p to paste.
5. Visual-Block copies a rectangular region, not just lines.

**Teaches:** Visual Block mode (Ctrl-v), rectangular selection and paste.

---

### Level 9: Find & Jump (60x30)

**Objective:** Navigate a large factory and fix all 5 broken clusters.

**Pre-placed:** 5 production clusters spread across a 60x30 map, each with a different defect:
- Cluster 1 (row 3): Missing conveyor at x=7
- Cluster 2 (row 5): Smelter facing Left instead of Right
- Cluster 3 (row 14): Missing conveyors at x=24-25
- Cluster 4 (row 22): Conveyor facing Up instead of Right
- Cluster 5 (row 25): Missing smelter at x=46

**Completion:** Custom -- all 5 clusters producing output.

**Hints:**
1. Use f followed by a character to jump to the next entity of that type.
2. Use / to search for entity types by name.
3. Use % to jump between matching input/output pairs.
4. Each cluster has a different problem.
5. Fix all 5 clusters so they produce output.

**Teaches:** Find (f/F/t/T), search (/), match connection (%), navigating large maps.

---

### Level 10: Macro Factory (50x20)

**Objective:** Record a macro and replay it to build all 5 lines. Produce 15 widgets.

**Pre-placed:** 5 Ore Deposits at (1,2), (1,6), (1,10), (1,14), (1,18) and 5 Output Bins at (48,2), (48,6), (48,10), (48,14), (48,18).

**Completion:** Produce 15 widgets.

**Hints:**
1. Press qa to start recording a macro into register 'a'.
2. Build one complete production line from ore to output.
3. Press q to stop recording.
4. Move to the next ore deposit.
5. Press 4@a to replay the macro 4 times for the remaining lines.

**Teaches:** Macro recording (qa...q), macro replay (@a, count prefix).

---

### Level 11: The Dot (30x15)

**Objective:** Use `~` and `.` to fix all conveyors. All conveyors must face Right.

**Pre-placed:** Ore Deposit at (1,7), 15 conveyors at (2-16, 7) all facing Up (wrong -- should be Right), Output Bin at (18,7).

**Completion:** Custom -- all conveyors facing Right.

**Hints:**
1. The ~ key rotates the entity under the cursor clockwise.
2. Press ~ on a conveyor facing Up to rotate it to Right.
3. The . key repeats your last action.
4. Move to the next conveyor with l, then press . to rotate it too.
5. This is much faster than re-placing each conveyor!

**Teaches:** Rotate (~), dot repeat (.), efficient repetitive editing.

---

### Level 12: Marks & Navigation (80x40)

**Objective:** Set marks and navigate between 4 scattered clusters. Produce 4 widgets.

**Pre-placed:** 4 partially built clusters in corners of an 80x40 map:
- Top-left (row 3): Ore (2,3), partial conveyors, output at (15,3)
- Top-right (row 3): Ore (60,3), partial conveyors, output at (73,3)
- Bottom-left (row 35): Ore (2,35), partial conveyors, output at (15,35)
- Bottom-right (row 35): Ore (60,35), partial conveyors, output at (73,35)

**Completion:** Produce 4 widgets.

**Hints:**
1. Use ma to set mark 'a' at the current position.
2. Use 'a to jump back to mark 'a'.
3. Set marks at each cluster so you can quickly jump between them.
4. Each cluster is partially built; complete the smelter chains.
5. Marks save your position even when you scroll far away.

**Teaches:** Mark setting (m), mark jumping (' and `), large-map workflow.

---

### Level 13: Split View (80x40)

**Objective:** Use split views to manage a cross-map conveyor chain. Produce 5 widgets.

**Pre-placed:** Top-left production area (rows 2-4) with ore deposits, conveyors, and smelters. Bottom-right area with just an output bin at (75,37). Player must connect the two areas with a long conveyor chain including an assembler.

**Completion:** Produce 5 widgets.

**Hints:**
1. Use :sp or :vs to split the view.
2. Use Ctrl-w + h/j/k/l to navigate between splits.
3. One split can show the ingot output area at top-left.
4. The other split shows the assembly area at bottom-right.
5. Build a long conveyor chain connecting the two areas with an assembler.

**Teaches:** Split views (Ctrl-w v/s), split navigation (Ctrl-w h/j/k/l), managing distant map areas.

---

## Freeplay Mode

Unlocked after completing all 13 tutorial levels. An 80x50 sandbox with no restrictions.

**Pre-placed:**
- 10 Ore Deposits at: (2,5), (2,15), (2,25), (2,35), (2,45), (40,5), (40,15), (40,25), (40,35), (40,45)
- 4 Output Bins at: (75,10), (75,20), (75,30), (75,40)
- Wall segments: horizontal at x=20-25 y=0, x=50-55 y=49; vertical at x=30 y=20-30, x=60 y=10-18

All commands available. No completion condition. Build whatever you want.

---

## Progression System

### Level Unlocking

- Level 1: Always unlocked
- Levels 2-13: Unlocked sequentially (complete N to unlock N+1)
- Freeplay: Unlocked after completing all 13 levels (also unlockable after level 6 in code)

### Command Proficiency Tracking

The game tracks mastery of 25 command categories:

| Category | Commands |
|----------|----------|
| hjkl | Basic movement |
| w_b | Word/entity jumping |
| 0_dollar | Line navigation |
| gg_G | Map navigation |
| f_char | Find character |
| paragraph | Paragraph movement |
| percent | Match connection |
| insert | Insert mode |
| delete | Delete operations |
| yank | Copy operations |
| change | Change operations |
| paste | Paste operations |
| visual | Visual mode |
| visual_line | Visual line mode |
| visual_block | Visual block mode |
| registers | Named registers |
| marks | Mark operations |
| macros | Macro record/play |
| dot_repeat | Dot repeat |
| search | Search operations |
| text_objects | Text object selection |
| splits | Split panes |
| rotate | Entity rotation |
| undo_redo | Undo/redo |
| command_mode | Command-line mode |
| counts | Number prefixes |

### Completion Conditions by Level

| Level | Condition |
|-------|-----------|
| 1 | Navigate to positions (15,2) and (15,7) |
| 2 | Deliver 3 ore |
| 3 | Deliver 3 ingots |
| 4 | Produce 3 widgets |
| 5 | Deliver 5 ingots |
| 6 | Produce 9 widgets |
| 7 | Produce 10 widgets |
| 8 | Produce 6 widgets |
| 9 | Custom: all 5 clusters producing |
| 10 | Produce 15 widgets |
| 11 | Custom: all conveyors facing right |
| 12 | Produce 4 widgets |
| 13 | Produce 5 widgets |

---

## Save / Load

### Save Location

Save files are stored in `~/.vimforge/` as JSON:
- Default path: `~/.vimforge/save.json`
- Custom paths via `:w <path>` and `:e <path>`

### What Gets Saved

| Data | Details |
|------|---------|
| Map dimensions | Width and height |
| All entities | Position, type, facing, processing state, emitter ticks, output counts, splitter/merger state, player-placed flag |
| All resources | Position and type (Ore/Ingot/Widget) |
| Registers | Named registers with blueprints |
| Marks | All mark positions |
| Score | Total widgets, ingots, ore produced |
| Simulation | Speed and tick count |
| Tutorial state | Current level and completed levels |

### Save Commands

| Command | Action |
|---------|--------|
| `:w` | Save to default path |
| `:w <path>` | Save to custom path |
| `:wq` or `:x` | Save and quit |
| `ZZ` | Save and quit |
| `:e <path>` | Load from file |
| `ZQ` | Quit without saving |
| `:q!` | Quit without saving |

---

## Technical Reference

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| ratatui | 0.29 | Terminal UI framework |
| crossterm | 0.28 | Terminal raw mode, events |
| hecs | 0.10 | Entity Component System |
| serde | 1 | Serialization derive macros |
| serde_json | 1 | JSON save file format |

### ECS Components

| Component | Used By | Fields |
|-----------|---------|--------|
| Position | All entities | x, y |
| EntityKind | All entities | kind: EntityType |
| FacingComponent | Smelter, Assembler, Conveyor, Splitter, Merger | facing: Facing |
| Processing | Smelter, Assembler | ticks_remaining, input_a, input_b, output |
| OreEmitter | OreDeposit | interval, ticks_since_emit |
| OutputCounter | OutputBin | ore_count, ingot_count, widget_count |
| SplitterState | Splitter | next_output: A or B |
| MergerState | Merger | priority: InputA or InputB |
| PlayerPlaced | Any player-placed entity | (marker, no fields) |

### Module Structure

```
src/
  main.rs          - Entry point, event loop, key handling, rendering
  lib.rs           - Module declarations
  app.rs           - AppState, AppMode, PopupKind
  commands.rs      - Command enum (44 variants), Operator, Range, Blueprint
  resources.rs     - EntityType, Resource, Facing, Direction, input/output sides
  ecs/
    components.rs  - All ECS component structs
    archetypes.rs  - Entity spawning functions
    systems.rs     - Simulation tick (7 steps), SimConfig
  map/
    grid.rs        - Map struct, Tile struct, entity/resource CRUD
    query.rs       - Spatial queries (find next/prev entity, paragraphs, flood fill)
    save.rs        - SaveData, save_game(), load_game()
  game/
    connections.rs - ConnectionGraph for % motion
    undo.rs        - UndoStack with full MapSnapshot (max 100 depth)
    simulation.rs  - Simulation timing, speed control, pause/resume/step
  vim/
    parser.rs      - VimParser state machine (7 modes, 15 awaiting states)
    registers.rs   - RegisterStore (named, unnamed, yank, delete)
    marks.rs       - MarkStore (named marks, previous jump)
    macros.rs      - MacroSystem (100 recursion limit)
    search.rs      - SearchState (pattern, matches, direction)
    dot.rs         - DotRepeat (normal edits, insert sessions)
    motions.rs     - Pure motion resolution functions
    operators.rs   - yank_range, delete_range, rotate_cw/ccw_range
    text_objects.rs - inner/around word, paragraph, block
  input/
    handler.rs     - InputState, command execution (70+ match arms)
  render/
    glyphs.rs      - Entity/resource glyphs and colors
    viewport.rs    - Viewport camera with scroll margin (5 tiles)
    splits.rs      - SplitManager (max 4 panes)
    highlights.rs  - 7 highlight types with priority
    animations.rs  - Flash animations (3 frames)
  ui/
    layout.rs      - Screen layout computation
    grid_render.rs - Tile rendering (2 chars per tile)
    sidebar.rs     - 16-column info sidebar
    statusbar.rs   - Mode indicator, pending keys, cursor position
    popup.rs       - Help/Stats/Registers/Marks popups
    menu.rs        - Main menu (Tutorial/Freeplay/Load/Quit)
    tutorial_bar.rs - 2-row tutorial hint bar
  levels/
    config.rs      - LevelConfig, CompletionCondition, get_level()
    level_01.rs - level_13.rs - Individual level configurations
    freeplay.rs    - Freeplay sandbox map
  tutorial/
    engine.rs      - TutorialState, completion checking, level progression
    hints.rs       - Hint/objective/name accessors
  progression/
    tracker.rs     - ProgressionTracker, 25 command categories
tests/
  simulation_tests.rs - 7 tests (ore emission, conveyors, blocking, smelter, output bin, splitter, full chain)
  vim_parser_tests.rs - 26 tests (movement, counts, modes, operators, marks, search, UI)
```

### Test Coverage

**Simulation Tests (7):**
- Ore deposit emission timing (4-tick interval)
- Conveyor resource movement (1 tile/tick)
- Conveyor blocking when destination occupied
- Smelter ore consumption and ingot production
- Output bin resource consumption and counting
- Splitter alternating output routing
- Full end-to-end ore-to-output chain (50 ticks)

**Vim Parser Tests (26):**
- hjkl movement and count multipliers
- w/b entity jumping
- 0/$ line bounds
- gg/G map navigation
- Insert mode enter/exit and entity placement (smelter, conveyor)
- dd/yy linewise operators
- Visual mode enter/exit
- Command mode and search mode entry
- Undo (u) and redo (Ctrl-r)
- x delete with count (x, 3x)
- ~ toggle facing
- p/P paste before/after
- Mark setting (m + char)
- Search navigation (n, N, *)
- Dot repeat (.)
- Ctrl-g sidebar toggle

**Viewport Tests (3):**
- Viewport position calculations
- Cursor following with scroll margin
- Map boundary clamping
