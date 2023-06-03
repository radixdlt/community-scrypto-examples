use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData, Debug)]
struct FriendData {
    #[mutable]
    savings: Decimal,
}

///
#[blueprint]
mod save_with_friends {

    struct SaveWithFriends {
        savings: Vault,
        friends_nft: ResourceAddress,
        desired_amount: Option<Decimal>,
    }

    impl SaveWithFriends {
        pub fn new_amount_bound(
            desired_amount: Decimal,
            accounts: Vec<ComponentAddress>,
        ) -> ComponentAddress {
            // TODO: require all participants' signatures and
            // TODO: split the fees

            // restrict minting with this one-time badge
            let minting_badge = ResourceBuilder::new_fungible()
                .metadata("name", "Save With Friends Minting Badge")
                .burnable(rule!(allow_all), LOCKED)
                .mint_initial_supply(1);

            // create the NFT resource
            let friends_nft = ResourceBuilder::new_uuid_non_fungible::<FriendData>()
                .metadata("name", "Save With Friends NFT")
                .metadata("symbol", "SWF")
                .mintable(rule!(require(minting_badge.resource_address())), LOCKED)
                .create_with_no_initial_supply();

            // grant an nft to each friend
            minting_badge.authorize(|| {
                let resource_manager = borrow_resource_manager!(friends_nft);
                for account in accounts {
                    let bucket =
                        resource_manager.mint_uuid_non_fungible(FriendData { savings: 0.into() });
                    borrow_component!(account).call::<()>("deposit", scrypto_args![bucket]);
                }
            });

            // burn the minting badge to prevent any further minting
            minting_badge.burn();

            // init the SaveWithFriends component
            let component_address = Self {
                savings: Vault::new(RADIX_TOKEN),
                friends_nft,
                desired_amount: Some(desired_amount),
            }
            .instantiate()
            .globalize();

            component_address
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

    #[test]
    fn test_new_amount_bound() {
        // Setup the environment
        let mut test_runner = TestRunner::builder().build();

        // Create an account
        let alice = TestAccount::new(&mut test_runner);
        let bob = TestAccount::new(&mut test_runner);

        // Publish package
        let package_address = test_runner.compile_and_publish(this_package!());

        // Test the `test_new_amount_bound` function.
        let manifest = ManifestBuilder::new()
            .call_function(
                package_address,
                "SaveWithFriends",
                "new_amount_bound",
                manifest_args!(
                    dec!(1000),
                    vec![alice.component_address, bob.component_address]
                ),
            )
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&alice.public_key)],
        );
        println!("{:?}\n", receipt);
        receipt.expect_commit_success();
    }
}
