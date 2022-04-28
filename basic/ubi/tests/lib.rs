use scrypto::crypto::{EcdsaPublicKey, EcdsaPrivateKey};
use radix_engine::model::{Receipt};
use radix_engine::transaction::*;
use radix_engine::engine::*;
use radix_engine::ledger::*;
use scrypto::prelude::*;

#[test]
fn register_by_admin() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    let private_key: EcdsaPrivateKey = EcdsaPrivateKey::from_bytes(&test_env.admin_private_key.to_bytes()[0..32]).unwrap();
    let receipt = test_env.register(test_env.admin_account, test_env.admin_account, test_env.admin_public_key, &private_key);
    assert!(receipt.result.is_ok(), "Failed to register an account.");
}

#[test]
fn register_by_person() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    let private_key: EcdsaPrivateKey = EcdsaPrivateKey::from_bytes(&test_env.person_private_key.to_bytes()[0..32]).unwrap();
    let receipt = test_env.register(test_env.person_account, test_env.person_account, test_env.person_public_key, &private_key);
    assert!(!receipt.result.is_ok(), "Only admins can register an account.");
}

#[test]
fn update_expiration() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    let private_key: EcdsaPrivateKey = EcdsaPrivateKey::from_bytes(&test_env.admin_private_key.to_bytes()[0..32]).unwrap();
    let mut receipt = test_env.register(test_env.admin_account, test_env.admin_account, test_env.admin_public_key, &private_key);
    let nft_id = test_env.get_nft_id(receipt);
    
    test_env.advance_epoch(5);
    
    let private_key: EcdsaPrivateKey = EcdsaPrivateKey::from_bytes(&test_env.admin_private_key.to_bytes()[0..32]).unwrap();
    receipt = test_env.update_expiration(nft_id, test_env.admin_account, test_env.admin_public_key, &private_key);
    assert!(receipt.result.is_ok(), "Failed to update registration.");
}

#[test]
fn available_tokens_person() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    test_env.advance_epoch(5);
    let private_key: EcdsaPrivateKey = EcdsaPrivateKey::from_bytes(&test_env.person_private_key.to_bytes()[0..32]).unwrap();
    let receipt = test_env.available_tokens(test_env.person_account, test_env.person_public_key, &private_key);
    assert!(receipt.result.is_ok(), "Failed to read available tokens.");
    assert_eq!("available_tokens: 5", receipt.logs.get(0).unwrap().1);
}

#[test]
fn available_tokens_bot() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    test_env.advance_epoch(5);
    let private_key: EcdsaPrivateKey = EcdsaPrivateKey::from_bytes(&test_env.bot_private_key.to_bytes()[0..32]).unwrap();
    let receipt = test_env.available_tokens(test_env.bot_account, test_env.bot_public_key, &private_key);
    assert!(receipt.result.is_err(), "Bot can check the balance of someone else.");
}

// TODO: Need to update the methods that check the substates before this can run.
// #[test] 
// fn collect_ubi_person() {
//     let mut ledger = InMemorySubstateStore::with_bootstrap();
//     let mut test_env = TestEnv::new(&mut ledger);

//     test_env.advance_epoch(33);
//     let receipt = test_env.collect_ubi(test_env.person_account, test_env.person_public_key, &test_env.person_private_key);
//     assert!(receipt.result.is_ok(), "Failed to collect UBI.");
//     assert_eq!(test_env.get_balance(test_env.person_account, test_env.ubi_token).unwrap(),  Decimal::from_str("33").unwrap());
// }

#[test]
fn collect_ubi_bot() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    test_env.advance_epoch(5);
    let private_key: EcdsaPrivateKey = EcdsaPrivateKey::from_bytes(&test_env.bot_private_key.to_bytes()[0..32]).unwrap();
    let receipt = test_env.collect_ubi(test_env.bot_account, test_env.bot_public_key, &private_key);
    assert!(!receipt.result.is_ok(), "Bot can't collect UBI.");
}

// TODO: Need to update the methods that check the substates before this can run.
// #[test]
// fn send_tokens_to_bot() {
//     let mut ledger = InMemorySubstateStore::with_bootstrap();
//     let mut test_env = TestEnv::new(&mut ledger);

//     test_env.advance_epoch(20);
//     test_env.collect_ubi(test_env.person_account, test_env.person_public_key, &test_env.person_private_key);
//     let receipt = test_env.send_tokens(test_env.person_account, test_env.person_public_key, &test_env.person_private_key, test_env.bot_account, 10);
//     assert!(receipt.result.is_ok(), "Failed to send tokens.");
//     assert_eq!(test_env.get_balance(test_env.person_account, test_env.ubi_token).unwrap(),  Decimal::from_str("10").unwrap());
//     assert_eq!(test_env.get_balance(test_env.bot_account, test_env.ubi_token).unwrap(),  Decimal::from_str("8").unwrap());
// }

struct TestEnv<'a> {
    executor: TransactionExecutor<'a, InMemorySubstateStore>,
    ubi_token: ResourceAddress,
    
    admin_private_key: EcdsaPrivateKey,
    admin_public_key: EcdsaPublicKey,
    admin_account: ComponentAddress,
    
    person_private_key: EcdsaPrivateKey,
    person_public_key: EcdsaPublicKey,
    person_account: ComponentAddress,
    
    bot_private_key: EcdsaPrivateKey,
    bot_public_key: EcdsaPublicKey,
    bot_account: ComponentAddress,
    
    component: ComponentAddress,
    
    admin_badge: ResourceAddress,
    person_badge: ResourceAddress,
    
    person_badge_nft_id: NonFungibleId
}

impl<'a> TestEnv<'a> {
    pub fn new(ledger: &'a mut InMemorySubstateStore) -> Self {
        let mut executor = TransactionExecutor::new(ledger, false);

        let package = executor.publish_package(compile_package!()).unwrap();
        let (admin_pk, admin_sk, admin_account) = executor.new_account();
        let tx = TransactionBuilder::new()
            .call_function(package, "UBI", "new", vec![])
            .call_method_with_all_resources(admin_account, "deposit_batch")
            .build(executor.get_nonce([admin_pk]))
            .sign([&admin_sk]);
        let mut receipt = executor.validate_and_execute(&tx).unwrap();
        println!("{:?}\n", receipt);
        assert!(receipt.result.is_ok(), "Failed to create the UBI component.");

        let admin_badge = receipt.new_resource_addresses[1];
        let person_badge = receipt.new_resource_addresses[2];
        let ubi_token = receipt.new_resource_addresses[3];

        let (person_pk, person_sk, person_account) = executor.new_account();
        let (bot_pk, bot_sk, bot_account) = executor.new_account();

        let mut test_env = Self {
            executor: executor,
            ubi_token: ubi_token,
            
            admin_private_key: EcdsaPrivateKey::from_bytes(&admin_sk.to_bytes()[0..32]).unwrap(),
            admin_public_key: admin_pk,
            admin_account: admin_account,
            
            person_private_key: person_sk,
            person_public_key: person_pk,
            person_account: person_account,
            
            bot_private_key: bot_sk,
            bot_public_key: bot_pk,
            bot_account: bot_account,

            component: receipt.new_component_addresses[0],

            admin_badge: admin_badge,
            person_badge: person_badge,

            person_badge_nft_id: NonFungibleId::from_u32(0)
        };
        receipt = test_env.register(person_account, admin_account, admin_pk, &admin_sk);
        test_env.person_badge_nft_id = test_env.get_nft_id(receipt);
        test_env
    }

    pub fn advance_epoch(&mut self, amount: u64) {
        let current_epoch = self.executor.substate_store().get_epoch();
        self.executor.substate_store_mut().set_epoch(current_epoch + amount);
    }

    pub fn register(&mut self, address: ComponentAddress, admin_address: ComponentAddress, admin_public_key: EcdsaPublicKey, admin_private_key: &EcdsaPrivateKey) -> Receipt {
        let tx = TransactionBuilder::new()
            .create_proof_from_account_by_amount(dec!("1"), self.admin_badge, admin_address)
            .call_method(self.component, "register", args![address])
            .call_method_with_all_resources(admin_address, "deposit_batch")
            .build(self.executor.get_nonce([admin_public_key]))
            .sign([admin_private_key]);
        let receipt = self.executor.validate_and_execute(&tx).unwrap();
        println!("{:?}\n", receipt);
        receipt
    }

    pub fn update_expiration(&mut self, nft_id: NonFungibleId, admin_address: ComponentAddress, admin_public_key: EcdsaPublicKey, admin_private_key: &EcdsaPrivateKey) -> Receipt {
        let tx = TransactionBuilder::new()
            .create_proof_from_account_by_amount(dec!("1"), self.admin_badge, admin_address)
            .create_proof_from_auth_zone(self.admin_badge, |builder, proof_id| {
                builder.call_method(self.component, "update_expiration", args![
                    nft_id,
                    scrypto::resource::Proof(proof_id)
                ])
            })
            .call_method_with_all_resources(admin_address, "deposit_batch")
            .build(self.executor.get_nonce([admin_public_key]))
            .sign([admin_private_key]);
        let receipt = self.executor.validate_and_execute(&tx).unwrap();
        println!("{:?}\n", receipt);
        receipt
    }

    pub fn available_tokens(&mut self, address: ComponentAddress, public_key: EcdsaPublicKey, private_key: &EcdsaPrivateKey) -> Receipt {
        let tx = TransactionBuilder::new()
            .create_proof_from_account_by_amount(dec!("1"), self.person_badge, address)
            .create_proof_from_auth_zone(self.person_badge, |builder, proof_id| {
                builder.call_method(self.component, "available_tokens", args![
                    scrypto::resource::Proof(proof_id)
                ])
            })
            .call_method_with_all_resources(address, "deposit_batch")
            .build(self.executor.get_nonce([public_key]))
            .sign([private_key]);
        let receipt = self.executor.validate_and_execute(&tx).unwrap();
        println!("{:?}\n", receipt);
        receipt
    }

    pub fn collect_ubi(&mut self, address: ComponentAddress, public_key: EcdsaPublicKey, private_key: &EcdsaPrivateKey) -> Receipt {
        let tx = TransactionBuilder::new()
            .create_proof_from_account_by_amount(dec!("1"), self.person_badge, address)
            .create_proof_from_auth_zone(self.person_badge, |builder, proof_id| {
                builder.call_method(self.component, "collect_ubi", args![
                    scrypto::resource::Proof(proof_id)
                ])
            })
            .call_method_with_all_resources(address, "deposit_batch")
            .build(self.executor.get_nonce([public_key]))
            .sign([private_key]);
        let receipt = self.executor.validate_and_execute(&tx).unwrap();
        println!("{:?}\n", receipt);
        receipt
    }

    pub fn send_tokens(&mut self, address: ComponentAddress, public_key: EcdsaPublicKey, private_key: &EcdsaPrivateKey, to: ComponentAddress, amount: u64) -> Receipt {
        let tx = TransactionBuilder::new()
            .withdraw_from_account_by_amount(Decimal::from(amount), self.ubi_token, address)
            .take_from_worktop(self.ubi_token, |builder, bucket_id| {
                builder.call_method(self.component, "send_tokens", args![
                    to,
                    scrypto::resource::Bucket(bucket_id)
                ])
            })
            .call_method_with_all_resources(address, "deposit_batch")
            .build(self.executor.get_nonce([public_key]))
            .sign([private_key]);
        let receipt = self.executor.validate_and_execute(&tx).unwrap();
        println!("{:?}\n", receipt);
        receipt
    }
    
    // fn get_balance(&self, account: ComponentAddress, token: ResourceAddress) -> Result<Decimal, String> {
    //     let ledger = self.executor.substate_store();
    //     let account_component = ledger.get_component(account).unwrap();
        
    //     let state = account_component.state();
    //     let state_validated = validate_data(state).unwrap();
        
    //     let mut queue: Vec<Mid> = state_validated.lazy_maps.clone();
    //     let mut i = 0;
    //     let mut maps_visited: HashSet<Mid> = HashSet::new();
    //     let mut vaults_found: HashSet<Vid> = state_validated.vaults.iter().cloned().collect();
    //     while i < queue.len() {
    //         let mid = queue[i];
    //         i += 1;
    //         if maps_visited.insert(mid) {
    //             let (maps, vaults) = Self::dump_lazy_map(&account, &mid, ledger);
    //             queue.extend(maps);
    //             for v in vaults {
    //                 vaults_found.insert(v);
    //             }
    //         }
    //     }

    //     for vid in vaults_found {
    //         let vault = self.executor.ledger().get_vault(&account, &vid).unwrap();
    //         let amount = vault.amount();
    //         let resource_def_address = vault.resource_address();
    //         if token == resource_def_address {
    //             return Ok(amount);
    //         }
    //     }
    //     return Err(format!(
    //         "No vault found for token {} in account {}",
    //         token, account
    //     ));
    // }

    fn get_nft_id(&self, receipt: Receipt) -> NonFungibleId {
        let result_log = &receipt.logs.get(0).unwrap().1;
        let nft_id_as_string = result_log.split_at("register: created badge for 02b61acea4378e307342b2b684fc35acf0238a4accb9f91e8a4364, nft id: ".len()).1;    
        NonFungibleId::from_str(nft_id_as_string).unwrap()
    }

    // fn dump_lazy_map<T: SubstateStore>(
    //     address: &Address,
    //     mid: &Mid,
    //     ledger: &T,
    // ) -> (Vec<Mid>, Vec<Vid>) {
    //     let mut referenced_maps = Vec::new();
    //     let mut referenced_vaults = Vec::new();
    //     let map = ledger.get_lazy_map(address, mid).unwrap();
    //     for (k, v) in map.map().iter() {
    //         let k_validated = validate_data(k).unwrap();
    //         let v_validated = validate_data(v).unwrap();
    //         referenced_maps.extend(k_validated.lazy_maps);
    //         referenced_maps.extend(v_validated.lazy_maps);
    //         referenced_vaults.extend(k_validated.vaults);
    //         referenced_vaults.extend(v_validated.vaults);
    //     }
    //     (referenced_maps, referenced_vaults)
    // }

}
