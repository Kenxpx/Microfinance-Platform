use scrypto::prelude::*;

#[blueprint]
mod ai_credit_scorer {
    struct AICreditScorer {
        ai_oracle: ComponentAddress,
    }

    impl AICreditScorer {
        pub fn new(ai_oracle: ComponentAddress) -> ComponentAddress {
            Self {
                ai_oracle,
            }
            .instantiate()
            .globalize()
        }

        pub fn get_credit_score(&self, borrower: ComponentAddress) -> (u32, Decimal, u64) {
            self.ai_oracle.call("get_credit_assessment", args![borrower])
        }
    }
}
