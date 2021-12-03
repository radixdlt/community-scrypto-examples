use scrypto::prelude::*;

blueprint! {
    struct VirtualXrd {
        vxrd_resource_def: ResourceDef,
        xrd_vault: Vault,
        minter_badge: Vault
    }

    impl VirtualXrd {
        pub fn new() -> Component {
            let minter_badge = ResourceBuilder::new()
                .metadata("name", "Minter Badge")
                .new_badge_fixed(1);

            let vxrd_resource_def = ResourceBuilder::new()
                .metadata("name", "VXRD")
                .metadata("symbol", "VXRD")
                .new_token_mutable(minter_badge.resource_def());

            Self {
                vxrd_resource_def: vxrd_resource_def,
                xrd_vault: Vault::new(RADIX_TOKEN),
                minter_badge: Vault::with_bucket(minter_badge)
            }
            .instantiate()
        }

        pub fn swap_xrd_for_vxrd(&mut self, xrd: Bucket) -> Bucket {
            scrypto_assert!(xrd.resource_address() == RADIX_TOKEN,
                "The tokens for the opportunity must be XRD");

            let amount = xrd.amount();
            self.xrd_vault.put(xrd);
            let vxrd_tokens = self.minter_badge.authorize(|badge| {
                self.vxrd_resource_def.mint(amount, badge)
            });

            vxrd_tokens
        }

        pub fn swap_vxrd_for_xrd(&mut self, vxrd: Bucket) -> Bucket {
            scrypto_assert!(vxrd.resource_address() == self.vxrd_resource_def.address(),
                "The tokens for the opportunity must be VXRD");

            let amount = vxrd.amount();
            let xrd_tokens = self.xrd_vault.take(amount);
            self.minter_badge.authorize(|badge| {
                vxrd.burn(badge);
            });

            xrd_tokens
        }
    }
}
