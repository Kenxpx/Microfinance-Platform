use scrypto::prelude::*;

#[blueprint]
mod mock_ai_oracle {
    struct MockAIOracle;

    impl MockAIOracle {
        pub fn new() -> ComponentAddress {
            Self {}
                .instantiate()
                .globalize()
        }

        pub fn get_credit_assessment(&self, _borrower: ComponentAddress) -> (u32, Decimal, u64) {
            // Mock implementation
            (750, dec!("1000"), 2592000) // 750 credit score, 1000 XRD recommended loan, 30 days term
        }
    }
}
