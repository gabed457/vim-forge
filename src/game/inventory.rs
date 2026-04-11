use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::resources::Resource;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    #[serde(default)]
    pub counts: HashMap<Resource, u64>,
}

impl Default for Inventory {
    fn default() -> Self {
        Inventory {
            counts: HashMap::new(),
        }
    }
}

impl Inventory {
    pub fn new() -> Self {
        Inventory::default()
    }

    pub fn add(&mut self, resource: Resource) {
        *self.counts.entry(resource).or_insert(0) += 1;
    }

    pub fn get(&self, resource: Resource) -> u64 {
        self.counts.get(&resource).copied().unwrap_or(0)
    }

    pub fn total(&self) -> u64 {
        self.counts.values().sum()
    }
}
