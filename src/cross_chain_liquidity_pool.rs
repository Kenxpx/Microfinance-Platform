use scrypto::prelude::*;

#[blueprint]
mod cross_chain_liquidity_pool {
    struct CrossChainLiquidityPool {
        local_pool: Vault,
        bridge_components: HashMap<String, ComponentAddress>,
    }

    impl CrossChainLiquidityPool {
        pub fn new(xrd_token: ResourceAddress) -> ComponentAddress {
            Self {
                local_pool: Vault::new(xrd_token),
                bridge_components: HashMap::new(),
            }
            .instantiate()
            .globalize()
        }

        pub fn add_bridge(&mut self, chain_name: String, bridge_component: ComponentAddress) {
            self.bridge_components.insert(chain_name, bridge_component);
        }

        pub fn request_liquidity(&mut self, amount: Decimal) -> Bucket {
            if self.local_pool.amount() >= amount {
                return self.local_pool.take(amount);
            }

            for (_, bridge) in self.bridge_components.iter() {
                let cross_chain_funds: Bucket = bridge.call("request_liquidity", args![amount]);
                if !cross_chain_funds.is_empty() {
                    self.local_pool.put(cross_chain_funds);
                    return self.local_pool.take(amount);
                }
            }

            panic!("Insufficient liquidity across all chains");
        }
    }
}
