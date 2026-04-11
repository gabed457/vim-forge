use serde::{Deserialize, Serialize};

use crate::resources::Resource;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Inventory {
    pub ore: u64,
    pub ingot: u64,
    pub widget: u64,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory::default()
    }

    pub fn add(&mut self, resource: Resource) {
        match resource {
            Resource::Ore => self.ore += 1,
            Resource::Ingot => self.ingot += 1,
            Resource::Widget => self.widget += 1,
        }
    }

    pub fn total(&self) -> u64 {
        self.ore + self.ingot + self.widget
    }
}
