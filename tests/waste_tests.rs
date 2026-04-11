use vimforge::waste::pollution::{PollutionState, PollutionEffect, update_pollution};
use vimforge::waste::disposal::{WasteBuffer, WasteBinState};
use vimforge::waste::degradation::MachineHealth;

#[test]
fn test_pollution_state_new() {
    let state = PollutionState::new();
    assert_eq!(state.level, 0.0);
    assert_eq!(state.current_effect(), PollutionEffect::None);
}

#[test]
fn test_pollution_add_remove() {
    let mut state = PollutionState::new();
    state.add(100.0);
    assert_eq!(state.level, 100.0);
    state.remove(30.0);
    assert_eq!(state.level, 70.0);
    state.remove(200.0); // should clamp to 0
    assert_eq!(state.level, 0.0);
}

#[test]
fn test_pollution_efficiency_multiplier() {
    let mut state = PollutionState::new();
    assert_eq!(state.efficiency_multiplier(), 1.0);
    state.add(500.0);
    assert!(state.efficiency_multiplier() <= 1.0);
}

#[test]
fn test_update_pollution_with_forests() {
    let mut state = PollutionState::new();
    state.add(50.0);
    let effect = update_pollution(&mut state, 10.0, 5, 0);
    // Forests should absorb some pollution
    assert!(matches!(effect, PollutionEffect::None | PollutionEffect::Fines));
}

#[test]
fn test_waste_buffer_new() {
    let buf = WasteBuffer::new();
    assert!(!buf.is_blocked());
}

#[test]
fn test_waste_buffer_fill_and_block() {
    let mut buf = WasteBuffer::new();
    // Fill to capacity
    for _ in 0..buf.solid_capacity {
        buf.add_solid(1);
    }
    assert!(buf.is_blocked());
}

#[test]
fn test_waste_bin_state() {
    let mut bin = WasteBinState::new(100);
    assert!(!bin.is_full());
    let added = bin.try_add(vimforge::resources::Resource::Slag, 50);
    assert_eq!(added, 50);
    let added = bin.try_add(vimforge::resources::Resource::Slag, 60);
    assert_eq!(added, 50); // only 50 remaining capacity
    assert!(bin.is_full());
}

#[test]
fn test_machine_health_new() {
    let health = MachineHealth::new();
    assert_eq!(health.health, 100.0);
    assert!(!health.broken);
}

#[test]
fn test_machine_health_wear_and_repair() {
    let mut health = MachineHealth::new();
    health.apply_wear(100.0, 5);
    assert!(health.health < 100.0);
    health.repair();
    assert_eq!(health.health, 100.0);
    assert!(!health.broken);
}
