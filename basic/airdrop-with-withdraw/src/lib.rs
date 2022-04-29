use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct AirdropWithWithdrawData {
    amount: Decimal,
    token_type: ResourceAddress,
    #[scrypto(mutable)]
    is_collected: bool,
}

blueprint! {
    struct AirdropWithWithdraw {
        admin_badge: ResourceAddress,
        tokens: Vault,
        recipient_badge_address: ResourceAddress,
        minter_badge_vault: Vault,
    }

    impl AirdropWithWithdraw {
        pub fn new(token_type: ResourceAddress) -> (ComponentAddress, Bucket) {
            let admin_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);

            let minter_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "minter badge")
                .initial_supply(1);

            let recipient_badge_address = ResourceBuilder::new_non_fungible()
                .metadata("name", "recipient badge")
                .mintable(rule!(require(minter_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(minter_badge.resource_address())), LOCKED)
                .no_initial_supply();

            let access_rules = AccessRules::new()
                .method("add_recipient", rule!(require(admin_badge.resource_address())))
                .default(rule!(allow_all));

            let component = Self {
                admin_badge: admin_badge.resource_address(),
                tokens: Vault::new(token_type),
                recipient_badge_address,
                minter_badge_vault: Vault::with_bucket(minter_badge),
            }
            .instantiate()
            .add_access_check(access_rules)
            .globalize();

            return (component, admin_badge);
        }

        pub fn add_recipient(&mut self, recipient: ComponentAddress, tokens: Bucket) {
            assert!(
                tokens.amount() > Decimal::zero(),
                "tokens quantity cannot be 0"
            );
            assert_eq!(
                tokens.resource_address(),
                self.tokens.resource_address(),
                "token address must match"
            );

            let recipient_badge = self.minter_badge_vault.authorize(|| {
                borrow_resource_manager!(self.recipient_badge_address).mint_non_fungible(
                    &NonFungibleId::random(),
                    AirdropWithWithdrawData {
                        amount: tokens.amount(),
                        token_type: tokens.resource_address(),
                        is_collected: false,
                    },
                )
            });
            self.tokens.put(tokens);

            borrow_component!(recipient).call::<()>("deposit", args![recipient_badge]);
        }

        pub fn available_token(&self, auth: Proof) -> Decimal {
            assert_eq!(auth.resource_address(), self.recipient_badge_address, "Invalid Badge Provided");
            assert_eq!(auth.amount(), dec!("1"), "Invalid Badge Provided");

            let nft_data = auth.non_fungible::<AirdropWithWithdrawData>().data();
            let mut result: Decimal = Decimal::zero();
            if !nft_data.is_collected {
                result = nft_data.amount;
            }
            info!("available : {}", result);
            return result;
        }

        pub fn withdraw_token(&mut self, auth: Proof) -> Bucket {
            assert_eq!(auth.resource_address(), self.recipient_badge_address, "Invalid Badge Provided");
            assert_eq!(auth.amount(), dec!("1"), "Invalid Badge Provided");

            let mut nft_data = auth.non_fungible::<AirdropWithWithdrawData>().data();
            assert!(!nft_data.is_collected, "withdraw already done");
            nft_data.is_collected = true;
            let amount = nft_data.amount;
            self.minter_badge_vault.authorize({|| {
                auth.non_fungible().update_data(nft_data);
                }
            });
            info!("withdraw_token : {}", amount);
            return self.tokens.take(amount);
        }
    }
}
