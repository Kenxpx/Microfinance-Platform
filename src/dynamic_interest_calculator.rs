use scrypto::prelude::*;

#[blueprint]
mod dynamic_interest_calculator {
    struct DynamicInterestCalculator {
        base_rate: Decimal,
        economic_indicator_oracle: ComponentAddress,
    }

    impl DynamicInterestCalculator {
        pub fn new(base_rate: Decimal, oracle: ComponentAddress) -> ComponentAddress {
            Self {
                base_rate,
                economic_indicator_oracle: oracle,
            }
            .instantiate()
            .globalize()
        }

        pub fn calculate_interest_rate(&self, credit_score: u32) -> Decimal {
            let economic_factor: Decimal = self.economic_indicator_oracle.call("get_economic_factor", args![]);
            let credit_factor = Decimal::from(1000 - credit_score) / Decimal::from(1000);
            self.base_rate * economic_factor * credit_factor
        }
    }
}
