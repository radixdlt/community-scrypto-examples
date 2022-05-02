use scrypto::crypto::{EcdsaPrivateKey, EcdsaPublicKey};
use radix_engine::transaction::*;
use radix_engine::ledger::*;
use scrypto::prelude::*;

struct User {
    private_key: EcdsaPrivateKey,
    public_key: EcdsaPublicKey,
    address: ComponentAddress
}

struct TestUtils<'a, L: SubstateStore> {
    current_user: Option<&'a User>,
    executor: TransactionExecutor<'a, L>,
    package: PackageAddress
}

#[allow(mutable_borrow_reservation_conflict)]
impl<'a, L: SubstateStore> TestUtils<'a, L> {
    fn new(ledger: &'a mut L) -> Self {
        let mut executor = TransactionExecutor::new(ledger, false);
        let package = executor.publish_package(compile_package!()).unwrap();

        Self {
            current_user: None,
            executor: executor,
            package: package
        }
    }

    fn get_user_or_fail(&self) -> &User {
        match &self.current_user {
            Some(user) => {
                return &user;
            },
            None => std::process::abort()
        }
    }

    pub fn get_call_results(
        &self, 
        receipt: radix_engine::model::Receipt
    ) -> (Vec<PackageAddress>, Vec<ComponentAddress>, Vec<ResourceAddress>) {
        (
            receipt.new_package_addresses.clone(),
            receipt.new_component_addresses.clone(),
            receipt.new_resource_addresses.clone(),
        )
    }

    pub fn create_account(&mut self) -> User {
        let (pk, sk, account) = self.executor.new_account();

        User {
            private_key: sk,
            public_key: pk,
            address: account
        }
    }

    pub fn act_as(&mut self, user: &'a User) -> &mut Self {
        self.current_user = Some(user);
        return self;
    }

    pub fn send_tokens(&mut self, account: ComponentAddress, amount: Decimal, token_address: ResourceAddress) {
        let user = self.get_user_or_fail();
        self.executor
            .validate_and_execute(
                &TransactionBuilder::new()
                    .withdraw_from_account_by_amount(amount, token_address, user.address)
                    .call_method_with_all_resources(account, "deposit_batch")
                    .build(self.executor.get_nonce([user.public_key]))
                    .sign([&user.private_key])
            )
            .unwrap();
    }

    pub fn create_token(&mut self, max_supply: Decimal) -> ResourceAddress {
        let user = self.get_user_or_fail();
        let receipt = self.executor
            .validate_and_execute(
                &TransactionBuilder::new()
                    .new_token_fixed(HashMap::new(), max_supply.into())
                    .call_method_with_all_resources(user.address, "deposit_batch")
                    .build(self.executor.get_nonce([user.public_key]))
                    .sign([&user.private_key])
            )
            .unwrap();

        return receipt.new_resource_addresses[0];
    }

    pub fn call_function(&mut self, package_name: &str, function_name: &str, params: Vec<Vec<u8>>) -> radix_engine::model::Receipt {
        let user = self.get_user_or_fail();

        self.executor
            .validate_and_execute(
                &TransactionBuilder::new()
                    .call_function(
                        self.package,
                        package_name,
                        function_name,
                        params,
                    )
                    .call_method_with_all_resources(user.address, "deposit_batch")
                    .build(self.executor.get_nonce([user.public_key]))
                    .sign([&user.private_key])
            ).unwrap()
    }
}

#[test]
fn test_simple_trade() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut utils = TestUtils::new(&mut ledger);
    let account1 = utils.create_account();
    let account2 = utils.create_account();

    let token1 = utils.act_as(&account1).create_token(dec!(8_000));
    let token2 = utils.act_as(&account2).create_token(dec!(8_000_000));

    let receipt = utils
        .act_as(&account1)
        .call_function(
            "Escrow", 
            "new", 
            args![token1, token2]
        );
    assert!(receipt.result.is_ok());
    let (_packages, components, resources) = utils.get_call_results(receipt);

    let badge1 = *resources.get(0).unwrap();
    let badge2 = *resources.get(1).unwrap();
    let component = components.get(0).unwrap();

    // Send badge to account B
    utils.act_as(&account1).send_tokens(account2.address, Decimal::one(), badge2);
    let receipt = utils.executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .withdraw_from_account_by_amount(dec!("500"), token1, account1.address)
                .create_proof_from_account_by_amount(dec!("1"), badge1, account1.address)
                .take_from_worktop(token1, |builder, token_bucket_id| {
                    builder.pop_from_auth_zone(|builder, badge_proof_id| {
                        builder.call_method(
                            *component, 
                            "put_tokens", 
                            args![
                                scrypto::resource::Bucket(token_bucket_id),
                                scrypto::resource::Proof(badge_proof_id)
                            ]
                        )
                    })
                })
                .call_method_with_all_resources(account1.address, "deposit_batch")
                .build(utils.executor.get_nonce([account1.public_key]))
                .sign([&account1.private_key])
        ).unwrap();
    assert!(receipt.result.is_ok());

    // Second call should not fail
    let receipt = utils.executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .withdraw_from_account_by_amount(dec!("500"), token1, account1.address)
                .create_proof_from_account_by_amount(dec!("1"), badge1, account1.address)
                .take_from_worktop(token1, |builder, token_bucket_id| {
                    builder.pop_from_auth_zone(|builder, badge_proof_id| {
                        builder.call_method(
                            *component, 
                            "put_tokens", 
                            args![
                                scrypto::resource::Bucket(token_bucket_id),
                                scrypto::resource::Proof(badge_proof_id)
                            ]
                        )
                    })
                })
                .call_method_with_all_resources(account1.address, "deposit_batch")
                .build(utils.executor.get_nonce([account1.public_key]))
                .sign([&account1.private_key])
        ).unwrap();
    assert!(receipt.result.is_ok());

    // Second user tries with wrong badge, should fail
    let receipt = utils.executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .withdraw_from_account_by_amount(dec!("500"), token2, account2.address)
                .create_proof_from_account_by_amount(dec!("1"), badge1, account2.address)
                .take_from_worktop(token1, |builder, token_bucket_id| {
                    builder.pop_from_auth_zone(|builder, badge_proof_id| {
                        builder.call_method(
                            *component, 
                            "put_tokens", 
                            args![
                                scrypto::resource::Bucket(token_bucket_id),
                                scrypto::resource::Proof(badge_proof_id)
                            ]
                        )
                    })
                })
                .call_method_with_all_resources(account2.address, "deposit_batch")
                .build(utils.executor.get_nonce([account2.public_key]))
                .sign([&account2.private_key])
        ).unwrap();
    assert!(receipt.result.is_err());

    // Second user tries with good badge
    let receipt = utils.executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .withdraw_from_account_by_amount(dec!("500"), token2, account2.address)
                .create_proof_from_account_by_amount(dec!("1"), badge2, account2.address)
                .take_from_worktop(token2, |builder, token_bucket_id| {
                    builder.pop_from_auth_zone(|builder, badge_proof_id| {
                        builder.call_method(
                            *component, 
                            "put_tokens", 
                            args![
                                scrypto::resource::Bucket(token_bucket_id),
                                scrypto::resource::Proof(badge_proof_id)
                            ]
                        )
                    })
                })
                .call_method_with_all_resources(account2.address, "deposit_batch")
                .build(utils.executor.get_nonce([account2.public_key]))
                .sign([&account2.private_key])
        ).unwrap();
    assert!(receipt.result.is_ok());

    // Second user tries to put tokens again, should not fail
    let receipt = utils.executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .withdraw_from_account_by_amount(dec!("500"), token2, account2.address)
                .create_proof_from_account_by_amount(dec!("1"), badge2, account2.address)
                .take_from_worktop(token2, |builder, token_bucket_id| {
                    builder.pop_from_auth_zone(|builder, badge_proof_id| {
                        builder.call_method(
                            *component, 
                            "put_tokens", 
                            args![
                                scrypto::resource::Bucket(token_bucket_id),
                                scrypto::resource::Proof(badge_proof_id)
                            ]
                        )
                    })
                })
                .call_method_with_all_resources(account2.address, "deposit_batch")
                .build(utils.executor.get_nonce([account2.public_key]))
                .sign([&account2.private_key])
        ).unwrap();
    assert!(receipt.result.is_ok());

    // First user accepts the trade
    let receipt = utils.executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .create_proof_from_account_by_amount(dec!("1"), badge1, account1.address)
                .pop_from_auth_zone(|builder, badge_proof_id| {
                    builder.call_method(
                        *component, 
                        "accept", 
                        args![scrypto::resource::Proof(badge_proof_id)]
                    )
                })
                .call_method_with_all_resources(account1.address, "deposit_batch")
                .build(utils.executor.get_nonce([account1.public_key]))
                .sign([&account1.private_key])
        ).unwrap();
    assert!(receipt.result.is_ok());

    // Second user accepts the trade
    let receipt = utils.executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .create_proof_from_account_by_amount(dec!("1"), badge2, account2.address)
                .pop_from_auth_zone(|builder, badge_proof_id| {
                    builder.call_method(
                        *component, 
                        "accept", 
                        args![scrypto::resource::Proof(badge_proof_id)]
                    )
                })
                .call_method_with_all_resources(account2.address, "deposit_batch")
                .build(utils.executor.get_nonce([account2.public_key]))
                .sign([&account2.private_key])
        ).unwrap();
    assert!(receipt.result.is_ok());
}
