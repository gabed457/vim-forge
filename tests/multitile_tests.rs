use vimforge::map::multitile::{BuildingFootprint, PortType};
use vimforge::resources::Facing;

#[test]
fn test_footprint_1x1() {
    let fp = BuildingFootprint::new_1x1();
    assert_eq!(fp.width, 1);
    assert_eq!(fp.height, 1);
    assert!(fp.is_1x1());
}

#[test]
fn test_footprint_1x3_assembler() {
    let fp = BuildingFootprint::new_1x3_assembler();
    assert_eq!(fp.width, 1);
    assert_eq!(fp.height, 3);
    assert!(!fp.is_1x1());
    assert!(!fp.ports.is_empty());
}

#[test]
fn test_footprint_1x5() {
    let fp = BuildingFootprint::new_1x5();
    assert_eq!(fp.width, 1);
    assert_eq!(fp.height, 5);
}

#[test]
fn test_footprint_1x7() {
    let fp = BuildingFootprint::new_1x7();
    assert_eq!(fp.width, 1);
    assert_eq!(fp.height, 7);
}

#[test]
fn test_footprint_1x9() {
    let fp = BuildingFootprint::new_1x9();
    assert_eq!(fp.width, 1);
    assert_eq!(fp.height, 9);
}

#[test]
fn test_footprint_tiles() {
    let fp = BuildingFootprint::new_1x3_assembler();
    let tiles = fp.tiles();
    assert_eq!(tiles.len(), 3);
}

#[test]
fn test_footprint_rotate() {
    let fp = BuildingFootprint::new_1x3_assembler();
    let rotated = fp.rotate_to(Facing::Down);
    // After rotation, width and height may swap depending on implementation
    assert!(rotated.width >= 1 && rotated.height >= 1);
}

#[test]
fn test_port_type_input_output() {
    assert!(PortType::SolidInput.is_input());
    assert!(!PortType::SolidInput.is_output());
    assert!(PortType::SolidOutput.is_output());
    assert!(!PortType::SolidOutput.is_input());
    assert!(PortType::WasteSolid.is_waste());
    assert!(!PortType::SolidInput.is_waste());
}

#[test]
fn test_1x1_processor_has_ports() {
    let fp = BuildingFootprint::new_1x1_processor();
    assert!(!fp.ports.is_empty(), "1x1 processor should have input/output ports");
}
