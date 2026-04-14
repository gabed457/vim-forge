use serde::{Deserialize, Serialize};

use crate::resources::{EntityType, Facing};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortType {
    SolidInput,
    SolidOutput,
    FluidInput,
    FluidOutput,
    GasInput,
    GasOutput,
    PlasmaOutput,
    WasteSolid,
    WasteFluid,
    WasteGas,
    WastePlasma,
}

impl PortType {
    pub fn is_input(&self) -> bool {
        matches!(self, PortType::SolidInput | PortType::FluidInput | PortType::GasInput)
    }

    pub fn is_output(&self) -> bool {
        matches!(
            self,
            PortType::SolidOutput
                | PortType::FluidOutput
                | PortType::GasOutput
                | PortType::PlasmaOutput
        )
    }

    pub fn is_waste(&self) -> bool {
        matches!(
            self,
            PortType::WasteSolid | PortType::WasteFluid | PortType::WasteGas | PortType::WastePlasma
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortDefinition {
    pub offset_x: i32,
    pub offset_y: i32,
    pub direction: Facing,
    pub port_type: PortType,
    pub port_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingFootprint {
    pub width: usize,
    pub height: usize,
    pub ports: Vec<PortDefinition>,
}

impl BuildingFootprint {
    pub fn new_1x1() -> Self {
        BuildingFootprint {
            width: 1,
            height: 1,
            ports: vec![],
        }
    }

    /// 1×1 conveyor: input from behind (Left), output forward (Right).
    /// Ports rotate with facing via `rotate_to()`.
    pub fn new_1x1_conveyor() -> Self {
        BuildingFootprint {
            width: 1,
            height: 1,
            ports: vec![
                PortDefinition {
                    offset_x: 0,
                    offset_y: 0,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0,
                    offset_y: 0,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
            ],
        }
    }

    pub fn new_1x1_processor() -> Self {
        BuildingFootprint {
            width: 1,
            height: 1,
            ports: vec![
                PortDefinition {
                    offset_x: 0,
                    offset_y: 0,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0,
                    offset_y: 0,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
            ],
        }
    }

    pub fn new_1x3_assembler() -> Self {
        BuildingFootprint {
            width: 1,
            height: 3,
            ports: vec![
                PortDefinition {
                    offset_x: 0,
                    offset_y: 0,
                    direction: Facing::Up,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0,
                    offset_y: 2,
                    direction: Facing::Down,
                    port_type: PortType::SolidInput,
                    port_index: 1,
                },
                PortDefinition {
                    offset_x: 0,
                    offset_y: 1,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
            ],
        }
    }

    pub fn new_1x5() -> Self {
        BuildingFootprint {
            width: 1,
            height: 5,
            ports: vec![
                PortDefinition {
                    offset_x: 0,
                    offset_y: 0,
                    direction: Facing::Up,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0,
                    offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 1,
                },
                PortDefinition {
                    offset_x: 0,
                    offset_y: 3,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 2,
                },
                PortDefinition {
                    offset_x: 0,
                    offset_y: 2,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
            ],
        }
    }

    pub fn new_1x7() -> Self {
        BuildingFootprint {
            width: 1,
            height: 7,
            ports: vec![
                PortDefinition {
                    offset_x: 0,
                    offset_y: 0,
                    direction: Facing::Up,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0,
                    offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 1,
                },
                PortDefinition {
                    offset_x: 0,
                    offset_y: 3,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 2,
                },
                PortDefinition {
                    offset_x: 0,
                    offset_y: 5,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 3,
                },
                PortDefinition {
                    offset_x: 0,
                    offset_y: 3,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
            ],
        }
    }

    pub fn new_1x9() -> Self {
        BuildingFootprint {
            width: 1,
            height: 9,
            ports: vec![
                PortDefinition {
                    offset_x: 0,
                    offset_y: 0,
                    direction: Facing::Up,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0,
                    offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 1,
                },
                PortDefinition {
                    offset_x: 0,
                    offset_y: 3,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 2,
                },
                PortDefinition {
                    offset_x: 0,
                    offset_y: 5,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 3,
                },
                PortDefinition {
                    offset_x: 0,
                    offset_y: 7,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 4,
                },
                PortDefinition {
                    offset_x: 0,
                    offset_y: 4,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
            ],
        }
    }

    /// Rotate the footprint to match the given facing (base is Facing::Right).
    pub fn rotate_to(&self, facing: Facing) -> BuildingFootprint {
        let rotations = match facing {
            Facing::Right => 0,
            Facing::Down => 1,
            Facing::Left => 2,
            Facing::Up => 3,
        };

        let mut fp = self.clone();
        for _ in 0..rotations {
            let old_w = fp.width;
            let old_h = fp.height;
            fp.width = old_h;
            fp.height = old_w;
            for port in &mut fp.ports {
                let old_x = port.offset_x;
                let old_y = port.offset_y;
                port.offset_x = (old_h as i32 - 1) - old_y;
                port.offset_y = old_x;
                port.direction = port.direction.rotate_cw();
            }
        }
        fp
    }

    pub fn is_1x1(&self) -> bool {
        self.width == 1 && self.height == 1
    }

    pub fn tiles(&self) -> Vec<(usize, usize)> {
        let mut result = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                result.push((x, y));
            }
        }
        result
    }

    // =======================================================================
    // 2D footprint constructors
    // =======================================================================

    /// 3×2 extractor: output port on right edge row 1.
    pub fn new_3x2_extractor() -> Self {
        BuildingFootprint {
            width: 3,
            height: 2,
            ports: vec![
                PortDefinition {
                    offset_x: 2, offset_y: 1,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
            ],
        }
    }

    /// 3×3 single-input processor: input left-center, output right-center,
    /// waste bottom-center.
    pub fn new_3x3_1in() -> Self {
        BuildingFootprint {
            width: 3,
            height: 3,
            ports: vec![
                PortDefinition {
                    offset_x: 0, offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 2, offset_y: 1,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 1, offset_y: 2,
                    direction: Facing::Down,
                    port_type: PortType::WasteSolid,
                    port_index: 0,
                },
            ],
        }
    }

    /// 3×4 dual-input processor: 2 inputs left rows 1-2, output right row 2,
    /// waste bottom-center.
    pub fn new_3x4_2in() -> Self {
        BuildingFootprint {
            width: 3,
            height: 4,
            ports: vec![
                PortDefinition {
                    offset_x: 0, offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 2,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 1,
                },
                PortDefinition {
                    offset_x: 2, offset_y: 2,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 1, offset_y: 3,
                    direction: Facing::Down,
                    port_type: PortType::WasteSolid,
                    port_index: 0,
                },
            ],
        }
    }

    /// 4×4 tier-2 processor: 2 inputs left rows 1-2, output right row 2.
    pub fn new_4x4() -> Self {
        BuildingFootprint {
            width: 4,
            height: 4,
            ports: vec![
                PortDefinition {
                    offset_x: 0, offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 2,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 1,
                },
                PortDefinition {
                    offset_x: 3, offset_y: 2,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
            ],
        }
    }

    /// 5×5 tier-3 processor: 3 inputs left rows 1-3, output right row 2,
    /// waste bottom-center.
    pub fn new_5x5() -> Self {
        BuildingFootprint {
            width: 5,
            height: 5,
            ports: vec![
                PortDefinition {
                    offset_x: 0, offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 2,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 1,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 3,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 2,
                },
                PortDefinition {
                    offset_x: 4, offset_y: 2,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 2, offset_y: 4,
                    direction: Facing::Down,
                    port_type: PortType::WasteSolid,
                    port_index: 0,
                },
            ],
        }
    }

    /// 6×6 tier-4 processor: 4 inputs left rows 1-4, output right row 3.
    pub fn new_6x6() -> Self {
        BuildingFootprint {
            width: 6,
            height: 6,
            ports: vec![
                PortDefinition {
                    offset_x: 0, offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 2,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 1,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 3,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 2,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 4,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 3,
                },
                PortDefinition {
                    offset_x: 5, offset_y: 3,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
            ],
        }
    }

    /// 6×7 tier-4 large processor: 5 inputs left rows 1-5, output right row 3.
    pub fn new_6x7() -> Self {
        BuildingFootprint {
            width: 6,
            height: 7,
            ports: vec![
                PortDefinition {
                    offset_x: 0, offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 2,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 1,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 3,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 2,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 4,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 3,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 5,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 4,
                },
                PortDefinition {
                    offset_x: 5, offset_y: 3,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
            ],
        }
    }

    /// 3×3 splitter: 1 input left-center, 2 outputs right rows 0 and 2.
    pub fn new_3x3_splitter() -> Self {
        BuildingFootprint {
            width: 3,
            height: 3,
            ports: vec![
                PortDefinition {
                    offset_x: 0, offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 2, offset_y: 0,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 2, offset_y: 2,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 1,
                },
            ],
        }
    }

    /// 3×3 merger: 2 inputs left rows 0 and 2, 1 output right-center.
    pub fn new_3x3_merger() -> Self {
        BuildingFootprint {
            width: 3,
            height: 3,
            ports: vec![
                PortDefinition {
                    offset_x: 0, offset_y: 0,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 2,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 1,
                },
                PortDefinition {
                    offset_x: 2, offset_y: 1,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
            ],
        }
    }

    /// 3×2 storage: input ports on left, top, and right edges.
    pub fn new_3x2_storage() -> Self {
        BuildingFootprint {
            width: 3,
            height: 2,
            ports: vec![
                PortDefinition {
                    offset_x: 1, offset_y: 0,
                    direction: Facing::Up,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 1,
                },
                PortDefinition {
                    offset_x: 2, offset_y: 1,
                    direction: Facing::Right,
                    port_type: PortType::SolidInput,
                    port_index: 2,
                },
            ],
        }
    }

    /// 3×3 storage: input ports on all 4 edges.
    pub fn new_3x3_storage() -> Self {
        BuildingFootprint {
            width: 3,
            height: 3,
            ports: vec![
                PortDefinition {
                    offset_x: 1, offset_y: 0,
                    direction: Facing::Up,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 1,
                },
                PortDefinition {
                    offset_x: 2, offset_y: 1,
                    direction: Facing::Right,
                    port_type: PortType::SolidInput,
                    port_index: 2,
                },
                PortDefinition {
                    offset_x: 1, offset_y: 2,
                    direction: Facing::Down,
                    port_type: PortType::SolidInput,
                    port_index: 3,
                },
            ],
        }
    }

    /// 3×3 power generator: fuel input left-center.
    pub fn new_3x3_power() -> Self {
        BuildingFootprint {
            width: 3,
            height: 3,
            ports: vec![
                PortDefinition {
                    offset_x: 0, offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
            ],
        }
    }

    /// 5×5 power plant: fuel input left row 1, coolant input left row 3,
    /// waste bottom-center.
    pub fn new_5x5_power() -> Self {
        BuildingFootprint {
            width: 5,
            height: 5,
            ports: vec![
                PortDefinition {
                    offset_x: 0, offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 3,
                    direction: Facing::Left,
                    port_type: PortType::FluidInput,
                    port_index: 1,
                },
                PortDefinition {
                    offset_x: 2, offset_y: 4,
                    direction: Facing::Down,
                    port_type: PortType::WasteSolid,
                    port_index: 0,
                },
            ],
        }
    }

    /// 3×3 research lab: input left-center, output right-center.
    pub fn new_3x3_lab() -> Self {
        BuildingFootprint {
            width: 3,
            height: 3,
            ports: vec![
                PortDefinition {
                    offset_x: 0, offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 2, offset_y: 1,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
            ],
        }
    }

    /// 4×4 advanced lab: 2 inputs left rows 1-2, output right row 2.
    pub fn new_4x4_lab() -> Self {
        BuildingFootprint {
            width: 4,
            height: 4,
            ports: vec![
                PortDefinition {
                    offset_x: 0, offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 2,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 1,
                },
                PortDefinition {
                    offset_x: 3, offset_y: 2,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
            ],
        }
    }

    /// 6×6 mega lab: 4 inputs left rows 1-4, output right row 3.
    pub fn new_6x6_lab() -> Self {
        BuildingFootprint {
            width: 6,
            height: 6,
            ports: vec![
                PortDefinition {
                    offset_x: 0, offset_y: 1,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 0,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 2,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 1,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 3,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 2,
                },
                PortDefinition {
                    offset_x: 0, offset_y: 4,
                    direction: Facing::Left,
                    port_type: PortType::SolidInput,
                    port_index: 3,
                },
                PortDefinition {
                    offset_x: 5, offset_y: 3,
                    direction: Facing::Right,
                    port_type: PortType::SolidOutput,
                    port_index: 0,
                },
            ],
        }
    }

    /// Validate that all ports are on the building perimeter with
    /// outward-facing directions. Panics on invalid ports.
    pub fn validate_ports(&self) {
        let w = self.width as i32;
        let h = self.height as i32;
        for (i, port) in self.ports.iter().enumerate() {
            let x = port.offset_x;
            let y = port.offset_y;
            assert!(
                x >= 0 && x < w && y >= 0 && y < h,
                "Port {} at ({},{}) is out of bounds for {}x{} building",
                i, x, y, self.width, self.height
            );
            let on_left = x == 0;
            let on_right = x == w - 1;
            let on_top = y == 0;
            let on_bottom = y == h - 1;
            assert!(
                on_left || on_right || on_top || on_bottom,
                "Port {} at ({},{}) is not on the perimeter of {}x{} building",
                i, x, y, self.width, self.height
            );
            let valid_dir = match port.direction {
                Facing::Left => on_left,
                Facing::Right => on_right,
                Facing::Up => on_top,
                Facing::Down => on_bottom,
            };
            assert!(
                valid_dir,
                "Port {} at ({},{}) faces {:?} but is not on that edge ({}x{})",
                i, x, y, port.direction, self.width, self.height
            );
        }
    }
}

/// Returns the building footprint for the given entity type (in base Right-facing orientation).
/// This function serves as the single dispatch point — update here and in building_art().
pub fn building_footprint(entity_type: EntityType) -> BuildingFootprint {
    use EntityType::*;
    match entity_type {
        // ── Extractors (3×2) ──────────────────────────────────────────────
        OreDeposit | CopperDeposit | CoalDeposit | StoneQuarry
        | UraniumMine | SandExtractor | SulfurMine | BauxiteMine
        | LithiumExtractor | RareEarthExtractor | OilWell | WaterPump
        | GasExtractor | BiomassHarvester | GeothermalTap
            => BuildingFootprint::new_3x2_extractor(),

        // ── 1-input processors (3×3) — smelter-class ─────────────────────
        Smelter | Kiln | Press | WireMill | PlateMachine | RubberVulcanizer
        | PlasticMolder | Electrolyzer | Caster | CokeFurnace | Gasifier
        | Boiler | WaferCutter
            => BuildingFootprint::new_3x3_1in(),

        // ── 2-input processors (3×4) — assembler-class ───────────────────
        Assembler | Mixer | ChemicalPlant | CircuitFabricator
        | MotorAssembly | CrushingMill
            => BuildingFootprint::new_3x4_2in(),

        // ── Tier-2 processors (4×4) ──────────────────────────────────────
        AdvancedAssembler | Refinery | CrackingTower | Cleanroom
        | EnrichmentCascade | CoolantProcessor
            => BuildingFootprint::new_4x4(),

        // ── Tier-3 processors (5×5) ──────────────────────────────────────
        PrecisionAssembler | QuantumLab | RocketAssembly
            => BuildingFootprint::new_5x5(),

        // ── Tier-4 processors (6×6 / 6×7) ────────────────────────────────
        Megassembler => BuildingFootprint::new_6x6(),
        SingularityLab => BuildingFootprint::new_6x7(),

        // ── Belts (stay 1×1) ─────────────────────────────────────────────
        BasicBelt | FastBelt | ExpressBelt
            => BuildingFootprint::new_1x1_conveyor(),

        // ── Splitter / Merger (3×3) ──────────────────────────────────────
        Splitter => BuildingFootprint::new_3x3_splitter(),
        Merger => BuildingFootprint::new_3x3_merger(),

        // ── Underground belts (1×1) ──────────────────────────────────────
        UndergroundEntrance | UndergroundExit
            => BuildingFootprint::new_1x1(),

        // ── Pipes / fluid transport ──────────────────────────────────────
        Pipe | PipeJunction | GasPipeline
            => BuildingFootprint::new_1x1(),
        PumpStation | FluidTank | GasCompressor
            => BuildingFootprint::new_3x2_storage(),

        // ── Rail / transport ─────────────────────────────────────────────
        RailTrack => BuildingFootprint::new_1x1(),
        TrainStation | DronePort
            => BuildingFootprint::new_3x3_storage(),

        // ── Power generators ─────────────────────────────────────────────
        CoalGenerator | GasGenerator | SolarArray | WindTurbine
        | GeothermalPlant
            => BuildingFootprint::new_3x3_power(),
        NuclearReactor | FusionReactor
            => BuildingFootprint::new_5x5_power(),

        // ── Power distribution (1×1) ─────────────────────────────────────
        Transformer | PowerPole | Substation | BatteryBank | Accumulator
            => BuildingFootprint::new_1x1(),

        // ── Storage ──────────────────────────────────────────────────────
        OutputBin | SiloHopper
            => BuildingFootprint::new_3x2_storage(),
        Warehouse | CryoTank | ContainmentVault
            => BuildingFootprint::new_3x3_storage(),

        // ── Defense (1×1) ────────────────────────────────────────────────
        Wall | ReinforcedWall | Turret | ShieldGenerator
            => BuildingFootprint::new_1x1(),

        // ── Environmental (3×3) ──────────────────────────────────────────
        WasteDump | RecyclingPlant | IncinerationPlant | FilterStack
        | ScrubberUnit | ContainmentField
            => BuildingFootprint::new_3x3_1in(),

        // ── Research ─────────────────────────────────────────────────────
        ResearchLab => BuildingFootprint::new_3x3_lab(),
        AdvancedLab => BuildingFootprint::new_4x4_lab(),

        // ── Victory (6×6) ────────────────────────────────────────────────
        SpaceElevatorBase | DysonSwarmLauncher | WarpGateFrame
            => BuildingFootprint::new_6x6_lab(),
    }
}
