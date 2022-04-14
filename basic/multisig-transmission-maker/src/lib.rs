use scrypto::prelude::*;

// Component allowing multiple parties to approve a transaction
// before it gets sent
blueprint! {
    struct MultiSigMaker {
        badge_minter_badge: Vault,
        signer_badge: ResourceDef,
        tokens: Vault,
        min_required_sig: usize,
        badges_approved: usize,
        destination: Address
    }

    impl MultiSigMaker {

        /*
         * nb_badges: The number of people that can approve the transaction
         * min_required_sig: The minimum amount of people that needs to approve before the transaction gets sent
         * destination: The receiver of the transaction
         * amount: Bucket containing the tokens for the transaction
         */
        pub fn new(nb_badges: usize, min_required_sig: usize, destination: Address, amount: Bucket) -> (Component, Bucket) {
            assert!(min_required_sig <= nb_badges, "Min required sig can't be greater than amount of badges");

            // This badge is used to create the initial supply of signer badges
            // and burn them when people approve
            let badge_minter_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .initial_supply_fungible(1);

            // This badge is used to authenticate the users so that only
            // people with this badge can approve the transaction
            let mut badge_resourcedef = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "MultiSig Signer Badge")
                .flags(MINTABLE | BURNABLE)
                .badge(badge_minter_badge.resource_def(), MAY_MINT | MAY_BURN)
                .no_initial_supply();

            // Mint the badges
            let badges = badge_resourcedef.mint(nb_badges as u64, badge_minter_badge.present());

            let component = Self {
                tokens: Vault::with_bucket(amount),
                min_required_sig: min_required_sig,
                badge_minter_badge: Vault::with_bucket(badge_minter_badge),
                signer_badge: badge_resourcedef,
                badges_approved: 0,
                destination: destination
            }
            .instantiate();

            (component, badges)
        }

        pub fn approve(&mut self, auth_badge: Bucket) {
            assert!(auth_badge.amount() > Decimal::zero(), "Invalid auth");
            assert!(auth_badge.resource_def() == self.signer_badge, "Invalid badge");
            assert!(self.badges_approved < self.min_required_sig, "Transaction already approved by majority");
            
            // Burn the badge to only allow one call to approve per badge
            self.badge_minter_badge.authorize(|badge| {
                self.signer_badge.burn_with_auth(auth_badge, badge);
            });

            self.badges_approved += 1;
            if self.badges_approved >= self.min_required_sig {
                // Send the transaction
                Component::from(self.destination).call::<()>("deposit", vec![scrypto_encode(&self.tokens.take_all())]);
            }
        }
    }
}
