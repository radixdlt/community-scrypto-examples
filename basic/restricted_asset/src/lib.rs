use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct AuthBadgeData {}

//
#[derive(ScryptoSbor, NonFungibleData, Clone)]
pub struct BalanceData {
    #[mutable]
    pub max_balance: Decimal,
    #[mutable]
    pub current_ballence: Decimal,
}

#[blueprint]
mod restricted_asset {
    struct RestrictedAsset {
        component_badge: Vault,
        balance_res_address: ResourceAddress,
        local_id_counter: u64,
    }

    impl RestrictedAsset {
        pub fn instantiate() -> (ComponentAddress, Bucket) {
            let component_badge = Vault::with_bucket(
                ResourceBuilder::new_uuid_non_fungible::<AuthBadgeData>()
                    .mint_initial_supply([(AuthBadgeData {})]),
            );

            let admin_badge = ResourceBuilder::new_uuid_non_fungible::<AuthBadgeData>()
                .mint_initial_supply([(AuthBadgeData {})]);

            let balance_res_address = ResourceBuilder::new_integer_non_fungible::<BalanceData>()
                .mintable(rule!(require(component_badge.resource_address())), LOCKED)
                .burnable(rule!(require(component_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(
                    rule!(require(component_badge.resource_address())),
                    LOCKED,
                )
                .create_with_no_initial_supply();

            let access_rules = AccessRulesConfig::new()
                .method("transfert", AccessRule::AllowAll, LOCKED)
                .method(
                    "deposit",
                    rule!(require(admin_badge.resource_address())),
                    LOCKED,
                )
                .method(
                    "withdraw",
                    rule!(require(admin_badge.resource_address())),
                    LOCKED,
                )
                .method(
                    "set_max_balance",
                    rule!(require(admin_badge.resource_address())),
                    LOCKED,
                )
                .default(rule!(require(component_badge.resource_address())), LOCKED);

            let component_address = Self {
                component_badge,
                balance_res_address,
                local_id_counter: 0,
            }
            .instantiate()
            .globalize_with_access_rules(access_rules);

            (component_address, admin_badge)
        }

        /*

        USER FUNCTIONS
        User facing functions that can be called by anyone to create an account or perform transfers between accounts

        */

        /// Create a new account
        /// This function will mint a new balance non-fungible resource and return it
        pub fn create_account(&mut self) -> Bucket {
            let balance_res_manager = borrow_resource_manager!(self.balance_res_address);

            let loacl_id = NonFungibleLocalId::Integer(self._get_new_local_id().into());

            self.component_badge.authorize(|| {
                let balance_data = BalanceData {
                    max_balance: 5000.into(),
                    current_ballence: 0.into(),
                };

                balance_res_manager.mint_non_fungible(&loacl_id, balance_data)
            })
        }

        /// Transfer an amount of token from one balance to another
        /// The balance proof is used to validate the origin of the transfer
        pub fn transfert(
            &self,
            balance_proof: Proof,
            destination_id: NonFungibleLocalId,
            amount: Decimal,
        ) {
            let validated_balance_proof = balance_proof
                .validate_proof(self.balance_res_address)
                .expect("Wrong badge provided.");

            let origine_id = validated_balance_proof.non_fungible_local_id();

            let mut balance_res_manager = borrow_resource_manager!(self.balance_res_address);

            let origine_balance_data: BalanceData =
                balance_res_manager.get_non_fungible_data(&origine_id);

            let destination_balance_data: BalanceData =
                balance_res_manager.get_non_fungible_data(&destination_id);

            assert!(
                origine_balance_data.current_ballence >= amount,
                "Not enough balance"
            );

            assert!(
                (destination_balance_data.current_ballence + amount)
                    <= destination_balance_data.max_balance,
                "Destination balance is too high"
            );

            self.component_badge.authorize(|| {
                balance_res_manager.update_non_fungible_data(
                    &origine_id,
                    "current_ballence",
                    origine_balance_data.current_ballence - amount,
                );

                balance_res_manager.update_non_fungible_data(
                    &destination_id,
                    "current_ballence",
                    destination_balance_data.current_ballence + amount,
                );
            })
        }

        /*

        ADMIN FUNCTIONS
        These functions are only callable by the admin badge
        They are meant to be used in a controlled environment were we can relate this oparation to real world events from the user

        */

        /// Deposit an amount of token to a balance
        pub fn deposit(&self, balance_id: NonFungibleLocalId, amount: Decimal) {
            let mut balance_res_manager = borrow_resource_manager!(self.balance_res_address);

            let balance_data: BalanceData = balance_res_manager.get_non_fungible_data(&balance_id);

            assert!(
                (balance_data.current_ballence + amount) <= balance_data.max_balance,
                "Destination balance is too high"
            );

            self.component_badge.authorize(|| {
                balance_res_manager.update_non_fungible_data(
                    &balance_id,
                    "current_ballence",
                    balance_data.current_ballence + amount,
                );
            })
        }

        /// Withdraw an amount of token from a balance
        pub fn withdraw(&self, balance_id: NonFungibleLocalId, amount: Decimal) {
            let mut balance_res_manager = borrow_resource_manager!(self.balance_res_address);

            let balance_data: BalanceData = balance_res_manager.get_non_fungible_data(&balance_id);

            assert!(
                balance_data.current_ballence >= amount,
                "Not enough balance"
            );

            self.component_badge.authorize(|| {
                balance_res_manager.update_non_fungible_data(
                    &balance_id,
                    "current_ballence",
                    balance_data.current_ballence - amount,
                );
            })
        }

        /// Set the max balance of a balance
        pub fn set_max_balance(&self, balance_id: NonFungibleLocalId, max_balance: Decimal) {
            let mut balance_res_manager = borrow_resource_manager!(self.balance_res_address);

            let balance_data: BalanceData = balance_res_manager.get_non_fungible_data(&balance_id);

            assert!(
                balance_data.current_ballence <= max_balance,
                "Current balance is higher than the new max balance"
            );

            self.component_badge.authorize(|| {
                balance_res_manager.update_non_fungible_data(
                    &balance_id,
                    "max_balance",
                    max_balance,
                );
            })
        }

        /*

        UTILITIES FUNCTIONS

        */

        /// Get a new LocalId for miniting a new BlanceData
        fn _get_new_local_id(&mut self) -> u64 {
            self.local_id_counter += 1;
            self.local_id_counter
        }
    }
}
