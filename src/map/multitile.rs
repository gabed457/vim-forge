use serde::{Deserialize, Serialize};

use crate::resources::Facing;

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
}
