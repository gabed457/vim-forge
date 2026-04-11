use vimforge::power::generators::{generator_spec, GeneratorState};
use vimforge::power::batteries::{battery_spec, BatteryState};
use vimforge::power::grid::PowerGrid;
use vimforge::resources::EntityType;

#[test]
fn test_power_grid_new() {
    let grid = PowerGrid::new(1);
    assert_eq!(grid.id, 1);
    assert_eq!(grid.available_mw, 0.0);
    assert_eq!(grid.consumed_mw, 0.0);
}

#[test]
fn test_generator_spec_coal() {
    let spec = generator_spec(EntityType::CoalGenerator);
    assert!(spec.is_some());
    let spec = spec.unwrap();
    assert!(spec.base_mw > 0.0);
}

#[test]
fn test_generator_spec_solar() {
    let spec = generator_spec(EntityType::SolarArray);
    assert!(spec.is_some());
    let spec = spec.unwrap();
    assert!(spec.variable); // solar is variable output
}

#[test]
fn test_generator_state_new() {
    let state = GeneratorState::new(EntityType::CoalGenerator, (5, 5));
    assert_eq!(state.pos, (5, 5));
    assert!(!state.active);
    assert_eq!(state.current_mw, 0.0);
}

#[test]
fn test_battery_spec_basic() {
    let spec = battery_spec(EntityType::BatteryBank);
    assert!(spec.is_some());
    let spec = spec.unwrap();
    assert!(spec.capacity_mwh > 0.0);
    assert!(spec.charge_rate > 0.0);
}

#[test]
fn test_battery_state_charge_discharge() {
    let mut state = BatteryState::new(EntityType::BatteryBank, (3, 3));
    assert_eq!(state.charge_fraction(), 0.0);

    let charged = state.charge(5.0);
    assert!(charged > 0.0);
    assert!(state.current_charge > 0.0);

    let discharged = state.discharge(2.0);
    assert!(discharged > 0.0);
}

#[test]
fn test_nonexistent_generator() {
    let spec = generator_spec(EntityType::BasicBelt);
    assert!(spec.is_none());
}
