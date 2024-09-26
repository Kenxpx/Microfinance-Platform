use scrypto::prelude::*;

#[blueprint]
mod mock_economic_oracle {
    struct MockEconomicOracle;

    impl MockEconomicOracle {
        pub fn new() -> ComponentAddress {
            Self {}
                .instantiate()
                .globalize()
        }

        pub fn get_economic_factor(&self) -> Decimal {
            // Mock implementation
            dec!("1.05") // 5% increase in base rate due to economic factors
        }
    }
}
