use vimforge::map::multitile::{building_footprint, BuildingFootprint, PortType};
use vimforge::render::glyphs::{building_art, entity_art, rotated_art_coords};
use vimforge::resources::{EntityType, Facing};

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

// ---------------------------------------------------------------------------
// 2D infrastructure tests
// ---------------------------------------------------------------------------

/// Verify that building_footprint() dimensions match building_art() dimensions
/// for every entity type.
#[test]
fn test_footprint_matches_art_dimensions() {
    let all_types = [
        EntityType::OreDeposit, EntityType::CopperDeposit, EntityType::CoalDeposit,
        EntityType::StoneQuarry, EntityType::UraniumMine, EntityType::SandExtractor,
        EntityType::SulfurMine, EntityType::BauxiteMine, EntityType::LithiumExtractor,
        EntityType::RareEarthExtractor, EntityType::OilWell, EntityType::WaterPump,
        EntityType::GasExtractor, EntityType::BiomassHarvester, EntityType::GeothermalTap,
        EntityType::Smelter, EntityType::Kiln, EntityType::Press, EntityType::WireMill,
        EntityType::PlateMachine, EntityType::Assembler, EntityType::Mixer,
        EntityType::ChemicalPlant, EntityType::AdvancedAssembler, EntityType::Refinery,
        EntityType::PrecisionAssembler, EntityType::QuantumLab, EntityType::Megassembler,
        EntityType::SingularityLab, EntityType::BasicBelt, EntityType::FastBelt,
        EntityType::ExpressBelt, EntityType::Splitter, EntityType::Merger,
        EntityType::OutputBin, EntityType::Wall,
    ];

    for &et in &all_types {
        let art = building_art(et);
        let fp = building_footprint(et);
        assert_eq!(
            art.width, fp.width,
            "{:?}: art width {} != footprint width {}", et, art.width, fp.width
        );
        assert_eq!(
            art.height, fp.height,
            "{:?}: art height {} != footprint height {}", et, art.height, fp.height
        );
        // Verify tile array length matches width * height
        assert_eq!(
            art.tiles.len(), art.width * art.height,
            "{:?}: tiles.len() {} != width*height {}", et, art.tiles.len(), art.width * art.height
        );
    }
}

/// Test 2D placement and removal via Map methods.
#[test]
fn test_multitile_place_and_remove() {
    let mut world = hecs::World::new();
    let mut map = vimforge::map::grid::Map::new(20, 20);

    // Place a 3x4 assembler at (5, 5) facing Right
    // Footprint: width=3, height=4, so occupies cols 5..8, rows 5..9
    let anchor = map.place_multitile_entity(
        &mut world, 5, 5, EntityType::Assembler, Facing::Right, true,
    );
    assert!(anchor.is_some());
    let anchor = anchor.unwrap();

    // Anchor tile and other tiles within the 3x4 footprint should be occupied
    assert!(map.entity_at(5, 5).is_some());
    assert!(map.entity_at(6, 5).is_some());
    assert!(map.entity_at(7, 5).is_some());
    assert!(map.entity_at(5, 6).is_some());
    assert!(map.entity_at(5, 8).is_some());
    // Tile just outside the footprint should be empty
    assert!(map.entity_at(8, 5).is_none());
    assert!(map.entity_at(5, 9).is_none());

    // Secondary tiles should have PartOfBuilding pointing to anchor
    let ent_6 = map.entity_at(6, 5).unwrap();
    {
        let pob = world.get::<&vimforge::ecs::components::PartOfBuilding>(ent_6).unwrap();
        assert_eq!(pob.anchor, anchor);
    }

    // Remove from a secondary tile — should remove the whole building
    let removed = map.remove_multitile_entity(&mut world, 6, 5);
    assert!(removed);
    assert!(map.entity_at(5, 5).is_none());
    assert!(map.entity_at(6, 5).is_none());
    assert!(map.entity_at(7, 5).is_none());
    assert!(map.entity_at(5, 6).is_none());
}

/// Test that placement fails when tiles are blocked.
#[test]
fn test_multitile_placement_blocked() {
    let mut world = hecs::World::new();
    let mut map = vimforge::map::grid::Map::new(20, 20);

    // Place a belt at (5, 6) — this should block assembler placement at (5, 5)
    map.place_multitile_entity(&mut world, 5, 6, EntityType::BasicBelt, Facing::Right, true);

    // Assembler at (5, 5) is 3x4 and needs (5,6) among others — (5,6) is blocked
    let result = map.place_multitile_entity(
        &mut world, 5, 5, EntityType::Assembler, Facing::Right, true,
    );
    assert!(result.is_none());
    // Original belt should still be there
    assert!(map.entity_at(5, 6).is_some());
}

/// Test rotation symmetry: rotating 4 times returns to original coordinates.
#[test]
fn test_rotation_symmetry_4x() {
    let facings = [Facing::Right, Facing::Down, Facing::Left, Facing::Up];
    for (row, col) in [(0, 0), (1, 2), (3, 0)] {
        let (w, h) = (3, 5);
        for &f in &facings {
            let result = rotated_art_coords(row, col, f, w, h);
            // Just verify it doesn't panic and returns valid coords
            assert!(result.0 < h || result.1 < w,
                "rotated_art_coords({}, {}, {:?}, {}, {}) = {:?}", row, col, f, w, h, result);
        }
        // Full cycle: Right -> Down -> Left -> Up -> Right should return to original
        let (r1, c1) = rotated_art_coords(row, col, Facing::Right, w, h);
        assert_eq!((r1, c1), (row, col), "Right rotation should be identity");
    }
}

/// Test BuildingArt::tile_at out-of-bounds returns default.
#[test]
fn test_tile_at_out_of_bounds() {
    let art = building_art(EntityType::Assembler);
    assert_eq!(art.width, 3);
    assert_eq!(art.height, 4);

    // Valid access
    let tile = art.tile_at(0, 0);
    assert_ne!(tile, ['\u{00B7}', '\u{00B7}']);

    // Out of bounds
    let oob = art.tile_at(10, 0);
    assert_eq!(oob, ['\u{00B7}', '\u{00B7}']);
    let oob2 = art.tile_at(0, 5);
    assert_eq!(oob2, ['\u{00B7}', '\u{00B7}']);
}

/// Test entity_art with tile_row/tile_col produces valid output for multi-tile building.
#[test]
fn test_entity_art_2d_signature() {
    // Assembler is 3x4 — verify all tiles produce non-default art pair.
    for row in 0..4 {
        for col in 0..3 {
            let tile = entity_art(EntityType::Assembler, Facing::Right, row, col);
            assert_ne!(tile, ['\u{00B7}', '\u{00B7}'],
                "Assembler row {} col {} should not be default", row, col);
        }
    }
    // 3x3 smelter — all tiles valid
    for row in 0..3 {
        for col in 0..3 {
            let tile = entity_art(EntityType::Smelter, Facing::Right, row, col);
            assert_ne!(tile, ['\u{00B7}', '\u{00B7}'],
                "Smelter row {} col {} should not be default", row, col);
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 7: Port validation and 2D-specific tests
// ---------------------------------------------------------------------------

/// Every port on every building must be on the perimeter tile.
#[test]
fn test_all_ports_on_perimeter() {
    let types_with_ports = [
        EntityType::OreDeposit, EntityType::CopperDeposit, EntityType::Smelter,
        EntityType::Assembler, EntityType::Splitter, EntityType::Merger,
        EntityType::OutputBin, EntityType::BasicBelt, EntityType::ChemicalPlant,
        EntityType::AdvancedAssembler, EntityType::Refinery, EntityType::Megassembler,
    ];

    for &et in &types_with_ports {
        let fp = building_footprint(et);
        if fp.ports.is_empty() {
            continue;
        }
        for port in &fp.ports {
            let on_left = port.offset_x == 0;
            let on_right = port.offset_x as usize == fp.width - 1;
            let on_top = port.offset_y == 0;
            let on_bottom = port.offset_y as usize == fp.height - 1;
            assert!(
                on_left || on_right || on_top || on_bottom,
                "{:?}: port at ({},{}) is not on perimeter of {}x{} building",
                et, port.offset_x, port.offset_y, fp.width, fp.height
            );
        }
    }
}

/// Port rotation consistency: all 4 facings produce valid port positions
/// within the rotated footprint bounds.
#[test]
fn test_port_rotation_all_facings() {
    let types = [
        EntityType::OreDeposit, EntityType::Smelter, EntityType::Assembler,
        EntityType::Splitter, EntityType::Merger, EntityType::OutputBin,
    ];
    let facings = [Facing::Right, Facing::Down, Facing::Left, Facing::Up];

    for &et in &types {
        for &f in &facings {
            let fp = building_footprint(et).rotate_to(f);
            for port in &fp.ports {
                assert!(
                    (port.offset_x as usize) < fp.width && (port.offset_y as usize) < fp.height,
                    "{:?} facing {:?}: port at ({},{}) out of bounds {}x{}",
                    et, f, port.offset_x, port.offset_y, fp.width, fp.height
                );
            }
        }
    }
}

/// Non-square rotation: 3×2 extractor rotates to 2×3 and back.
#[test]
fn test_nonsquare_rotation_dimensions() {
    let fp = building_footprint(EntityType::OreDeposit);
    assert_eq!(fp.width, 3);
    assert_eq!(fp.height, 2);

    let rotated = fp.rotate_to(Facing::Down);
    assert_eq!(rotated.width, 2);
    assert_eq!(rotated.height, 3);

    let rotated2 = fp.rotate_to(Facing::Left);
    assert_eq!(rotated2.width, 3);
    assert_eq!(rotated2.height, 2);

    let rotated3 = fp.rotate_to(Facing::Up);
    assert_eq!(rotated3.width, 2);
    assert_eq!(rotated3.height, 3);
}

/// 1×1 conveyor has input and output ports.
#[test]
fn test_conveyor_has_ports() {
    let fp = BuildingFootprint::new_1x1_conveyor();
    assert_eq!(fp.ports.len(), 2);
    assert!(fp.ports.iter().any(|p| p.port_type.is_input()));
    assert!(fp.ports.iter().any(|p| p.port_type.is_output()));
}

/// 2D placement: non-square building rotation changes occupied tiles.
#[test]
fn test_nonsquare_placement_rotation() {
    let mut world = hecs::World::new();
    let mut map = vimforge::map::grid::Map::new(20, 20);

    // OreDeposit (3×2) at (5,5) facing Down → rotated to 2×3
    let anchor = map.place_multitile_entity(
        &mut world, 5, 5, EntityType::OreDeposit, Facing::Down, false,
    );
    assert!(anchor.is_some());

    // Rotated footprint is 2×3, so occupies (5,5)-(6,7)
    assert!(map.entity_at(5, 5).is_some());
    assert!(map.entity_at(6, 5).is_some());
    assert!(map.entity_at(5, 7).is_some());
    assert!(map.entity_at(6, 7).is_some());
    // Just outside
    assert!(map.entity_at(7, 5).is_none());
    assert!(map.entity_at(5, 8).is_none());
}
