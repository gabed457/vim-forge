```
 __     __ ___  __  __     _____ ___  ____   ____ _____
 \ \   / /|_ _||  \/  |   |  ___/ _ \|  _ \ / ___| ____|
  \ \ / /  | | | |\/| |   | |_ | | | | |_) | |  _|  _|
   \ V /   | | | |  | |   |  _|| |_| |  _ <| |_| | |___
    \_/   |___||_|  |_|   |_|   \___/|_| \_\\____|_____|
```

**What if Factorio spoke Vim?**

VimForge is a terminal factory-builder where every action — placing machines, routing belts, demolishing walls, copying blueprints — is a Vim command. No mouse. No menus. Just keystrokes.

You don't click a conveyor belt onto the map. You press `i` to enter Insert mode, `c` to select Conveyors, `b` for Basic Belt, then type your facing direction. You delete a row of machines with `dd`. You yank a blueprint with `yy` and paste it with `p`. You search for an entity with `/`. You undo with `u`.

If you know Vim, you already know how to play. If you don't — you're about to learn.

---

## The Game

You start with iron ore and a dream. Mine raw materials, smelt them into ingots, assemble them into circuit boards, processors, rocket fuel, and eventually — a Dyson Sphere.

**75+ resources** flow through your factory across 6 tiers:

| Tier | Examples |
|------|----------|
| 0 — Raw | Iron ore, copper ore, crude oil, uranium, quartz sand |
| 1 — Basic | Steel, glass, plastic, rubber, sulfuric acid |
| 2 — Intermediate | Circuit boards, gears, silicon wafers, lithium cells |
| 3 — Advanced | Processors, servos, nuclear fuel rods, rocket fuel |
| 4 — Mega | Quantum processors, fusion cores, antimatter capsules |
| 5 — Victory | Space elevator segments, Dyson swarm clusters, warp gate components |

Every tier of production generates **waste** — slag, CO2, toxic sludge, nuclear waste, dimensional residue. Ignore it and your factory chokes. Pollution rises (0–1000 scale), regulations tighten, machines degrade, and eventually your permits get revoked.

**90+ building types** span extractors, 3 tiers of conveyor belts, fluid pipes with pressure physics, smelters, assemblers, chemical plants, refineries, nuclear reactors, research labs, power generators, storage, logistics, transport networks (trains, trucks, drones, planes with A* pathfinding), circuit networks (red/green wires, combinators), and waste processing.

## The Vim Grammar

The control scheme is real Vim grammar, not "Vim-inspired hotkeys." Operators compose with motions and text objects just like in the editor:

```
d3j       — demolish 3 rows down
y$        — yank from cursor to end of row
diw       — delete inner word (connected cluster)
ciB       — change inner block (rectangular region)
"ayy      — yank current row into register 'a'
"ap       — paste register 'a'
3dd       — demolish 3 lines
.         — repeat last edit
/furnace  — search for furnaces
n         — jump to next match
ma        — set mark 'a'
'a        — jump to mark 'a'
qa...q    — record macro into 'a'
@a        — play macro 'a'
```

**Insert mode** uses a hierarchical two-key menu. Press `i` to enter Insert, then:

```
c → Conveyors     (b=basic, f=fast, e=express)
p → Pipes         (p=pipe, u=pump, v=valve, t=tank)
1 → Processing T1 (f=furnace, s=smelter, c=crusher)
2 → Processing T2 (a=assembler, m=mixer, w=welder)
r → Research      (l=lab, a=adv lab)
g → Power         (c=coal gen, s=solar, w=wind, n=nuclear)
...and 15 categories total
```

**Command mode** (`:`) has its own vocabulary:

```
:w            — save
:q            — quit
:contracts    — open contract board
:market       — view market prices
:research     — open research tree
:finance      — financial report
:sell          — sell inventory
:prestige     — prestige reset
:level 3      — jump to level 3
:help motion  — contextual help
```

## Systems

**Economy** — Cash ledger with income from sales and contract deliveries. Expenses for power, wages, maintenance, raw materials. Take out loans (careful with the interest). Market prices shift with supply and demand.

**Contracts** — 6 tiers of delivery contracts. "Deliver 50 iron ingots" at Tier 1 becomes "Deliver 200 quantum processors" at Tier 6. Three active slots, five on the board, scaling rewards.

**Research** — 57 technologies across 5 science pack tiers. Unlock faster belts, advanced buildings, nuclear power, logistics drones, infinite research for scaling bonuses.

**Power Grid** — Coal generators, solar panels (affected by day/night cycle), wind turbines, geothermal, nuclear reactors, fusion. Grid uses BFS-connected components — isolated sections need their own generation.

**Fluids** — Pressure-based pipe simulation. Pumps push fluid through networks, valves control flow, tanks buffer storage. Pipe junctions split flow proportionally.

**Day/Night** — 600-tick cycle. Solar output follows a curve. Weather events (storms, heat waves, cold snaps, acid rain) affect production and machine health.

**Scaling** — Difficulty ratchets up through 10 walls. Regulations tighten, waste thresholds shrink, operating costs climb. When you've hit the ceiling — **prestige**. Reset the map, keep permanent bonuses (speed, cost reduction, pollution resistance, capacity, luck, XP), climb higher.

## Running

```bash
cargo run
```

Dependencies: `ratatui`, `crossterm`, `hecs`, `serde`, `serde_json`. That's it.

Runs in any terminal that supports 256-color or truecolor. Tested on macOS, Linux, and Windows Terminal.

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

137 tests covering the parser, simulation, economy, contracts, scaling, fluids, power, waste, multi-tile buildings, and recipes.

## Architecture

- **ECS**: `hecs` — entities are composed from small components, systems iterate over them each tick
- **Vim Parser**: State machine in `src/vim/parser.rs` — not a match block, a proper `(Mode, Awaiting)` state machine that composes operators, counts, motions, and registers
- **Rendering**: `ratatui` — all `Color::Rgb`, biome palettes, particle effects, animated belt glyphs, day/night tinting. Rendering never mutates game state.
- **Undo**: Every edit goes through the undo stack. `u` and `Ctrl-R` work exactly like Vim.

```
src/
├── app.rs              # AppState, Mode, PopupKind
├── commands.rs         # Command enum — every possible action
├── resources.rs        # Resource (75+), EntityType (90+), Facing, Direction
├── vim/
│   ├── parser.rs       # Vim state machine
│   ├── registers.rs    # Named registers (a-z, ", 0-9)
│   ├── marks.rs        # Mark store
│   ├── macros.rs       # Macro recording/playback
│   └── search.rs       # / and ? search
├── ecs/
│   ├── components.rs   # Position, Processing, MultiTile, etc.
│   ├── archetypes.rs   # Entity spawning
│   ├── systems.rs      # Tick-based simulation
│   └── recipes.rs      # Recipe registry
├── map/
│   ├── grid.rs         # Tile map with entity + resource layers
│   ├── terrain.rs      # Terrain types (water, forest, lava, etc.)
│   └── multitile.rs    # Multi-tile footprint system
├── fluid/              # Pressure-based pipe simulation
├── power/              # Power grid, generators, batteries
├── waste/              # Pollution, degradation, nuclear leaks
├── transport/          # Trains, trucks, drones, planes, A* pathfinding
├── circuit/            # Red/green wire networks, combinators
├── environment/        # Day/night cycle, weather
├── economy/            # Cash ledger, expenses, loans
├── contracts/          # Contract board, delivery matching
├── market/             # Supply/demand price model
├── research/           # Tech tree (57 techs), labs, science packs
├── scaling/            # Difficulty walls, regulations, prestige
├── render/             # Glyphs, colors, particles, trails, animations
├── input/              # Command execution (handler.rs)
├── ui/                 # Menu, sidebar, statusbar, popups
└── game/               # Simulation driver, undo, inventory, save/load
```

## License

MIT
