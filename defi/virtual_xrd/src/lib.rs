use scrypto::prelude::*;

blueprint! {
    struct VirtualXrd {
        vxrd_resource_def: ResourceDef,
        xrd_vault: Vault,
        minter_badge: Vault
    }

    impl VirtualXrd {
        pub fn new() -> Component {
            let minter_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Minter Badge")
                .initial_supply_fungible(1);

            let vxrd_resource_def = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "VXRD")
                .metadata("symbol", "VXRD")
                .flags(MINTABLE | BURNABLE)
                .badge(minter_badge.resource_def(), MAY_MINT | MAY_BURN)
                .no_initial_supply();

            Self {
                vxrd_resource_def: vxrd_resource_def,
                xrd_vault: Vault::new(RADIX_TOKEN),
                minter_badge: Vault::with_bucket(minter_badge)
            }
            .instantiate()
        }

        pub fn swap_xrd_for_vxrd(&mut self, xrd: Bucket) -> Bucket {
            assert!(xrd.resource_address() == RADIX_TOKEN,
                "The tokens for the opportunity must be XRD");

            let amount = xrd.amount();
            self.xrd_vault.put(xrd);
            let vxrd_tokens = self.minter_badge.authorize(|badge| {
                self.vxrd_resource_def.mint(amount, badge)
            });

            vxrd_tokens
        }

        pub fn swap_vxrd_for_xrd(&mut self, vxrd: Bucket) -> Bucket {
            assert!(vxrd.resource_address() == self.vxrd_resource_def.address(),
                "The tokens for the opportunity must be VXRD");

            let amount = vxrd.amount();
            let xrd_tokens = self.xrd_vault.take(amount);
            self.minter_badge.authorize(|badge| {
                vxrd.burn_with_auth(badge);
            });

            xrd_tokens
        }
    }
}
