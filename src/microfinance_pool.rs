use scrypto::prelude::*;

#[blueprint]
mod microfinance_pool {
    struct MicrofinancePool {
        lending_pools: HashMap<ResourceAddress, Vault>,
        xrd_token: ResourceAddress,
        ai_credit_scorer: ComponentAddress,
        interest_calculator: ComponentAddress,
        liquidity_pool: ComponentAddress,
        insurance_pool: ComponentAddress,
        governance: ComponentAddress,
        impact_tracker: ComponentAddress,
        loans: HashMap<ComponentAddress, (ResourceAddress, Decimal, Decimal, u64)>, // (currency, amount, interest_rate, due_epoch)
        admin_badge: ResourceAddress,
    }

    impl MicrofinancePool {
        pub fn new(
            xrd_token: ResourceAddress,
            ai_credit_scorer: ComponentAddress,
            interest_calculator: ComponentAddress,
            liquidity_pool: ComponentAddress,
            insurance_pool: ComponentAddress,
            governance: ComponentAddress,
            impact_tracker: ComponentAddress
        ) -> (ComponentAddress, Bucket) {
            let admin_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Microfinance Admin Badge")
                .initial_supply(1);

            let mut lending_pools = HashMap::new();
            lending_pools.insert(xrd_token, Vault::new(xrd_token));

            let component = Self {
                lending_pools,
                xrd_token,
                ai_credit_scorer,
                interest_calculator,
                liquidity_pool,
                insurance_pool,
                governance,
                impact_tracker,
                loans: HashMap::new(),
                admin_badge: admin_badge.resource_address(),
            }
            .instantiate();

            (component.globalize(), admin_badge)
        }

        pub fn add_currency(&mut self, currency: ResourceAddress) {
            self.lending_pools.insert(currency, Vault::new(currency));
        }

        pub fn deposit(&mut self, amount: Decimal, currency: ResourceAddress) -> Bucket {
            let pool = self.lending_pools.get_mut(&currency).unwrap();
            pool.put(amount)
        }

        pub fn request_loan(&mut self, borrower: ComponentAddress, amount: Decimal, currency: ResourceAddress) -> (Bucket, Bucket) {
            let (credit_score, _, _) = self.ai_credit_scorer.call("get_credit_score", args![borrower]);
            let interest_rate: Decimal = self.interest_calculator.call("calculate_interest_rate", args![credit_score]);
            
            let pool = self.lending_pools.get_mut(&currency).unwrap();
            let loan = pool.take(amount);
            
            let due_epoch = Runtime::current_epoch() + 2592000; // 30 days in seconds
            self.loans.insert(borrower, (currency, amount, interest_rate, due_epoch));
            
            let insurance_premium: Bucket = self.insurance_pool.call("insure_loan", args![borrower, amount]);
            
            self.impact_tracker.call("record_loan", args![borrower, amount]);
            
            (loan, insurance_premium)
        }

        pub fn repay_loan(&mut self, borrower: ComponentAddress, payment: Bucket) {
            let (currency, loan_amount, interest_rate, due_epoch) = self.loans.remove(&borrower).unwrap();
            let repayment_amount = loan_amount * (Decimal::ONE + interest_rate);
            
            assert!(payment.amount() >= repayment_amount, "Insufficient repayment");
            
            let pool = self.lending_pools.get_mut(&currency).unwrap();
            pool.put(payment.take(repayment_amount));
            
            if Runtime::current_epoch() <= due_epoch {
                self.governance.call("update_reputation", args![borrower, true]);
            } else {
                self.governance.call("update_reputation", args![borrower, false]);
            }
            
            self.impact_tracker.call("record_repayment", args![borrower, repayment_amount]);
        }

        pub fn liquidate_overdue_loans(&mut self, auth: Proof) {
            auth.validate_proof(self.admin_badge).unwrap();

            let current_epoch = Runtime::current_epoch();
            let overdue_loans: Vec<_> = self.loans.iter()
                .filter(|(_, (_, _, _, due_epoch))| *due_epoch < current_epoch)
                .map(|(borrower, _)| *borrower)
                .collect();

            for borrower in overdue_loans {
                let (currency, loan_amount, _, _) = self.loans.remove(&borrower).unwrap();
                let insurance_payout: Bucket = self.insurance_pool.call("claim_insurance", args![borrower, loan_amount]);
                let pool = self.lending_pools.get_mut(&currency).unwrap();
                pool.put(insurance_payout);
                self.governance.call("update_reputation", args![borrower, false]);
            }
        }
    }
}
