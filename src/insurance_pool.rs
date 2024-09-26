use scrypto::prelude::*;

#[blueprint]
mod insurance_pool {
    struct InsurancePool {
        pool: Vault,
        insured_loans: HashMap<ComponentAddress, Decimal>,
        premium_rate: Decimal,
    }

    impl InsurancePool {
        pub fn new(xrd_token: ResourceAddress, premium_rate: Decimal) -> ComponentAddress {
            Self {
                pool: Vault::new(xrd_token),
                insured_loans: HashMap::new(),
                premium_rate,
            }
            .instantiate()
            .globalize()
        }

        pub fn insure_loan(&mut self, borrower: ComponentAddress, loan_amount: Decimal) -> Bucket {
            let premium = loan_amount * self.premium_rate;
            self.insured_loans.insert(borrower, loan_amount);
            self.pool.take(premium)
        }

        pub fn claim_insurance(&mut self, borrower: ComponentAddress, loan_amount: Decimal) -> Bucket {
            let insured_amount = self.insured_loans.remove(&borrower).unwrap();
            assert!(insured_amount >= loan_amount, "Claim exceeds insured amount");
            self.pool.take(loan_amount)
        }
    }
}
