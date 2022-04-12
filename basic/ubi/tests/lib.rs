use radix_engine::model::{Receipt};
use radix_engine::transaction::*;
use radix_engine::engine::*;
use radix_engine::ledger::*;
use scrypto::prelude::*;

#[test]
fn register_by_admin() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    let receipt = test_env.register(test_env.admin_account, test_env.admin_account, test_env.admin_key);
    assert!(receipt.result.is_ok(), "Failed to register an account.");
}

#[test]
fn register_by_person() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    let receipt = test_env.register(test_env.person_account, test_env.person_account, test_env.person_key);
    assert!(!receipt.result.is_ok(), "Only admins can register an account.");
}

#[test]
fn update_expiration() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    let mut receipt = test_env.register(test_env.admin_account, test_env.admin_account, test_env.admin_key);
    let nft_id = test_env.get_nft_id(receipt);
    test_env.advance_epoch(5);
    receipt = test_env.update_expiration(nft_id, test_env.admin_account, test_env.admin_key);
    assert!(receipt.result.is_ok(), "Failed to update registration.");
}

#[test]
fn available_tokens_person() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    test_env.advance_epoch(5);
    let receipt = test_env.available_tokens(test_env.person_account, test_env.person_key, test_env.person_badge_nft_id);
    assert!(receipt.result.is_ok(), "Failed to read available tokens.");
    assert_eq!("available_tokens: 5", receipt.logs.get(0).unwrap().1);
}

#[test]
fn available_tokens_bot() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    test_env.advance_epoch(5);
    let receipt = test_env.available_tokens(test_env.bot_account, test_env.bot_key, test_env.person_badge_nft_id);
    assert!(receipt.result.is_err(), "Bot can check the balance of someone else.");
}

#[test]
fn collect_ubi_person() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    test_env.advance_epoch(33);
    let receipt = test_env.collect_ubi(test_env.person_account, test_env.person_key);
    assert!(receipt.result.is_ok(), "Failed to collect UBI.");
    assert_eq!(test_env.get_balance(test_env.person_account, test_env.ubi_token).unwrap(),  Decimal::from_str("33").unwrap());
}

#[test]
fn collect_ubi_bot() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    test_env.advance_epoch(5);
    let receipt = test_env.collect_ubi(test_env.bot_account, test_env.bot_key);
    assert!(!receipt.result.is_ok(), "Bot can't collect UBI.");
}

#[test]
fn send_tokens_to_bot() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    test_env.advance_epoch(20);
    test_env.collect_ubi(test_env.person_account, test_env.person_key);
    let receipt = test_env.send_tokens(test_env.person_account, test_env.person_key, test_env.bot_account, 10);
    assert!(receipt.result.is_ok(), "Failed to send tokens.");
    assert_eq!(test_env.get_balance(test_env.person_account, test_env.ubi_token).unwrap(),  Decimal::from_str("10").unwrap());
    assert_eq!(test_env.get_balance(test_env.bot_account, test_env.ubi_token).unwrap(),  Decimal::from_str("8").unwrap());
}

struct TestEnv<'a> {
    executor: TransactionExecutor<'a, InMemorySubstateStore>,
    ubi_token: Address,
    admin_key: EcdsaPublicKey,
    admin_account: Address,
    person_key: EcdsaPublicKey,
    person_account: Address,
    bot_key: EcdsaPublicKey,
    bot_account: Address,
    component: Address,
    admin_badge: Address,
    person_badge: Address,
    person_badge_nft_id: u128
}

impl<'a> TestEnv<'a> {
    pub fn new(ledger: &'a mut InMemorySubstateStore) -> Self {
        let mut executor = TransactionExecutor::new(ledger, true);

        let package = executor.publish_package(include_code!("ubi")).unwrap();
        let admin_key = executor.new_public_key();
        let admin_account = executor.new_account(admin_key);

        let tx = TransactionBuilder::new(&executor)
            .call_function(package, "UBI", "new", vec![], Some(admin_account))
            .call_method_with_all_resources(admin_account, "deposit_batch")
            .build(vec![admin_key])
            .unwrap();
        let mut receipt = executor.run(tx).unwrap();
        println!("{:?}\n", receipt);
        assert!(receipt.result.is_ok(), "Failed to create the UBI component.");

        let admin_badge = receipt.resource_def(1).unwrap();
        let person_badge = receipt.resource_def(2).unwrap();
        let ubi_token = receipt.resource_def(3).unwrap();
        let person_key = executor.new_public_key();
        let person_account = executor.new_account(person_key);
        let bot_key = executor.new_public_key();
        let bot_account = executor.new_account(bot_key);

        let mut test_env = Self {
            executor,
            ubi_token,
            admin_key,
            admin_account,
            person_key,
            person_account,
            bot_key,
            bot_account,
            component: receipt.component(0).unwrap(),
            admin_badge,
            person_badge,
            person_badge_nft_id: 0u128
        };
        receipt = test_env.register(person_account, admin_account, admin_key);
        test_env.person_badge_nft_id = test_env.get_nft_id(receipt);
        test_env
    }

    pub fn advance_epoch(&mut self, amount: u64) {
        let ledger = self.executor.ledger_mut();
        ledger.set_epoch(ledger.get_epoch() + amount);
    }

    pub fn register(&mut self, address: Address, admin_address: Address, admin_key: EcdsaPublicKey) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(self.component, "register", vec![address.to_string(), format!("1,{}", self.admin_badge)], Some(admin_address))
            .call_method_with_all_resources(admin_address, "deposit_batch")
            .build(vec![admin_key])
            .unwrap();
        let receipt = self.executor.run(tx).unwrap();
        println!("{:?}\n", receipt);
        receipt
    }

    pub fn update_expiration(&mut self, nft_id: u128, admin_address: Address, admin_key: EcdsaPublicKey) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(self.component, "update_expiration", vec![nft_id.to_string(), format!("1,{}", self.admin_badge)], Some(admin_address))
            .call_method_with_all_resources(admin_address, "deposit_batch")
            .build(vec![admin_key])
            .unwrap();
        let receipt = self.executor.run(tx).unwrap();
        println!("{:?}\n", receipt);
        receipt
    }

    pub fn available_tokens(&mut self, address: Address, key: EcdsaPublicKey, _nft_id: u128) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(self.component, "available_tokens", vec![format!("{},{}", 1, self.person_badge)], Some(address))
            .call_method_with_all_resources(address, "deposit_batch")
            .build(vec![key])
            .unwrap();
        let receipt = self.executor.run(tx).unwrap();
        println!("{:?}\n", receipt);
        receipt
    }

    pub fn collect_ubi(&mut self, address: Address, key: EcdsaPublicKey) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(self.component, "collect_ubi", vec![format!("1,{}", self.person_badge)], Some(address))
            .call_method_with_all_resources(address, "deposit_batch")
            .build(vec![key])
            .unwrap();
        let receipt = self.executor.run(tx).unwrap();
        println!("{:?}\n", receipt);
        receipt
    }

    pub fn send_tokens(&mut self, address: Address, key: EcdsaPublicKey, to: Address, amount: u64) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(self.component, "send_tokens", vec![to.to_string(), format!("{},{}", amount, self.ubi_token)], Some(address))
            .call_method_with_all_resources(address, "deposit_batch")
            .build(vec![key])
            .unwrap();
        let receipt = self.executor.run(tx).unwrap();
        println!("{:?}\n", receipt);
        receipt
    }

    
    fn get_balance(&self, account: Address, token: Address) -> Result<Decimal, String> {
        let ledger = self.executor.ledger();
        let account_component = ledger.get_component(account).unwrap();
        
        let state = account_component.state();
        let state_validated = validate_data(state).unwrap();
        
        let mut queue: Vec<Mid> = state_validated.lazy_maps.clone();
        let mut i = 0;
        let mut maps_visited: HashSet<Mid> = HashSet::new();
        let mut vaults_found: HashSet<Vid> = state_validated.vaults.iter().cloned().collect();
        while i < queue.len() {
            let mid = queue[i];
            i += 1;
            if maps_visited.insert(mid) {
                let (maps, vaults) = Self::dump_lazy_map(&account, &mid, ledger);
                queue.extend(maps);
                for v in vaults {
                    vaults_found.insert(v);
                }
            }
        }

        for vid in vaults_found {
            let vault = self.executor.ledger().get_vault(&account, &vid).unwrap();
            let amount = vault.amount();
            let resource_def_address = vault.resource_address();
            if token == resource_def_address {
                return Ok(amount);
            }
        }
        return Err(format!(
            "No vault found for token {} in account {}",
            token, account
        ));
    }

    fn get_nft_id(&self, receipt: Receipt) -> u128 {
        let result_log = &receipt.logs.get(0).unwrap().1;
        let nft_id_as_string = result_log.split_at("register: created badge for 02b8dd9f4232ce3c00dcb3496956fb57096d5d50763b989ca56f3b, nft id: ".len()).1;
        let nft_id : u128 = nft_id_as_string.parse().unwrap();
        nft_id
    }

    fn dump_lazy_map<T: SubstateStore>(
        address: &Address,
        mid: &Mid,
        ledger: &T,
    ) -> (Vec<Mid>, Vec<Vid>) {
        let mut referenced_maps = Vec::new();
        let mut referenced_vaults = Vec::new();
        let map = ledger.get_lazy_map(address, mid).unwrap();
        for (k, v) in map.map().iter() {
            let k_validated = validate_data(k).unwrap();
            let v_validated = validate_data(v).unwrap();
            referenced_maps.extend(k_validated.lazy_maps);
            referenced_maps.extend(v_validated.lazy_maps);
            referenced_vaults.extend(k_validated.vaults);
            referenced_vaults.extend(v_validated.vaults);
        }
        (referenced_maps, referenced_vaults)
    }

}
