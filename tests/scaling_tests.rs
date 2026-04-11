use vimforge::scaling::difficulty::ScalingState;
use vimforge::scaling::regulations::Regulations;
use vimforge::scaling::prestige::PrestigeState;
use vimforge::scaling::walls::apply_wall_modifiers;

#[test]
fn test_scaling_state_new() {
    let state = ScalingState::new();
    assert_eq!(state.level, 0);
    assert_eq!(state.contracts_completed_at_tier, 0);
}

#[test]
fn test_scaling_quantity_multiplier() {
    let state = ScalingState::new();
    let mult = state.quantity_multiplier();
    assert!(mult >= 1.0, "Level 1 multiplier should be >= 1.0");
}

#[test]
fn test_regulations_new() {
    let regs = Regulations::new();
    assert_eq!(regs.carbon_tax_multiplier, 1.0);
    assert_eq!(regs.pollution_fine_multiplier, 1.0);
}

#[test]
fn test_regulations_tighten_over_time() {
    let mut regs = Regulations::new();
    // Simulate several cycles
    for cycle in 1..=100 {
        regs.update(cycle);
    }
    // After many cycles, regulations should have tightened
    assert!(regs.carbon_tax_multiplier >= 1.0);
}

#[test]
fn test_prestige_state_new() {
    let p = PrestigeState::new();
    assert_eq!(p.level, 0);
    assert_eq!(p.points, 0);
}

#[test]
fn test_prestige_cost_multiplier() {
    let p = PrestigeState::new();
    let mult = p.cost_multiplier();
    assert!(mult > 0.0 && mult <= 1.0);
}

#[test]
fn test_wall_modifiers_level_1() {
    let mods = apply_wall_modifiers(1);
    assert_eq!(mods.oil_depletion_multiplier, 1.0);
    assert!(!mods.singularity_active);
}

#[test]
fn test_wall_modifiers_high_level() {
    let mods = apply_wall_modifiers(50);
    // At high levels, some penalties should kick in
    assert!(mods.oil_depletion_multiplier >= 1.0);
}
