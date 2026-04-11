use std::time::{Duration, Instant};

use hecs::World;

use crate::ecs::systems::{self, SimConfig};
use crate::map::grid::Map;

pub struct Simulation {
    pub config: SimConfig,
    pub speed: u32, // ticks per second, 0 = paused
    pub tick_count: u64,
    last_tick: Instant,
    last_speed_before_pause: u32,
}

impl Simulation {
    pub fn new() -> Self {
        Simulation {
            config: SimConfig::default_config(),
            speed: 5,
            tick_count: 0,
            last_tick: Instant::now(),
            last_speed_before_pause: 5,
        }
    }

    pub fn is_paused(&self) -> bool {
        self.speed == 0
    }

    pub fn set_speed(&mut self, speed: u32) {
        if speed > 0 {
            self.last_speed_before_pause = speed;
        }
        self.speed = speed.min(20);
    }

    pub fn pause(&mut self) {
        if self.speed > 0 {
            self.last_speed_before_pause = self.speed;
        }
        self.speed = 0;
    }

    pub fn resume(&mut self) {
        if self.speed == 0 {
            self.speed = self.last_speed_before_pause;
        }
    }

    /// Returns true if a tick should occur.
    pub fn should_tick(&self) -> bool {
        if self.speed == 0 {
            return false;
        }
        let interval = Duration::from_millis(1000 / self.speed as u64);
        self.last_tick.elapsed() >= interval
    }

    /// Process a tick if enough time has elapsed.
    pub fn update(&mut self, world: &mut World, map: &mut Map) -> bool {
        if !self.should_tick() {
            return false;
        }
        self.do_tick(world, map);
        true
    }

    /// Force a single tick (used by :step command).
    pub fn step(&mut self, world: &mut World, map: &mut Map) {
        self.do_tick(world, map);
    }

    fn do_tick(&mut self, world: &mut World, map: &mut Map) {
        systems::tick(world, map, &self.config);
        self.tick_count += 1;
        self.last_tick = Instant::now();
    }
}
