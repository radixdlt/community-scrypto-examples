use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData, Debug, Clone)]
struct FriendData {
    #[mutable]
    savings: Decimal,
}

#[blueprint]
mod save_with_friends {

    struct SaveWithFriends {
        internal_badge: Vault,
        savings: Vault,
        friend_nft_address: ResourceAddress,
        desired_amount: Option<Decimal>,
        goal_achieved: bool,
    }

    impl SaveWithFriends {
        /// Only while instantiating the component we can add
        /// friends to the list of participants.
        pub fn instantiate(
            desired_amount: Decimal,
            accounts: Vec<ComponentAddress>,
        ) -> ComponentAddress {
            // TODO: require all participants' signatures and
            // TODO: split the fees
            // maybe via transaction manifest? e.g.: by returning the bucket of nfts
            // from this method instead of depositing them to addresses

            // a badge that will be stored in the component itself
            let internal_badge = ResourceBuilder::new_fungible()
                .metadata("name", "Save With Friends Component Badge")
                .mint_initial_supply(1);

            // create the NFT resource
            let friend_nft_address = ResourceBuilder::new_uuid_non_fungible::<FriendData>()
                .metadata("name", "Save With Friends NFT")
                .metadata("symbol", "SWF")
                .updateable_non_fungible_data(
                    rule!(require(internal_badge.resource_address())),
                    LOCKED,
                )
                .mintable(rule!(require(internal_badge.resource_address())), LOCKED)
                .burnable(rule!(require(internal_badge.resource_address())), LOCKED)
                .create_with_no_initial_supply();

            // grant an nft to each friend
            internal_badge.authorize(|| {
                let resource_manager = borrow_resource_manager!(friend_nft_address);
                for account in accounts {
                    let bucket =
                        resource_manager.mint_uuid_non_fungible(FriendData { savings: 0.into() });
                    borrow_component!(account).call::<()>("deposit", scrypto_args![bucket]);
                }
            });

            // set a rule that only nft owners can view the savings status
            let access_rules_config = AccessRulesConfig::new()
                .method("show_info", rule!(require(friend_nft_address)), LOCKED)
                .default(rule!(allow_all), LOCKED);

            // init the SaveWithFriends component
            let component_address = Self {
                internal_badge: Vault::with_bucket(internal_badge),
                savings: Vault::new(RADIX_TOKEN),
                friend_nft_address,
                desired_amount: Some(desired_amount),
                goal_achieved: false,
            }
            .instantiate()
            .globalize_with_access_rules(access_rules_config);

            component_address
        }

        pub fn show_info(&self) {
            info!("Current savings: {}", self.savings.amount());
        }

        pub fn deposit(&mut self, xrd: Bucket, nft: Bucket) -> Bucket {
            assert!(
                !self.goal_achieved,
                "Goal has been achieved, no more deposits are allowed!"
            );
            assert!(
                nft.resource_address() == self.friend_nft_address,
                "This is not a valid Save With Friends NFT!"
            );
            let amount = xrd.amount();
            self.savings.put(xrd);
            self.internal_badge.authorize(|| {
                let id = nft.non_fungible_local_id();
                borrow_resource_manager!(self.friend_nft_address)
                    .update_non_fungible_data(&id, "savings", amount);
            });

            // allow withdrawals if the goal has been achieved
            if let Some(desired_amount) = self.desired_amount {
                if self.savings.amount() >= desired_amount {
                    self.goal_achieved = true;
                    info!(
                        "Your goal of saving {desired_amount} has been achieved! Congratulations!"
                    )
                }
            }

            nft
        }

        pub fn withdraw(&mut self, nft: Bucket) -> Bucket {
            assert!(
                self.goal_achieved,
                "Goal has not been achieved, no withdrawals are allowed"
            );
            let id = nft.non_fungible_local_id();
            let friend_savings = borrow_resource_manager!(self.friend_nft_address)
                .get_non_fungible_data::<FriendData>(&id)
                .savings;
            self.internal_badge
                .authorize(|| borrow_resource_manager!(self.friend_nft_address).burn(nft));
            let xrd = self.savings.take(friend_savings);
            // TODO: delete component if no more friends are left
            xrd
        }

        pub fn close_early(&mut self, proof: Proof) {
            assert!(
                !self.goal_achieved,
                "Goal has been achieved, you can already withdraw your savings"
            );

            proof
                .validate_resource_address(self.friend_nft_address)
                .expect("Invalid proof");
            let total_supply = borrow_resource_manager!(self.friend_nft_address).total_supply();
            proof
                .validate_contains_amount(total_supply)
                .expect("All friends must agree to withdraw early");

            self.goal_achieved = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scrypto_unit::*;
    use transaction::{builder::ManifestBuilder, ecdsa_secp256k1::EcdsaSecp256k1PrivateKey};

    struct TestAccount {
        public_key: EcdsaSecp256k1PublicKey,
        _private_key: EcdsaSecp256k1PrivateKey,
        component_address: ComponentAddress,
    }

    impl TestAccount {
        fn new(test_runner: &mut TestRunner) -> Self {
            let (public_key, _private_key, component_address) = test_runner.new_allocated_account();
            Self {
                public_key,
                _private_key,
                component_address,
            }
        }
    }

    struct TestSetup {
        component_address: ComponentAddress,
        nft_address: ResourceAddress,
        accounts: Vec<TestAccount>,
        test_runner: TestRunner,
    }

    impl TestSetup {
        fn new() -> Self {
            let mut test_runner = TestRunner::builder().build();
            let accounts = vec![
                TestAccount::new(&mut test_runner),
                TestAccount::new(&mut test_runner),
                TestAccount::new(&mut test_runner),
            ];
            let package_address = test_runner.compile_and_publish(this_package!());

            let manifest = ManifestBuilder::new()
                .call_function(
                    package_address,
                    "SaveWithFriends",
                    "new_amount_bound",
                    manifest_args!(
                        dec!(1000),
                        accounts
                            .iter()
                            .map(|a| a.component_address)
                            .collect::<Vec<ComponentAddress>>()
                    ),
                )
                .build();
            let receipt = test_runner.execute_manifest_ignoring_fee(
                manifest,
                vec![NonFungibleGlobalId::from_public_key(
                    &accounts[0].public_key,
                )],
            );

            println!("{:?}\n", receipt);
            let component_address = receipt.expect_commit_success().new_component_addresses()[0];
            let nft_address = receipt.expect_commit_success().new_resource_addresses()[0];

            Self {
                component_address,
                accounts,
                test_runner,
                nft_address,
            }
        }
    }

    #[test]
    fn test_close_early() {
        let mut setup = TestSetup::new();
        let mut manifest_builder = ManifestBuilder::new();

        // all friends put their nft in to be able to withdraw early
        for account in &setup.accounts {
            // TODO: find out if there is a simpler way to get ids
            let vaults = setup
                .test_runner
                .get_component_vaults(account.component_address, setup.nft_address);
            let ids = setup
                .test_runner
                // FIXME: vaults[0] is out of bounds
                //  for some reason a test account didn't get a nft
                .inspect_non_fungible_vault(vaults[0])
                .unwrap();

            manifest_builder.withdraw_non_fungibles_from_account(
                account.component_address,
                setup.nft_address,
                &ids,
            );
        }

        let manifest = manifest_builder
            .call_method(setup.component_address, "close_early", manifest_args!())
            // dump all leftovers after the transaction to the first account
            .call_method(
                setup.accounts[0].component_address,
                "deposit_batch",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = setup.test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &setup.accounts[0].public_key,
            )],
        );

        println!("{:?}\n", receipt);
    }
}
