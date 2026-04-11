use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct DifficultyWall {
    pub level: u32,
    pub name: &'static str,
    pub effect: &'static str,
}

static WALLS: &[DifficultyWall] = &[
    DifficultyWall {
        level: 10,
        name: "Oil Crisis",
        effect: "Oil depletion rate doubled",
    },
    DifficultyWall {
        level: 20,
        name: "Carbon Regulations",
        effect: "Carbon tax tripled",
    },
    DifficultyWall {
        level: 30,
        name: "Labor Shortage",
        effect: "Wages doubled",
    },
    DifficultyWall {
        level: 40,
        name: "Supply Shock",
        effect: "Random deposit exhaustion",
    },
    DifficultyWall {
        level: 50,
        name: "Global Competition",
        effect: "Basic goods prices halved",
    },
    DifficultyWall {
        level: 60,
        name: "Energy Crisis",
        effect: "Power costs tripled",
    },
    DifficultyWall {
        level: 70,
        name: "Waste Emergency",
        effect: "Disposal costs x5",
    },
    DifficultyWall {
        level: 80,
        name: "The Squeeze",
        effect: "Loan rates +10%",
    },
    DifficultyWall {
        level: 90,
        name: "Dark Times",
        effect: "Random breakdowns every cycle",
    },
    DifficultyWall {
        level: 100,
        name: "Singularity",
        effect: "All effects stack and accelerate",
    },
];

/// Get all difficulty walls active at the given scaling level.
pub fn get_active_walls(level: u32) -> Vec<&'static DifficultyWall> {
    WALLS.iter().filter(|w| level >= w.level).collect()
}

/// Modifiers applied by difficulty walls.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WallModifiers {
    pub oil_depletion_multiplier: f64,
    pub carbon_tax_multiplier: f64,
    pub wage_multiplier: f64,
    pub supply_shock_active: bool,
    pub basic_goods_price_multiplier: f64,
    pub power_cost_multiplier: f64,
    pub waste_disposal_multiplier: f64,
    pub loan_rate_bonus: f64,
    pub random_breakdowns: bool,
    pub singularity_active: bool,
}

impl Default for WallModifiers {
    fn default() -> Self {
        Self {
            oil_depletion_multiplier: 1.0,
            carbon_tax_multiplier: 1.0,
            wage_multiplier: 1.0,
            supply_shock_active: false,
            basic_goods_price_multiplier: 1.0,
            power_cost_multiplier: 1.0,
            waste_disposal_multiplier: 1.0,
            loan_rate_bonus: 0.0,
            random_breakdowns: false,
            singularity_active: false,
        }
    }
}

/// Compute wall modifiers for the given scaling level.
pub fn apply_wall_modifiers(level: u32) -> WallModifiers {
    let mut m = WallModifiers::default();

    // Singularity: stack count for levels >= 100
    let singularity_stacks = if level >= 100 {
        m.singularity_active = true;
        1 + (level - 100) / 10
    } else {
        0
    };

    let mult = |base: f64| -> f64 {
        if singularity_stacks > 0 {
            base * (1.0 + singularity_stacks as f64 * 0.2)
        } else {
            base
        }
    };

    if level >= 10 {
        m.oil_depletion_multiplier = mult(2.0);
    }
    if level >= 20 {
        m.carbon_tax_multiplier = mult(3.0);
    }
    if level >= 30 {
        m.wage_multiplier = mult(2.0);
    }
    if level >= 40 {
        m.supply_shock_active = true;
    }
    if level >= 50 {
        m.basic_goods_price_multiplier = mult(0.5);
    }
    if level >= 60 {
        m.power_cost_multiplier = mult(3.0);
    }
    if level >= 70 {
        m.waste_disposal_multiplier = mult(5.0);
    }
    if level >= 80 {
        m.loan_rate_bonus = mult(0.10);
    }
    if level >= 90 {
        m.random_breakdowns = true;
    }

    m
}
