use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct FlashloanTerm {
    pub amount_due: Decimal,
    pub loan_amount: Decimal,
    pub fee_amount: Decimal,
}

#[blueprint]
pub mod pool {

    enable_method_auth! {
        roles {
            admin => updatable_by: [OWNER];
        },
        methods {

            deposit => restrict_to :[admin];
            withdraw => restrict_to :[admin];
            remove_external_liquidity => restrict_to :[admin];
            add_external_liquidity => restrict_to :[admin];

            get_pool_unit_ratio => PUBLIC;
            get_pooled_amount => PUBLIC;
            contribute => PUBLIC;
            redeem => PUBLIC;
            take_flashloan => PUBLIC;
            repay_flashloan => PUBLIC;

        }
    }

    pub struct OneResourcePoolExtended {
        /// Vaul containing the pooled token
        liquidity: Vault,

        /// Variable representing the amount of liquidity being used outside of the pool
        external_liquidity_amount: Decimal,

        /// Flashloan term non fungible  resource manager
        flashloan_term_res_manager: ResourceManager,

        /// Pool unit fungible resource manager
        pool_unit_res_manager: ResourceManager,
    }

    impl OneResourcePoolExtended {
        pub fn instantiate(
            pool_res_address: ResourceAddress,
            admin_rule: AccessRule,
            owner_rule: AccessRule,
        ) -> Global<OneResourcePoolExtended> {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Runtime::blueprint_id());

            let component_rule = rule!(require(global_caller(component_address)));

            let pool_unit_res_manager = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata!(
                    roles {
                        metadata_setter => owner_rule.clone();
                        metadata_setter_updater => rule!(deny_all);
                        metadata_locker => rule!(deny_all);
                        metadata_locker_updater => rule!(deny_all);
                    }
                ))
                .mint_roles(mint_roles! {
                    minter => component_rule.clone();
                    minter_updater => rule!(deny_all);
                })
                .burn_roles(burn_roles! {
                    burner => component_rule.clone();
                    burner_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply();

            let flashloan_term_res_manager =
                ResourceBuilder::new_ruid_non_fungible::<FlashloanTerm>(OwnerRole::None)
                    .metadata(metadata!(
                        roles {
                            metadata_setter => owner_rule.clone();
                            metadata_setter_updater => rule!(deny_all);
                            metadata_locker => rule!(deny_all);
                            metadata_locker_updater => rule!(deny_all);
                        }
                    ))
                    .mint_roles(mint_roles! {
                        minter => component_rule.clone();
                        minter_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => component_rule.clone();
                        burner_updater => rule!(deny_all);
                    })
                    .deposit_roles(deposit_roles! {
                        depositor => rule!(deny_all);
                        depositor_updater => rule!(deny_all);
                    })
                    .create_with_no_initial_supply();

            Self {
                liquidity: Vault::new(pool_res_address),
                flashloan_term_res_manager,
                pool_unit_res_manager,
                external_liquidity_amount: 0.into(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles!(
                admin => admin_rule;
            ))
            .with_address(address_reservation)
            .metadata(metadata!(
                roles {
                    metadata_setter => owner_rule.clone();
                    metadata_setter_updater => rule!(deny_all);
                    metadata_locker => rule!(deny_all);
                    metadata_locker_updater => rule!(deny_all);
                },
                init {
                    "pool_res_address" =>GlobalAddress::from(pool_res_address), locked;
                    "pool_unit_res_address"=> GlobalAddress::from(pool_unit_res_manager.address()), locked;
                }
            ))
            .globalize()
        }

        pub fn get_pool_unit_ratio(&mut self) -> PreciseDecimal {
            let total_liquidity_amount = self.liquidity.amount() + self.external_liquidity_amount;

            let total_supply = self.pool_unit_res_manager.total_supply().unwrap_or(dec!(0));

            if total_liquidity_amount != 0.into() {
                PreciseDecimal::from(total_supply) / PreciseDecimal::from(total_liquidity_amount)
            } else {
                1.into()
            }
        }

        pub fn get_pooled_amount(&mut self) -> (Decimal, Decimal) {
            (self.liquidity.amount(), self.external_liquidity_amount)
        }

        // Handle request to increase asset amount.
        // it add assets to the pool and send back newly minted pool units
        pub fn contribute(&mut self, assets: Bucket) -> Bucket {
            assert!(
                assets.resource_address() == self.liquidity.resource_address(),
                "Pool resource address mismatch"
            );

            let unit_amount = assets.amount() * self.get_pool_unit_ratio();

            self.liquidity.put(assets);

            self.pool_unit_res_manager.mint(unit_amount)
        }

        // Handle request to decrease liquidity.
        // it remove liquidity from the pool and and burn corresponding pool units
        pub fn redeem(&mut self, pool_units: Bucket) -> Bucket {
            assert!(
                pool_units.resource_address() == self.pool_unit_res_manager.address(),
                "Pool unit resource address missmatch"
            );

            let amount =
                (PreciseDecimal::from(pool_units.amount()) / self.get_pool_unit_ratio()).truncate();

            self.pool_unit_res_manager.burn(pool_units);

            assert!(
                amount <= self.liquidity.amount(),
                "Not enough liquidity to withdraw this amount"
            );

            self.liquidity.take(amount)
        }

        pub fn withdraw(&mut self, amount: Decimal, external_liquidity_withdraw: bool) -> Bucket {
            if external_liquidity_withdraw {
                self.external_liquidity_amount += amount;
            }

            self.liquidity.take(amount)
        }

        pub fn deposit(&mut self, assets: Bucket, external_liquidity_deposit: bool) {
            if external_liquidity_deposit {
                self.external_liquidity_amount -= assets.amount();
            }

            self.liquidity.put(assets);
        }

        pub fn add_external_liquidity(&mut self, amount: Decimal) {
            self.external_liquidity_amount += amount;
        }

        pub fn remove_external_liquidity(&mut self, amount: Decimal) {
            self.external_liquidity_amount -= amount;
        }

        /// Flashloan methods

        pub fn take_flashloan(
            &mut self,
            loan_amount: Decimal,
            fee_amount: Decimal,
        ) -> (Bucket, Bucket) {
            assert!(
                loan_amount <= self.liquidity.amount(),
                "Not enough liquidity to supply this loan!"
            );

            // Mint the loan term. it can be deposited in any caccount so, it will need to be return with the repayment and burn for the transaction to be able to suuceed
            let loan_terms =
                self.flashloan_term_res_manager
                    .mint_ruid_non_fungible(FlashloanTerm {
                        amount_due: fee_amount + loan_amount,
                        fee_amount,
                        loan_amount,
                    });
            (self.liquidity.take(loan_amount), loan_terms)
        }

        pub fn repay_flashloan(
            &mut self,
            mut loan_repayment: Bucket,
            loan_terms: Bucket,
        ) -> Bucket {
            let terms: FlashloanTerm = loan_terms.as_non_fungible().non_fungible().data();

            // Verify we are being sent at least the amount due
            assert!(
                loan_repayment.amount() >= terms.amount_due,
                "Insufficient repayment given for your loan!"
            );

            self.liquidity.put(loan_repayment.take(terms.amount_due));

            // We have our payment; we can now burn the transient token
            loan_terms.burn();

            // Return the change to the work top
            loan_repayment
        }
    }
}
