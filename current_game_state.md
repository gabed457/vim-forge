# VimForge - Current Game State

A terminal TUI factory/automation game where you build production lines using Vim keybinding grammar as the control scheme. Built with Rust, ratatui, crossterm, and hecs (ECS).

---

## Table of Contents

1. [High-Level Game Concept](#1-high-level-game-concept)
2. [Architecture Overview](#2-architecture-overview)
3. [Resources & Production Chain](#3-resources--production-chain)
4. [Entity Types](#4-entity-types)
5. [Vim Input System](#5-vim-input-system)
6. [Modes](#6-modes)
7. [Movement / Motions](#7-movement--motions)
8. [Operators](#8-operators)
9. [Insert Mode (Building)](#9-insert-mode-building)
10. [Visual Mode (Selection)](#10-visual-mode-selection)
11. [Registers (Copy/Paste)](#11-registers-copypaste)
12. [Marks](#12-marks)
13. [Macros](#13-macros)
14. [Search](#14-search)
15. [Dot Repeat](#15-dot-repeat)
16. [Undo/Redo](#16-undoredo)
17. [Command Mode](#17-command-mode)
18. [Simulation Engine](#18-simulation-engine)
19. [Map & Grid System](#19-map--grid-system)
20. [Connection Graph](#20-connection-graph)
21. [Inventory System](#21-inventory-system)
22. [Tutorial & Level System](#22-tutorial--level-system)
23. [Level Catalog](#23-level-catalog)
24. [Progression & Completion](#24-progression--completion)
25. [Rendering & UI](#25-rendering--ui)
26. [Save/Load System](#26-saveload-system)
27. [ECS Component Model](#27-ecs-component-model)
28. [Application Loop](#28-application-loop)

---

## 1. High-Level Game Concept

VimForge is a factory game played entirely in the terminal. The player views a 2D grid map and must build production lines that convert raw ore into finished widgets. The twist: all interaction uses Vim editor keybindings. Moving the cursor is `hjkl`, placing machines uses insert mode, deleting machines uses `d` + motion, copying factory layouts uses `y`/`p`, and so on.

The game has 13 tutorial levels that progressively teach Vim commands through factory-building puzzles, plus an unlockable Freeplay sandbox.

---

## 2. Architecture Overview

```
src/
  main.rs           — Entry point, terminal setup, game loop
  app.rs            — Central AppState struct, Mode/Popup enums
  commands.rs       — Command enum (all possible actions), Range, Blueprint types
  resources.rs      — Resource/EntityType/Facing/Direction enums, input/output side logic
  lib.rs            — Module declarations

  vim/
    parser.rs       — State machine: KeyEvent -> Vec<Command>
    motions.rs      — Cursor movement math (direction, map start/end)
    operators.rs    — Delete/yank/rotate range operations on the map
    text_objects.rs — Text object stubs (iw, ap, etc.)
    dot.rs          — Dot repeat recording/playback
    macros.rs       — Macro recording/playback with recursion limits
    marks.rs        — Named position marks (a-z) + previous jump mark
    registers.rs    — Named registers storing Blueprints or Macro keystrokes
    search.rs       — Entity search by type name

  ecs/
    components.rs   — All ECS components (Position, EntityKind, Processing, etc.)
    archetypes.rs   — Entity spawn functions (one per entity type)
    systems.rs      — Simulation tick: ore emit, machine I/O, conveyor, splitter, merger

  game/
    simulation.rs   — Time-based tick scheduling (speed, pause, step)
    undo.rs         — Snapshot-based undo/redo stack (max 100 deep)
    connections.rs  — Connection graph between entities, conveyor chain following
    inventory.rs    — Player resource inventory

  map/
    grid.rs         — Map struct: 2D tile grid, entity placement, neighbor lookup
    query.rs        — Map query helpers (find next/prev entity, paragraph, etc.)
    save.rs         — JSON save/load with SaveData struct

  input/
    handler.rs      — InputState: bridges parser -> game state, executes commands

  levels/
    config.rs       — LevelConfig, CompletionCondition, level registry
    level_01..13.rs — Individual level definitions
    freeplay.rs     — Freeplay sandbox config

  tutorial/
    engine.rs       — TutorialState: completion checking, hint progression
    hints.rs        — Hint text helpers

  render/
    glyphs.rs       — Entity/resource display characters and colors
    animations.rs   — Animation manager (tick-based)
    highlights.rs   — Search/visual selection highlighting
    viewport.rs     — Scrollable viewport (camera follow)
    splits.rs       — Split pane manager (stub)

  ui/
    layout.rs       — Terminal layout computation (grid, sidebar, statusbar areas)
    grid_render.rs  — Main game grid rendering
    sidebar.rs      — Entity info sidebar
    statusbar.rs    — Vim-style status bar (mode, position, pending keys)
    menu.rs         — Main menu screen
    popup.rs        — Popup overlays (help, stats, registers, marks)
    tutorial_bar.rs — Tutorial hint bar at top
```

**Key design rules:**
- Rendering never mutates game state
- All editable actions go through the undo system
- The vim parser is a state machine, not a giant match block
- Only dependencies: ratatui, crossterm, hecs, serde, serde_json

---

## 3. Resources & Production Chain

Three resource types flow through the factory:

| Resource | Glyph | Color | Description |
|----------|-------|-------|-------------|
| Ore | `o` | Gold/brown | Raw material, emitted by OreDeposit |
| Ingot | `i` | Silver/white | Smelted from 1 Ore |
| Widget | `w` | Green (bold) | Assembled from 2 Ingots |

**Production chain:**
```
OreDeposit -> [Ore] -> Smelter -> [Ingot] -> Assembler -> [Widget] -> OutputBin
                                                ^
                                                |
                               Smelter -> [Ingot]
```

Resources exist as items sitting on map tiles. They are moved between entities by conveyors and consumed/produced by machines.

---

## 4. Entity Types

### OreDeposit (glyph: `O`, color: gold, bold)
- **Input sides:** None
- **Output sides:** All 4 directions
- **Behavior:** Emits 1 Ore every N ticks (default interval: 4) to an adjacent entity that can receive it
- **Components:** Position, EntityKind, OreEmitter
- **Player-placeable:** No

### Smelter (glyph: `S`, color: red, bold)
- **Input sides:** Opposite of facing (back)
- **Output sides:** Facing direction (front)
- **Behavior:** Consumes 1 Ore, processes for 3 ticks, produces 1 Ingot
- **Components:** Position, EntityKind, FacingComponent, Processing
- **Player-placeable:** Yes (insert key: `s`)

### Assembler (glyph: `A`, color: cyan, bold)
- **Input sides:** Both perpendicular sides (left + right if facing up/down)
- **Output sides:** Facing direction (front)
- **Behavior:** Consumes 2 Ingots (one from each side), processes for 5 ticks, produces 1 Widget
- **Components:** Position, EntityKind, FacingComponent, Processing
- **Player-placeable:** Yes (insert key: `a`)

### Conveyor (glyph: `↑↓←→`, color: white; dimmed when idle)
- **Input sides:** Back + both perpendicular (3 sides total)
- **Output sides:** Facing direction only
- **Behavior:** Moves a resource from its own tile to the next tile in the facing direction. Simultaneous movement with destination claiming to prevent conflicts. Sorted by entity ID for determinism.
- **Components:** Position, EntityKind, FacingComponent
- **Player-placeable:** Yes (insert key: `c`)

### Splitter (glyph: `Y`, color: yellow)
- **Input sides:** Opposite of facing (back)
- **Output sides:** Both perpendicular sides
- **Behavior:** Takes resource from input, alternates output between side A and side B. Toggles after each successful routing.
- **Components:** Position, EntityKind, FacingComponent, SplitterState
- **Player-placeable:** Yes (insert key: `p`)

### Merger (glyph: `λ`, color: yellow)
- **Input sides:** Both perpendicular sides
- **Output sides:** Facing direction
- **Behavior:** Takes resource from one of two input sides, outputs forward. Alternates priority between inputs after each successful merge.
- **Components:** Position, EntityKind, FacingComponent, MergerState
- **Player-placeable:** Yes (insert key: `e`)

### OutputBin (glyph: `B`, color: green, bold)
- **Input sides:** All 4 directions
- **Output sides:** None
- **Behavior:** Consumes any resource delivered to it and increments counters (ore_count, ingot_count, widget_count). Checks own tile first, then adjacent tiles.
- **Components:** Position, EntityKind, OutputCounter
- **Player-placeable:** No

### Wall (glyph: `█`, color: dark gray)
- **Input sides:** None
- **Output sides:** None
- **Behavior:** Blocks tile. No resource interaction.
- **Components:** Position, EntityKind
- **Player-placeable:** Yes (insert key: `w`)

---

## 5. Vim Input System

### Parser State Machine (`src/vim/parser.rs`)

The parser is a state machine with these states tracked via the `Awaiting` enum:

- `Nothing` — Ready for a new command
- `Operator` — An operator key was pressed, waiting for motion
- `Motion` — Waiting for a motion after operator + optional text object prefix
- `FChar` / `FCharBack` / `TChar` / `TCharBack` — Waiting for entity type char after f/F/t/T
- `MarkSet` — Waiting for mark name after `m`
- `MarkJump` / `MarkJumpExact` — Waiting for mark name after `'` or `` ` ``
- `RegisterSelect` — Waiting for register name after `"`
- `SecondG` — Waiting for second `g` in `gg`
- `MacroRecord` — Waiting for register name after `q`
- `MacroPlay` — Waiting for register name after `@`
- `ReplaceChar` — Waiting for entity type after `r`
- `CtrlW` — Waiting for window command after `Ctrl-W`

### Count System

Vim-style counts are supported: `[count1][operator][count2][motion]`. The effective count is `count1 * count2`, each defaulting to 1. Digits accumulate in a buffer; `0` as the first character maps to `LineStart` instead of count.

### Command Flow

```
KeyEvent -> VimParser.handle_key_event() -> Vec<Command>
                                               |
                                               v
                                    InputState.execute_command()
                                               |
                                               v
                                    Mutates: map, world, undo, inventory
```

---

## 6. Modes

| Mode | Display | Description |
|------|---------|-------------|
| Normal | `NORMAL` | Default. Navigation, operators, commands |
| Insert | `INSERT` | Place entities on the map |
| Visual | `VISUAL` | Character-wise selection |
| VisualLine | `V-LINE` | Line-wise (full row) selection |
| VisualBlock | `V-BLOCK` | Rectangular block selection |
| Command | `COMMAND` | `:` command line input |
| Search | `SEARCH` | `/` or `?` search input |
| Menu | — | Main menu (not a vim mode) |

---

## 7. Movement / Motions

All motions work in Normal mode and extend selection in Visual modes.

| Key | Command | Description |
|-----|---------|-------------|
| `h` / `Left` | `Move(Left, N)` | Move left N tiles |
| `j` / `Down` | `Move(Down, N)` | Move down N tiles |
| `k` / `Up` | `Move(Up, N)` | Move up N tiles |
| `l` / `Right` | `Move(Right, N)` | Move right N tiles |
| `w` | `JumpNextEntity(N)` | Jump to next entity (like vim word) |
| `W` | `JumpNextEntityBig(N)` | Jump to next entity of different type |
| `b` | `JumpPrevEntity(N)` | Jump to previous entity |
| `B` | `JumpPrevEntityBig(N)` | Jump to previous entity of different type |
| `e` | `JumpEndCluster` | Jump to end of current entity cluster |
| `0` / `Home` | `LineStart` | Jump to column 0 |
| `$` / `End` | `LineEnd` | Jump to last column |
| `^` | `FirstEntityInRow` | Jump to first entity in current row |
| `gg` | `MapStart(row?)` | Jump to top of map (or row N if count given) |
| `G` | `MapEnd(row?)` | Jump to bottom of map (or row N) |
| `H` | `ViewportTop` | Jump to top visible row |
| `M` | `ViewportMiddle` | Jump to middle visible row |
| `L` | `ViewportBottom` | Jump to bottom visible row |
| `f{char}` | `FindEntity(type, N, fwd)` | Jump forward to Nth entity of type |
| `F{char}` | `FindEntity(type, N, back)` | Jump backward to entity of type |
| `t{char}` | `TilEntity(type, N, fwd)` | Jump to tile before Nth entity of type |
| `T{char}` | `TilEntity(type, N, back)` | Jump to tile after entity of type (backward) |
| `;` | `RepeatFind(true)` | Repeat last f/F/t/T in same direction |
| `,` | `RepeatFind(false)` | Repeat last f/F/t/T in opposite direction |
| `}` | `NextParagraph(N)` | Jump to next empty row gap |
| `{` | `PrevParagraph(N)` | Jump to previous empty row gap |
| `%` | `MatchConnection` | Jump to connected entity (conveyor chain end) |

**Find character mapping** (for `f`/`F`/`t`/`T`/`r`):

| Char | Entity Type |
|------|-------------|
| `s` | Smelter |
| `a` | Assembler |
| `c` | Conveyor |
| `p` | Splitter |
| `m` | Merger |
| `o` | OreDeposit |
| `b` | OutputBin |
| `w` | Wall |

---

## 8. Operators

Operators combine with motions: `[operator][motion]` applies the operator to the range of tiles the motion covers. Double-operator (e.g., `dd`) applies to the current line(s).

| Operator | Key | Line form | Description |
|----------|-----|-----------|-------------|
| Delete/Demolish | `d` | `dd` | Remove all entities in range, store as blueprint in register |
| Yank | `y` | `yy` | Copy entities in range as blueprint into register |
| Change | `c` | `cc` | Delete range then enter Insert mode |
| Rotate CW | `>` | `>>` | Rotate all entity facings clockwise in range |
| Rotate CCW | `<` | `<<` | Rotate all entity facings counter-clockwise in range |

**With register prefix:** `"a` before an operator stores into register `a`. E.g., `"ayy` yanks into register `a`.

**How ranges work:** The `Range` struct holds a `Vec<(usize, usize)>` of tile coordinates and a `linewise` flag. Range constructors:
- `single(x, y)` — One tile
- `horizontal(y, x_start, x_end)` — One row segment
- `linewise_rows(y_start, y_end, width)` — Full rows
- `block(x1, y1, x2, y2)` — Rectangular block

---

## 9. Insert Mode (Building)

Enter with `i` (supports count prefix). Exit with `Esc`.

| Key | Action |
|-----|--------|
| `s` | Place Smelter at cursor, advance cursor in facing direction |
| `a` | Place Assembler |
| `c` | Place Conveyor |
| `p` | Place Splitter |
| `e` | Place Merger |
| `w` | Place Wall |
| `h/j/k/l` (lowercase) | Move cursor without placing (no facing change) |
| `H/J/K/L` (uppercase) | Change insert facing + move cursor |
| Arrow keys | Change insert facing + move cursor |
| `Backspace` | Undo last placement |

**Insert facing:** Determines the direction newly placed entities face. Changed by Shift+HJKL or arrow keys. Shown in the status bar. Also changeable in Normal mode with `~` (rotates CW).

**Entity placement:** Only succeeds if the target tile is empty and in bounds. After placement, cursor auto-advances one tile in the facing direction.

---

## 10. Visual Mode (Selection)

Three visual sub-modes:

| Key | Mode | Selection Shape |
|-----|------|----------------|
| `v` | Visual | Character-wise (reading order from anchor to cursor) |
| `V` | VisualLine | Full rows from anchor row to cursor row |
| `Ctrl-V` | VisualBlock | Rectangular block from anchor to cursor |

**In visual mode:**
- All motion keys extend the selection
- `o` swaps anchor and cursor
- `d` / `y` / `c` / `>` / `<` apply operator to selection, then return to Normal
- `p` deletes selection then pastes register content
- Pressing the same visual mode key again (e.g., `v` while in Visual) returns to Normal
- `Esc` cancels

---

## 11. Registers (Copy/Paste)

Registers store either a `Blueprint` (entities with relative offsets) or a `Macro` (recorded keystrokes).

**Register selection:** `"x` before y/d/p uses register `x`.

| Key | Action |
|-----|--------|
| `"xy{motion}` | Yank into register x |
| `"xd{motion}` | Delete into register x |
| `"xp` | Paste from register x after cursor |
| `"xP` | Paste from register x before cursor |
| `p` | Paste from default (unnamed) register |
| `P` | Paste before cursor from default register |

**Blueprint structure:** Each blueprint stores `Vec<BlueprintEntity>` with offset_x, offset_y, entity_type, and facing relative to the top-left of the selection. Also stores width, height, and linewise flag.

**Paste behavior:** Entities are placed at `(cursor_x + offset_x, cursor_y + offset_y)`. Tiles that are already occupied are skipped.

---

## 12. Marks

Named positions on the map. Lowercase letters a-z are available.

| Key | Action |
|-----|--------|
| `m{a-z}` | Set mark at cursor position |
| `'{a-z}` | Jump to row of mark (first entity in that row) |
| `` `{a-z} `` | Jump to exact position of mark |
| `''` | Jump to row of previous jump position |
| ` `` ` | Jump to exact previous jump position |

**Previous jump mark:** Automatically set before `gg`, `G`, mark jumps, and search jumps.

---

## 13. Macros

Record and replay sequences of keystrokes.

| Key | Action |
|-----|--------|
| `q{a-z}` | Start recording into register (lowercase only) |
| `q` (while recording) | Stop recording |
| `@{a-z}` | Play macro from register N times |
| `@@` | Replay last played macro |

**Technical details:**
- All keys pressed during recording (except the stop `q`) are captured
- Playback feeds stored keystrokes back through the parser
- Recursion limit prevents infinite loops (checked per playback)
- Macros stored in the RegisterStore alongside blueprints

---

## 14. Search

Search for entity types by name.

| Key | Action |
|-----|--------|
| `/` | Enter search mode (forward) |
| `?` | Enter search mode (backward) |
| `n` | Jump to next search match (N times) |
| `N` | Jump to previous search match (N times) |
| `*` | Search forward for entity type under cursor |
| `#` | Search backward for entity type under cursor |
| `:noh` | Clear search highlights |

**Search input:** Types an entity name prefix (e.g., "sme" matches Smelter). Uses `EntityType::from_search_prefix()` for fuzzy matching. Enter confirms, Esc cancels. Search history navigable with Up/Down arrows.

---

## 15. Dot Repeat

| Key | Action |
|-----|--------|
| `.` | Repeat the last edit action |

**Implementation (`src/vim/dot.rs`):** Records keystrokes from when insert mode is entered until it is exited. On `.` press, replays those keystrokes through the parser.

---

## 16. Undo/Redo

| Key | Action |
|-----|--------|
| `u` | Undo last edit |
| `Ctrl-R` | Redo |

**Implementation (`src/game/undo.rs`):**
- Full snapshot-based: captures entire map state (all entities, resources, processing states, inventory)
- Max depth: 100 snapshots
- Undo pops from undo stack, pushes current state to redo stack, restores popped snapshot
- Redo is the reverse
- Snapshots are pushed before: entity placement, deletion, rotation, paste, replace, visual operations

---

## 17. Command Mode

Enter with `:`. Type command, press Enter. Supports tab completion and command history (Up/Down).

| Command | Aliases | Description |
|---------|---------|-------------|
| `:w [path]` | `:write` | Save game (optional custom path) |
| `:q` | `:quit` | Quit |
| `:q!` | `:quit!` | Force quit |
| `:wq` | `:x` | Save and quit |
| `:e <path>` | `:edit` | Load a save file |
| `:speed <N>` | — | Set simulation speed (1-20 ticks/sec) |
| `:pause` | — | Pause simulation (speed=0) |
| `:resume` | `:run` | Resume simulation |
| `:step` | — | Advance simulation by one tick |
| `:stats` | — | Show stats popup |
| `:registers` | `:reg` | Show registers popup |
| `:marks` | — | Show marks popup |
| `:mapinfo` | `:map` | Show map dimensions |
| `:help [topic]` | `:h` | Show help popup |
| `:level [N]` | — | Jump to level N |
| `:restart` | — | Restart current level |
| `:freeplay` | — | Enter freeplay mode (if unlocked) |
| `:menu` | — | Return to main menu |
| `:noh` | `:nohlsearch` | Clear search highlighting |
| `:version` | `:ver` | Show version |

**Quick save/quit shortcuts (Normal mode):**
- `ZZ` — Save and quit
- `ZQ` — Quit without saving

---

## 18. Simulation Engine

### Tick Scheduling (`src/game/simulation.rs`)

- Speed: 1-20 ticks per second (0 = paused)
- Default speed: 5 ticks/sec
- Tick interval: `1000ms / speed`
- Updates every frame if enough time has elapsed

### Tick Order (`src/ecs/systems.rs`)

Each tick executes these steps in strict order:

1. **Ore Deposit Emit** — OreDeposits with elapsed intervals place Ore on an adjacent tile that has a receiving entity
2. **Machine Output** — Smelters/Assemblers with completed output push their product to the output side
3. **Conveyor Movement** — All conveyors simultaneously move resources from their tile to the facing tile (destination claiming prevents conflicts)
4. **Splitter Process** — Splitters route input to alternating output sides
5. **Merger Process** — Mergers take from alternating input sides and output forward
6. **Machine Consume** — Smelters consume Ore, Assemblers consume Ingots. Processing timers start.
7. **Output Bin Consume** — OutputBins consume resources on their tile or adjacent tiles
8. **Machine Process Tick** — Decrement processing timers. When done: Smelter produces Ingot, Assembler produces Widget.

### Simulation Config

| Parameter | Default | Description |
|-----------|---------|-------------|
| ore_emit_interval | 4 ticks | How often OreDeposits emit |
| smelter_process_ticks | 3 ticks | Time to smelt Ore -> Ingot |
| assembler_process_ticks | 5 ticks | Time to assemble 2 Ingots -> Widget |

---

## 19. Map & Grid System

### Tile Structure

Each tile has two optional occupants:
- `entity: Option<hecs::Entity>` — A machine/wall/etc.
- `resource: Option<Resource>` — A resource item sitting on the tile

These are independent: a conveyor (entity) can have ore (resource) on it simultaneously.

### Map Operations

- `place_entity_on_map(world, x, y, type, facing, player_placed)` — Spawns ECS entity and sets tile. Fails if tile already has entity.
- `remove_entity_from_map(world, x, y)` — Despawns ECS entity and clears tile.
- `set_resource(x, y, resource)` / `remove_resource(x, y)` — Manage tile resources.
- `neighbor(x, y, facing)` — Returns adjacent coordinates in the given direction (bounds-checked).
- `entity_type_at(world, x, y)` — Looks up entity type via ECS.
- `first_entity_in_row(y)` — Finds leftmost entity in a row.
- `row_has_entities(y)` — Checks if a row contains any entities.

### Query Helpers (`src/map/query.rs`)

- `find_next_entity(x, y)` — Scans right then wrapping to find next entity
- `find_prev_entity(x, y)` — Scans left then wrapping
- `find_next_entity_big(world, x, y)` — Skips to next entity of a different type
- `find_prev_entity_big(world, x, y)` — Same, backward
- `find_end_of_cluster(x, y)` — Finds last entity in a contiguous horizontal cluster
- `find_next_paragraph(y)` / `find_prev_paragraph(y)` — Finds next/prev empty row
- `find_entity_type_forward(world, x, y, type)` — Finds next entity of specific type
- `find_entity_type_backward(world, x, y, type)` — Same, backward

---

## 20. Connection Graph

`src/game/connections.rs` builds a bidirectional graph of which entities are connected (output -> input) based on their positions and facing directions. Used for:

- `%` motion (MatchConnection) — jump to connected entity
- Conveyor chain following — follow a chain forward or backward to its endpoint with loop detection

---

## 21. Inventory System

Simple counter: `{ ore: u64, ingot: u64, widget: u64 }`. Incremented when resources are collected from demolished entities (via `collect_resources_at`). Included in undo snapshots.

---

## 22. Tutorial & Level System

### Level Config (`src/levels/config.rs`)

Each level defines:
- `number` — Level index (1-13, 14=freeplay)
- `name` — Display name
- `map_width`, `map_height` — Grid dimensions
- `entities` — Pre-placed entities (with positions, types, facings, player_placed flags)
- `objective` — Text description shown to player
- `hints` — Progressive hint messages
- `allowed_commands` — Optional whitelist of commands (None = all allowed)
- `completion` — Win condition

### Completion Conditions

| Condition | Description |
|-----------|-------------|
| `ProduceWidgets(N)` | Deliver N widgets to output bins |
| `DeliverOre(N)` | Deliver N ore to output bins |
| `DeliverIngots(N)` | Deliver N ingots to output bins |
| `NavigateToAll(positions)` | Visit all listed (x,y) positions with cursor |
| `UseCommands(cmds)` | Use all listed command types |
| `ScoreInMoves(widgets, max_moves)` | Produce N widgets in at most M edits |
| `Custom(string)` | Externally evaluated condition |

### Tutorial State (`src/tutorial/engine.rs`)

Tracks per-session:
- `current_level` — Active level number
- `levels_completed` — List of completed level numbers
- `current_hint_index` — Which hint to show
- `visited_positions` — Set of (x,y) the cursor has visited
- `commands_used` — Set of command names used
- `edit_count` — Number of edits (placements, deletions, modifications)

**Auto-advance hints:** Hints advance automatically based on `(visited_positions.len() + edit_count * 3) / 5`.

**Level unlocking:** Level N is unlocked if level N-1 is completed. Freeplay unlocks after all 13 levels.

---

## 23. Level Catalog

| # | Name | Objective | Win Condition |
|---|------|-----------|---------------|
| 1 | Movement | Navigate to both Output Bins | `NavigateToAll[(15,2),(15,7)]` |
| 2 | First Placement | Place conveyors between Ore and Bin | `DeliverOre(3)` |
| 3 | Smelting | Build: Ore -> Conveyors -> Smelter -> Conveyors -> Output | `DeliverIngots(3)` |
| 4 | Full Production | Two ore lines -> smelters -> assembler -> widgets | `ProduceWidgets(3)` |
| 5 | Demolish & Rebuild | Fix broken factory using d, x | `DeliverIngots(5)` |
| 6 | Copy That | Build one line, yy to copy, p to paste | `ProduceWidgets(9)` |
| 7 | Blueprints | Use named registers ("a2yy / "ap) | `ProduceWidgets(10)` |
| 8 | Block Select | Visual-Block to copy rectangular section | `ProduceWidgets(6)` |
| 9 | Find & Jump | Navigate large factory with f, /, % | `Custom` |
| 10 | Macro Factory | Record macro qa...q, replay 4@a | `ProduceWidgets(15)` |
| 11 | The Dot | Use ~ and . to fix all conveyors | `Custom` |
| 12 | Marks & Navigation | Set marks, navigate, fix 4 clusters | `ProduceWidgets(4)` |
| 13 | Split View | Use split views, build cross-map chain | `ProduceWidgets(5)` |
| 14 | Freeplay | Sandbox — no restrictions | `Custom(freeplay)` |

---

## 24. Progression & Completion

- On level completion: `tut.complete_level()` advances to the next level
- `start_level()` resets world, map, simulation, undo, inventory; places level entities; creates TutorialState
- After all 13 levels: `freeplay_unlocked = true`
- Completion is checked every simulation tick by querying OutputCounter components

---

## 25. Rendering & UI

### Frame Rate
~30 FPS (`FRAME_DURATION = 33ms`)

### Layout (`src/ui/layout.rs`)

```
+-------------------------------------------+
| Tutorial Hint Bar (if show_tutorial)       |
+-------------------------------+-----------+
|                               |           |
|  Game Grid (viewport)         | Sidebar   |
|  (entities, resources,        | (entity   |
|   cursor, selections)         |  info)    |
|                               |           |
+-------------------------------+-----------+
| Status Bar (mode, pos, pending, message)  |
+-------------------------------------------+
```

Minimum terminal size: 80x24.

### Glyphs & Colors (`src/render/glyphs.rs`)

| Entity | Glyph | Color |
|--------|-------|-------|
| OreDeposit | `O` | Gold (bold) |
| Smelter | `S` | Red (bold) |
| Assembler | `A` | Cyan (bold) |
| Conveyor | `↑↓←→` | White (dimmed when idle) |
| Splitter | `Y` | Yellow |
| Merger | `λ` | Yellow |
| OutputBin | `B` | Green (bold) |
| Wall | `█` | Dark gray |
| Empty tile | `·` | Very dark gray |

Resources on tiles: `o` (gold), `i` (silver), `w` (green bold).

Processing machines show remaining ticks as a digit suffix (e.g., smelter with 2 ticks left).

### Viewport (`src/render/viewport.rs`)

Scrollable camera that follows the cursor. Offset-based: tracks `offset_x`, `offset_y` within the map. Width/height computed from terminal size minus sidebar and tutorial bar.

### Popups (`src/ui/popup.rs`)

Modal overlays for:
- **Help** — Scrollable help text (topic-specific)
- **Stats** — Game statistics
- **Registers** — Contents of all named registers
- **Marks** — All set marks with positions

Dismissed with `Esc` or `q`. Scrollable with `j`/`k`.

### Main Menu (`src/ui/menu.rs`)

Options:
1. Start Tutorial (level 1)
2. Freeplay (if unlocked)
3. Load Save (if save exists)
4. Quit

---

## 26. Save/Load System

### Save Format (`src/map/save.rs`)

JSON serialization via serde. Save data structure:

```
SaveData {
  version: u32,
  map_width, map_height,
  entities: Vec<SavedEntity>,      // All entities with full state
  resources: Vec<SavedResource>,   // Resources on tiles
  registers: HashMap,              // (currently empty in saves)
  marks: HashMap<char, (usize, usize)>,
  score: { total_widgets, total_ingots, total_ore },
  simulation_speed, tick_count,
  inventory: Inventory,
  tutorial_state: Option<TutorialSaveState>,
}
```

Each `SavedEntity` captures: position, type, facing, processing state, ore emitter state, output counts, splitter state, merger state, player_placed flag.

**Default save path:** Platform-specific (via a helper function).

---

## 27. ECS Component Model

Using `hecs` crate. Components are plain structs attached to entities.

| Component | Used By | Data |
|-----------|---------|------|
| `Position` | All entities | x, y |
| `EntityKind` | All entities | EntityType enum |
| `FacingComponent` | Smelter, Assembler, Conveyor, Splitter, Merger | Facing enum |
| `Processing` | Smelter, Assembler | ticks_remaining, input_a, input_b, output |
| `OreEmitter` | OreDeposit | interval, ticks_since_emit |
| `OutputCounter` | OutputBin | ore_count, ingot_count, widget_count |
| `SplitterState` | Splitter | next_output (A or B) |
| `MergerState` | Merger | priority (InputA or InputB) |
| `PlayerPlaced` | Any player-placed entity | Marker (no data) |

Entity archetypes (spawn functions) compose these components per entity type. The `spawn_entity()` dispatcher function routes by EntityType.

---

## 28. Application Loop

```
main()
  -> enable_raw_mode, enter alternate screen
  -> run_app() loop:
      1. terminal.draw(render_frame)    [never mutates state]
      2. event::poll(timeout)
         -> Key event: handle_key()
            - Global: Ctrl-C quits
            - Popup: j/k scroll, Esc/q dismiss
            - Menu: 1/2/3/4 select option
            - Game: feed to InputState.handle_key()
                    -> parser produces Commands
                    -> commands executed against game state
                    -> sync cursor, mode, search, visual state to AppState
         -> Resize event: update viewport dimensions
      3. simulation.update()            [tick if enough time elapsed]
         -> check tutorial completion after each tick
         -> auto-advance to next level if completed
      4. animations.tick()              [frame-based animation updates]
  -> disable_raw_mode, leave alternate screen
```

### State Synchronization

After each key event, these fields are synced from InputState to AppState:
- cursor_x, cursor_y
- pending_keys (command buffer display)
- command_buffer (command line display)
- insert_facing
- recording_macro
- show_sidebar
- mode (translated from parser mode to app mode)
- search state
- visual_anchor
- status_message

---

## Stub / Incomplete Features

These are defined in the code but not fully implemented:

- **Split panes** (`Ctrl-W` commands) — Parser produces commands, handler sets status messages, SplitManager exists but pane rendering is not wired up
- **Text objects** (`diw`, `dap`, etc.) — Parser recognizes `i`/`a` after operator but `_handle_text_object_key` is a partial stub
- **`%` match connection** — Handler shows status message but doesn't perform the jump (ConnectionGraph exists and can follow conveyor chains)
- **`;`/`,` repeat find** — Handler shows status message but doesn't replay the last find
- **Animations** — AnimationManager.tick() is called each frame but no animations are defined
- **Inventory display** — Inventory tracks resources but isn't shown in the UI
