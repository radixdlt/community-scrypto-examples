use scrypto::prelude::*;

blueprint! {
    struct VirtualXrd {
        vxrd_resource_def: ResourceAddress,
        xrd_vault: Vault,
        minter_badge: Vault
    }

    impl VirtualXrd {
        pub fn new() -> ComponentAddress {
            let minter_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Minter Badge")
                .initial_supply(1);

            let vxrd_resource_def = ResourceBuilder::new_fungible()
                .metadata("name", "VXRD")
                .metadata("symbol", "VXRD")
                .mintable(rule!(require(minter_badge.resource_address())), LOCKED)
                .burnable(rule!(require(minter_badge.resource_address())), LOCKED)
                .no_initial_supply();

            Self {
                vxrd_resource_def: vxrd_resource_def,
                xrd_vault: Vault::new(RADIX_TOKEN),
                minter_badge: Vault::with_bucket(minter_badge)
            }
            .instantiate().globalize()
        }

        pub fn swap_xrd_for_vxrd(&mut self, xrd: Bucket) -> Bucket {
            assert!(xrd.resource_address() == RADIX_TOKEN,
                "The tokens for the opportunity must be XRD");

            let amount = xrd.amount();
            self.xrd_vault.put(xrd);
            let vxrd_tokens = self.minter_badge.authorize(|| {
                borrow_resource_manager!(self.vxrd_resource_def).mint(amount)
            });

            vxrd_tokens
        }

        pub fn swap_vxrd_for_xrd(&mut self, vxrd: Bucket) -> Bucket {
            assert!(vxrd.resource_address() == self.vxrd_resource_def,
                "The tokens for the opportunity must be VXRD");

            let amount = vxrd.amount();
            let xrd_tokens = self.xrd_vault.take(amount);
            self.minter_badge.authorize(|| {
                vxrd.burn();
            });

            xrd_tokens
        }
    }
}
