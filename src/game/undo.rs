use hecs::World;

use crate::ecs::archetypes;
use crate::ecs::components::*;
use crate::game::inventory::Inventory;
use crate::map::grid::Map;
use crate::resources::{EntityType, Facing, Resource};

const MAX_UNDO_DEPTH: usize = 100;

#[derive(Clone, Debug)]
pub struct MapSnapshot {
    pub tiles_resources: Vec<Vec<Option<Resource>>>,
    pub entities: Vec<SnapshotEntity>,
    pub inventory: Inventory,
}

#[derive(Clone, Debug)]
pub struct SnapshotEntity {
    pub x: usize,
    pub y: usize,
    pub entity_type: EntityType,
    pub facing: Facing,
    pub processing: Option<ProcessingSnapshot>,
    pub ore_emitter: Option<OreEmitterSnapshot>,
    pub output_counter: Option<OutputCounterSnapshot>,
    pub splitter_state: Option<SplitterOutput>,
    pub merger_state: Option<MergerPriority>,
    pub player_placed: bool,
}

#[derive(Clone, Debug)]
pub struct ProcessingSnapshot {
    pub ticks_remaining: u32,
    pub input_a: Option<Resource>,
    pub input_b: Option<Resource>,
    pub output: Option<Resource>,
}

#[derive(Clone, Debug)]
pub struct OreEmitterSnapshot {
    pub interval: u32,
    pub ticks_since_emit: u32,
}

#[derive(Clone, Debug)]
pub struct OutputCounterSnapshot {
    pub ore_count: u64,
    pub ingot_count: u64,
    pub widget_count: u64,
}

pub struct UndoStack {
    undo: Vec<MapSnapshot>,
    redo: Vec<MapSnapshot>,
}

impl UndoStack {
    pub fn new() -> Self {
        UndoStack {
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    /// Take a snapshot of current state and push to undo stack.
    pub fn push_snapshot(&mut self, world: &World, map: &Map, inventory: &Inventory) {
        let snapshot = capture_snapshot(world, map, inventory);
        self.undo.push(snapshot);
        if self.undo.len() > MAX_UNDO_DEPTH {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Undo: pop from undo, push current to redo, restore popped.
    pub fn undo(&mut self, world: &mut World, map: &mut Map, inventory: &mut Inventory) -> bool {
        if let Some(snapshot) = self.undo.pop() {
            let current = capture_snapshot(world, map, inventory);
            self.redo.push(current);
            restore_snapshot(world, map, inventory, &snapshot);
            true
        } else {
            false
        }
    }

    /// Redo: pop from redo, push current to undo, restore popped.
    pub fn redo(&mut self, world: &mut World, map: &mut Map, inventory: &mut Inventory) -> bool {
        if let Some(snapshot) = self.redo.pop() {
            let current = capture_snapshot(world, map, inventory);
            self.undo.push(current);
            restore_snapshot(world, map, inventory, &snapshot);
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

fn capture_snapshot(world: &World, map: &Map, inventory: &Inventory) -> MapSnapshot {
    let tiles_resources = map
        .tiles
        .iter()
        .map(|row| row.iter().map(|t| t.resource).collect())
        .collect();

    let mut entities = Vec::new();
    for (entity, (pos, kind)) in world.query::<(&Position, &EntityKind)>().iter() {
        let facing = world
            .get::<&FacingComponent>(entity)
            .ok()
            .map(|f| f.facing)
            .unwrap_or(Facing::Right);

        let processing = world.get::<&Processing>(entity).ok().map(|p| {
            ProcessingSnapshot {
                ticks_remaining: p.ticks_remaining,
                input_a: p.input_a,
                input_b: p.input_b,
                output: p.output,
            }
        });

        let ore_emitter = world.get::<&OreEmitter>(entity).ok().map(|e| {
            OreEmitterSnapshot {
                interval: e.interval,
                ticks_since_emit: e.ticks_since_emit,
            }
        });

        let output_counter = world.get::<&OutputCounter>(entity).ok().map(|c| {
            OutputCounterSnapshot {
                ore_count: c.ore_count,
                ingot_count: c.ingot_count,
                widget_count: c.widget_count,
            }
        });

        let splitter_state = world
            .get::<&SplitterState>(entity)
            .ok()
            .map(|s| s.next_output);

        let merger_state = world
            .get::<&MergerState>(entity)
            .ok()
            .map(|s| s.priority);

        let player_placed = world.get::<&PlayerPlaced>(entity).is_ok();

        entities.push(SnapshotEntity {
            x: pos.x,
            y: pos.y,
            entity_type: kind.kind,
            facing,
            processing,
            ore_emitter,
            output_counter,
            splitter_state,
            merger_state,
            player_placed,
        });
    }

    MapSnapshot {
        tiles_resources,
        entities,
        inventory: inventory.clone(),
    }
}

fn restore_snapshot(world: &mut World, map: &mut Map, inventory: &mut Inventory, snapshot: &MapSnapshot) {
    *inventory = snapshot.inventory.clone();
    // Clear all entities from world
    let all_entities: Vec<hecs::Entity> = world.iter().map(|e| e.entity()).collect();
    for entity in all_entities {
        let _ = world.despawn(entity);
    }

    // Reset map
    for row in &mut map.tiles {
        for tile in row {
            tile.entity = None;
            tile.resource = None;
        }
    }

    // Restore resources
    for (y, row) in snapshot.tiles_resources.iter().enumerate() {
        for (x, resource) in row.iter().enumerate() {
            if let Some(r) = resource {
                map.set_resource(x, y, *r);
            }
        }
    }

    // Restore entities
    for se in &snapshot.entities {
        let entity = archetypes::spawn_entity(
            world,
            se.entity_type,
            se.x,
            se.y,
            se.facing,
            se.player_placed,
        );
        map.set_entity(se.x, se.y, entity);

        // Restore processing state
        if let Some(ps) = &se.processing {
            if let Ok(mut proc) = world.get::<&mut Processing>(entity) {
                proc.ticks_remaining = ps.ticks_remaining;
                proc.input_a = ps.input_a;
                proc.input_b = ps.input_b;
                proc.output = ps.output;
            }
        }

        // Restore ore emitter state
        if let Some(oe) = &se.ore_emitter {
            if let Ok(mut emitter) = world.get::<&mut OreEmitter>(entity) {
                emitter.interval = oe.interval;
                emitter.ticks_since_emit = oe.ticks_since_emit;
            }
        }

        // Restore output counter
        if let Some(oc) = &se.output_counter {
            if let Ok(mut counter) = world.get::<&mut OutputCounter>(entity) {
                counter.ore_count = oc.ore_count;
                counter.ingot_count = oc.ingot_count;
                counter.widget_count = oc.widget_count;
            }
        }

        // Restore splitter state
        if let Some(ss) = &se.splitter_state {
            if let Ok(mut state) = world.get::<&mut SplitterState>(entity) {
                state.next_output = *ss;
            }
        }

        // Restore merger state
        if let Some(ms) = &se.merger_state {
            if let Ok(mut state) = world.get::<&mut MergerState>(entity) {
                state.priority = *ms;
            }
        }
    }
}
