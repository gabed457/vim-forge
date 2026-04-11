use vimforge::economy::ledger::{Difficulty, Economy};
use vimforge::economy::loans::LoanManager;

#[test]
fn test_economy_new_normal() {
    let eco = Economy::new(Difficulty::Normal);
    assert_eq!(eco.cash, 25_000);
    assert_eq!(eco.total_earned, 0);
    assert_eq!(eco.total_spent, 0);
    assert_eq!(eco.bankruptcy_counter, 0);
}

#[test]
fn test_economy_new_easy() {
    let eco = Economy::new(Difficulty::Easy);
    assert!(eco.cash >= 10_000, "Easy mode should have at least as much starting cash");
}

#[test]
fn test_economy_new_hard() {
    let eco = Economy::new(Difficulty::Hard);
    assert!(eco.cash <= 10_000, "Hard mode should have less starting cash");
}

#[test]
fn test_economy_new_tutorial() {
    let eco = Economy::new(Difficulty::Tutorial);
    assert!(eco.cash > 0, "Tutorial should have positive starting cash");
}

#[test]
fn test_can_afford_and_deduct() {
    let mut eco = Economy::new(Difficulty::Normal);
    let starting = eco.cash;
    assert!(eco.can_afford(1000));
    assert!(!eco.can_afford(starting + 1));
    eco.deduct(500);
    assert_eq!(eco.cash, starting - 500);
    assert_eq!(eco.total_spent, 500);
}

#[test]
fn test_credit() {
    let mut eco = Economy::new(Difficulty::Normal);
    let starting = eco.cash;
    eco.credit(2000);
    assert_eq!(eco.cash, starting + 2000);
    assert_eq!(eco.total_earned, 2000);
}

#[test]
fn test_net_worth() {
    let mut eco = Economy::new(Difficulty::Normal);
    eco.add_asset(5000);
    assert_eq!(eco.net_worth(), eco.cash + eco.asset_value as i64);
}

#[test]
fn test_bankruptcy_counter() {
    let mut eco = Economy::new(Difficulty::Normal);
    eco.cash = -1;
    let bankrupt = eco.update_bankruptcy();
    assert!(eco.bankruptcy_counter > 0 || bankrupt);
}

#[test]
fn test_loan_manager_new() {
    let lm = LoanManager::new(Difficulty::Normal);
    assert_eq!(lm.total_debt(), 0);
    assert!(lm.available_credit > 0);
}

#[test]
fn test_loan_take_and_debt() {
    let mut lm = LoanManager::new(Difficulty::Normal);
    let mut eco = Economy::new(Difficulty::Normal);
    // Need asset_value > 0 for can_borrow to pass
    eco.add_asset(10_000);
    let initial_cash = eco.cash;
    let took = lm.take_loan(1000, 0.05, 10, &mut eco);
    assert!(took);
    assert_eq!(lm.total_debt(), 1000);
    assert_eq!(eco.cash, initial_cash + 1000);
}

#[test]
fn test_difficulty_names() {
    assert_eq!(Difficulty::Tutorial.name(), "Tutorial");
    assert_eq!(Difficulty::Easy.name(), "Easy");
    assert_eq!(Difficulty::Normal.name(), "Normal");
    assert_eq!(Difficulty::Hard.name(), "Hard");
    assert_eq!(Difficulty::Brutal.name(), "Brutal");
}
