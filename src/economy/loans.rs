use serde::{Deserialize, Serialize};

use crate::economy::ledger::{Difficulty, Economy};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Loan {
    pub principal: i64,
    pub remaining: i64,
    pub interest_rate: f64,
    pub min_payment: i64,
    pub cycles_remaining: u32,
}

impl Loan {
    pub fn interest_due(&self) -> f64 {
        self.remaining as f64 * self.interest_rate
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoanManager {
    pub active_loans: Vec<Loan>,
    pub available_credit: i64,
    pub default_counter: u32,
}

/// Cycles of debt > 5x assets before default triggers.
const DEFAULT_THRESHOLD: u32 = 10;

impl LoanManager {
    pub fn new(difficulty: Difficulty) -> Self {
        let mut mgr = Self {
            active_loans: Vec::new(),
            available_credit: 0,
            default_counter: 0,
        };

        match difficulty {
            Difficulty::Tutorial => {
                mgr.available_credit = i64::MAX / 2;
            }
            Difficulty::Easy => {
                mgr.available_credit = 100_000;
                // Pre-available loan at 2%
            }
            Difficulty::Normal => {
                mgr.available_credit = 50_000;
            }
            Difficulty::Hard => {
                mgr.active_loans.push(Loan {
                    principal: 5_000,
                    remaining: 5_000,
                    interest_rate: 0.08,
                    min_payment: 200,
                    cycles_remaining: 60,
                });
                mgr.available_credit = 25_000;
            }
            Difficulty::Brutal => {
                mgr.active_loans.push(Loan {
                    principal: 10_000,
                    remaining: 10_000,
                    interest_rate: 0.12,
                    min_payment: 500,
                    cycles_remaining: 40,
                });
                mgr.available_credit = 10_000;
            }
        }

        mgr
    }

    pub fn total_debt(&self) -> i64 {
        self.active_loans.iter().map(|l| l.remaining).sum()
    }

    pub fn total_interest_due(&self) -> f64 {
        self.active_loans.iter().map(|l| l.interest_due()).sum()
    }

    /// Max debt is 3x asset value.
    pub fn can_borrow(&self, amount: i64, asset_value: u64) -> bool {
        let max_debt = (asset_value as i64) * 3;
        self.total_debt() + amount <= max_debt && amount <= self.available_credit
    }

    pub fn take_loan(
        &mut self,
        amount: i64,
        rate: f64,
        cycles: u32,
        economy: &mut Economy,
    ) -> bool {
        if !self.can_borrow(amount, economy.asset_value) {
            return false;
        }
        let min_payment = (amount as f64 / cycles as f64).ceil() as i64;
        self.active_loans.push(Loan {
            principal: amount,
            remaining: amount,
            interest_rate: rate,
            min_payment,
            cycles_remaining: cycles,
        });
        self.available_credit -= amount;
        economy.credit(amount);
        true
    }

    pub fn make_payment(&mut self, loan_index: usize, amount: i64, economy: &mut Economy) -> bool {
        if loan_index >= self.active_loans.len() || !economy.can_afford(amount) {
            return false;
        }
        let loan = &mut self.active_loans[loan_index];
        let payment = amount.min(loan.remaining);
        economy.deduct(payment);
        loan.remaining -= payment;
        if loan.remaining <= 0 {
            self.available_credit += loan.principal;
            self.active_loans.remove(loan_index);
        }
        true
    }

    /// Process one economic cycle. Returns (interest_charged, missed_any_payment).
    pub fn update_cycle(&mut self, economy: &mut Economy) -> (f64, bool) {
        let mut total_interest = 0.0;
        let mut missed = false;

        for loan in &mut self.active_loans {
            let interest = loan.interest_due();
            total_interest += interest;
            // Interest accrues to remaining balance
            loan.remaining += interest.ceil() as i64;

            if loan.cycles_remaining > 0 {
                loan.cycles_remaining -= 1;
            }

            // Check if minimum payment can be met
            if economy.cash < loan.min_payment as i64 {
                missed = true;
            }
        }

        if missed {
            economy.credit_rating = (economy.credit_rating - 0.05).max(0.0);
        }

        // Remove fully paid loans
        self.active_loans.retain(|l| l.remaining > 0);

        (total_interest, missed)
    }

    /// Check for default: debt > 5x assets for DEFAULT_THRESHOLD cycles.
    pub fn check_default(&mut self, economy: &Economy) -> bool {
        if economy.difficulty == Difficulty::Tutorial {
            return false;
        }
        let debt = self.total_debt();
        let threshold = (economy.asset_value as i64) * 5;
        if debt > threshold {
            self.default_counter += 1;
        } else {
            self.default_counter = 0;
        }
        self.default_counter >= DEFAULT_THRESHOLD
    }

    /// Interest rate offered based on credit rating and difficulty.
    pub fn offered_rate(&self, economy: &Economy) -> f64 {
        let base = match economy.difficulty {
            Difficulty::Tutorial => 0.0,
            Difficulty::Easy => 0.02,
            Difficulty::Normal => 0.05,
            Difficulty::Hard => 0.10,
            Difficulty::Brutal => 0.15,
        };
        // Worse credit = higher rate
        let credit_penalty = (1.0 - economy.credit_rating) * 0.10;
        base + credit_penalty
    }
}
