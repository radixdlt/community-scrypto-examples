use scrypto::prelude::*;

// Component allowing multiple parties to approve a transaction
// before it gets sent
blueprint! {
    struct MultiSigMaker {
        badge_minter_badge: Vault,
        signer_badge: ResourceAddress,
        tokens: Vault,
        min_required_sig: usize,
        badges_approved: usize,
        destination: ComponentAddress
    }

    impl MultiSigMaker {

        /*
         * nb_badges: The number of people that can approve the transaction
         * min_required_sig: The minimum amount of people that needs to approve before the transaction gets sent
         * destination: The receiver of the transaction
         * amount: Bucket containing the tokens for the transaction
         */
        pub fn new(
            nb_badges: usize, 
            min_required_sig: usize, 
            destination: ComponentAddress, 
            amount: Bucket
        ) -> (ComponentAddress, Bucket) {
            assert!(min_required_sig <= nb_badges, "Min required sig can't be greater than amount of badges");

            // This badge is used to create the initial supply of signer badges
            // and burn them when people approve
            let badge_minter_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);

            // This badge is used to authenticate the users so that only
            // people with this badge can approve the transaction
            let badge_resource_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "MultiSig Signer Badge")
                .mintable(rule!(require(badge_minter_badge.resource_address())), LOCKED)
                .burnable(rule!(require(badge_minter_badge.resource_address())), LOCKED)
                .no_initial_supply();

            // Mint the badges
            let badges = badge_minter_badge.authorize(|| {
                borrow_resource_manager!(badge_resource_address).mint(nb_badges as u64)
            });

            let component = Self {
                tokens: Vault::with_bucket(amount),
                min_required_sig: min_required_sig,
                badge_minter_badge: Vault::with_bucket(badge_minter_badge),
                signer_badge: badge_resource_address,
                badges_approved: 0,
                destination: destination
            }
            .instantiate()
            .globalize();

            (component, badges)
        }

        pub fn approve(&mut self, auth_badge: Bucket) {
            assert!(auth_badge.amount() > Decimal::zero(), "Invalid auth");
            assert!(auth_badge.resource_address() == self.signer_badge, "Invalid badge");
            assert!(self.badges_approved < self.min_required_sig, "Transaction already approved by majority");
            
            // Burn the badge to only allow one call to approve per badge
            self.badge_minter_badge.authorize(|| {
                auth_badge.burn();
            });

            self.badges_approved += 1;
            if self.badges_approved >= self.min_required_sig {
                // Send the transaction
                borrow_component!(self.destination).call::<()>("deposit", vec![scrypto_encode(&self.tokens.take_all())]);
            }
        }
    }
}
