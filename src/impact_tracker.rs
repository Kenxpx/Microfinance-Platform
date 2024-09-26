use scrypto::prelude::*;

#[blueprint]
mod impact_tracker {
    struct ImpactTracker {
        total_loans: Decimal,
        total_repayments: Decimal,
    }

    impl ImpactTracker {
        pub fn new() -> ComponentAddress {
            Self {
                total_loans: Decimal::zero(),
                total_repayments: Decimal::zero(),
            }
            .instantiate()
            .globalize()
        }

        pub fn record_loan(&mut self, _borrower: ComponentAddress, amount: Decimal) {
            self.total_loans += amount;
        }

        pub fn record_repayment(&mut self, _borrower: ComponentAddress, amount: Decimal) {
            self.total_repayments += amount;
        }

        pub fn get_impact_metrics(&self) -> (Decimal, Decimal, Decimal) {
            let repayment_rate = if self.total_loans > Decimal::zero() {
                self.total_repayments / self.total_loans
            } else {
                Decimal::zero()
            };
            (self.total_loans, self.total_repayments, repayment_rate)
        }
    }
}
