use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[derive(Copy, Clone)]
struct User {
    pub_key: EcdsaPublicKey,
    address: Address
}

struct TestUtils<'a, L: SubstateStore> {
    current_user: Option<User>,
    executor: TransactionExecutor<'a, L>,
    package: Address
}

impl<'a, L: SubstateStore> TestUtils<'a, L> {
    fn new(ledger: &'a mut L) -> Self {
        let mut executor = TransactionExecutor::new(ledger, false);
        let package = executor.publish_package(include_code!("escrow")).unwrap();

        Self {
            current_user: None,
            executor: executor,
            package: package
        }
    }

    fn get_user_or_fail(&self) -> User {
        match &self.current_user {
            Some(user) => {
                return *user;
            },
            None => std::process::abort()
        }
    }

    pub fn get_call_results(&self, receipt: radix_engine::model::Receipt) -> (Vec<Address>, Vec<Address>, Vec<Address>) {
        let packages = receipt.new_entities
            .iter()
            .filter(|a| matches!(a, Address::Package(_)))
            .map(Clone::clone)
            .collect();

        let components = receipt.new_entities
            .iter()
            .filter(|a| matches!(a, Address::Component(_)))
            .map(Clone::clone)
            .collect();

        let resources = receipt.new_entities
            .iter()
            .filter(|a| matches!(a, Address::ResourceDef(_)))
            .map(Clone::clone)
            .collect();

        (resources, components, packages)
    }

    pub fn create_account(&mut self) -> User {
        let account_key = self.executor.new_public_key();
        let account = self.executor.new_account(account_key);

        User {
            pub_key: account_key,
            address: account
        }
    }

    pub fn act_as(&mut self, user: User) -> &mut Self {
        self.current_user = Some(user);
        return self;
    }

    pub fn send_tokens(&mut self, account: Address, amount: i64, token_address: String) {
        let user = self.get_user_or_fail();
        self.executor
                .run(
                    TransactionBuilder::new(&self.executor)
                        .withdraw_from_account(&Resource::from_str(&format!("{},{}", amount, token_address)).unwrap(), user.address)
                        .call_method_with_all_resources(account, "deposit_batch")
                        .build(vec![user.pub_key])
                        .unwrap()
                )
                .unwrap();
    }

    pub fn create_token(&mut self, max_supply: Decimal) -> ResourceDef {
        let user = self.get_user_or_fail();
        let receipt = self.executor
                .run(
                    TransactionBuilder::new(&self.executor)
                        .new_token_fixed(HashMap::new(), max_supply.into())
                        .call_method_with_all_resources(user.address, "deposit_batch")
                        .build(vec![user.pub_key])
                        .unwrap()
                )
                .unwrap();

        return receipt.resource_def(0).unwrap().into();
    }

    pub fn call_method(&mut self, component: &Address, method_name: &str, params: Vec<String>) -> radix_engine::model::Receipt {
        let user = self.get_user_or_fail();

        self.executor
            .run(
                TransactionBuilder::new(&self.executor)
                    .call_method(*component, method_name, params, Some(user.address))
                    .call_method_with_all_resources(user.address, "deposit_batch")
                    .build(vec![user.pub_key])
                    .unwrap()
            ).unwrap()
    }

    pub fn call_function(&mut self, package_name: &str, function_name: &str, params: Vec<String>) -> radix_engine::model::Receipt {
        let user = self.get_user_or_fail();

        self.executor
            .run(
                TransactionBuilder::new(&self.executor)
                    .call_function(
                        self.package,
                        package_name,
                        function_name,
                        params,
                        Some(user.address),
                    )
                    .call_method_with_all_resources(user.address, "deposit_batch")
                    .build(vec![user.pub_key])
                    .unwrap()
            ).unwrap()
    }
}

#[test]
fn test_simple_trade() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut utils = TestUtils::new(&mut ledger);
    let account1 = utils.create_account();
    let account2 = utils.create_account();

    let token1 = utils.act_as(account1).create_token(dec!(8_000));
    let token2 = utils.act_as(account2).create_token(dec!(8_000_000));

    let receipt = utils.act_as(account1)
            .call_function(
                "Escrow", 
                "new", 
                vec![token1.address().to_string(), token2.address().to_string(), account1.address.to_string(), account2.address.to_string()]
            );
    assert!(receipt.result.is_ok());
    let (resources, components, _) = utils.get_call_results(receipt);

    let badge1 = resources.get(0).unwrap();
    let badge2 = resources.get(1).unwrap();
    let component = components.get(0).unwrap();

    // Send badge to account B
    utils.act_as(account1).send_tokens(account2.address, 1, badge2.to_string());

    let receipt = utils.act_as(account1)
            .call_method(component, "put_tokens", vec![format!("500,{}", token1.address()), format!("1,{}", badge1)]);
    assert!(receipt.result.is_ok());

    // Second call should not fail
    let receipt = utils.act_as(account1)
            .call_method(component, "put_tokens", vec![format!("500,{}", token1.address()), format!("1,{}", badge1)]);
    assert!(receipt.result.is_ok());

    // Second user tries with wrong badge, should fail
    let receipt = utils.act_as(account2)
            .call_method(component, "put_tokens", vec![format!("500,{}", token2.address()), format!("1,{}", badge1)]);
    assert!(receipt.result.is_err());

    // Second user tries with good badge
    let receipt = utils.act_as(account2)
            .call_method(component, "put_tokens", vec![format!("500,{}", token2.address()), format!("1,{}", badge2)]);
    assert!(receipt.result.is_ok());

    // Second user tries to put tokens again, should not fail
    let receipt = utils.act_as(account2)
            .call_method(component, "put_tokens", vec![format!("500,{}", token2.address()), format!("1,{}", badge2)]);
    assert!(receipt.result.is_ok());

    // First user accepts the trade
    let receipt = utils.act_as(account1)
            .call_method(component, "accept", vec![format!("1,{}", badge1)]);
    assert!(receipt.result.is_ok());

    // Second user accepts the trade
    let receipt = utils.act_as(account2)
            .call_method(component, "accept", vec![format!("1,{}", badge2)]);
    assert!(receipt.result.is_ok());
}
