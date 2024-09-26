use scrypto::prelude::*;

#[blueprint]
mod reputation_governance {
    struct ReputationGovernance {
        reputation_scores: HashMap<ComponentAddress, u32>,
    }

    impl ReputationGovernance {
        pub fn new() -> ComponentAddress {
            Self {
                reputation_scores: HashMap::new(),
            }
            .instantiate()
            .globalize()
        }

        pub fn update_reputation(&mut self, borrower: ComponentAddress, on_time: bool) {
            let current_score = self.reputation_scores.get(&borrower).unwrap_or(&50);
            let new_score = if on_time {
                (*current_score + 10).min(100)
            } else {
                (*current_score - 10).max(0)
            };
            self.reputation_scores.insert(borrower, new_score);
        }

        pub fn get_reputation(&self, borrower: ComponentAddress) -> u32 {
            *self.reputation_scores.get(&borrower).unwrap_or(&50)
        }
    }
}
