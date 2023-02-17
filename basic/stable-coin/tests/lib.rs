use radix_engine::ledger::*;
use scrypto::prelude::*;
use scrypto_unit::*;

mod helper;
use helper::*;

#[test]
fn test_stablecoin() {
    // Setup the environment
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    let (user1, user2, user3) = make_users(&mut test_runner);

    let total_supply_original = dec!(1000000);
    let keys_str = vec!["name", "symbol", "icon_url", "url", "author", "stage"];
    let keys_owned = convert_str_slices(keys_str);
    let blueprint_name = "StableCoinVault";
    let mut total_supply = total_supply_original;
    let token_name = "USD Tether";
    let token_symbol = "USDT";
    let icon_url = "https://token_website.com/icon.ico";
    let url = "https://token_website.com";
    let author = "Xman";
    let stage = "stage 1 - Fixed supply, withdraw may be restricted";

    let values_str = vec![token_name, token_symbol, icon_url, url, author, stage];
    let values_owned = convert_str_slices(values_str);
    println!("test4");

    let (resources, package_addr, component) = deploy_blueprint(
        &mut test_runner,
        blueprint_name,
        "new",
        total_supply_original,
        keys_owned.clone(),
        values_owned.clone(),
        &user1,
    );
    println!("new resource addr: {:?}", resources);
    assert_eq!(resources.len(), 4);

    let admin_badge_addr = resources[0];
    let token_addr = resources[3];
    let admin_badge_balance = user_balance(&mut test_runner, &user1, admin_badge_addr);
    let wd_badge_balance = user_balance(&mut test_runner, &user1, resources[1]);
    let auth_badge_balance = user_balance(&mut test_runner, &user1, resources[2]);
    let u1token_bal = user_balance(&mut test_runner, &user1, token_addr);
    println!(
        "admin_badge_balance:{}, wd_badge_balance:{}, auth_badge_balance:{}, u1token_bal:{}",
        admin_badge_balance, wd_badge_balance, auth_badge_balance, u1token_bal
    );
    assert_eq!(admin_badge_balance, dec!(3));
    assert_eq!(wd_badge_balance, dec!(2));
    assert_eq!(auth_badge_balance, dec!(0));
    assert_eq!(u1token_bal, dec!(0));

    //----------------== get_data
    let data = get_data(&mut test_runner, &user1, component);
    assert_eq!(data.2, total_supply);

    //----------------== mint_to_bucket
    let amount = dec!(1000);
    let badge_amount = dec!(3);
    invoke_badge_access_decimal(
        &mut test_runner,
        &user1,
        component,
        "mint_to_bucket",
        amount,
        admin_badge_addr,
        badge_amount,
    );
    let data = get_data(&mut test_runner, &user1, component);
    assert_eq!(data.2, total_supply + amount);
    total_supply += amount;
    assert_eq!(data.1, total_supply_original);
    let mut _vault_amount = data.1;
    println!("total supply increased accordingly");

    let u1token_bal = user_balance(&mut test_runner, &user1, token_addr);
    println!("u1token_bal:{}", u1token_bal);
    assert_eq!(u1token_bal, amount);
    println!("user token balance increased accordingly");
    let mut _u1token_bal_t = u1token_bal;

    //----------------== acct2
    let mut _u2token_bal_t = dec!(0);
    let u2token_bal = user_balance(&mut test_runner, &user2, token_addr);
    assert_eq!(u2token_bal, _u2token_bal_t);

    //----------------== Transfering acct1 to acct2
    let amount = dec!(123);
    send_tokens(
        &mut test_runner,
        &user1,
        &user2,
        component,
        amount,
        token_addr,
    );
    let u1token_bal = user_balance(&mut test_runner, &user1, token_addr);
    assert_eq!(u1token_bal, _u1token_bal_t - amount);
    _u1token_bal_t -= amount;

    let u2token_bal = user_balance(&mut test_runner, &user2, token_addr);
    assert_eq!(u2token_bal, _u2token_bal_t + amount);
    _u2token_bal_t += amount;

    //----------------== mint_to_vault
    let amount = dec!(1000);
    let badge_amount = dec!(3);
    invoke_badge_access_decimal(
        &mut test_runner,
        &user1,
        component,
        "mint_to_vault",
        amount,
        admin_badge_addr,
        badge_amount,
    );
    let data = get_data(&mut test_runner, &user1, component);
    assert_eq!(data.2, total_supply + amount);
    total_supply += amount;
    assert_eq!(data.1, total_supply_original + amount);
    _vault_amount = total_supply_original + amount;
    println!("vault amount increased accordingly");

    //----------------== withdraw_to_bucket
    let amount = dec!(937);
    let badge_amount = dec!(3);
    invoke_badge_access_decimal(
        &mut test_runner,
        &user1,
        component,
        "withdraw_to_bucket",
        amount,
        admin_badge_addr,
        badge_amount,
    );
    let data = get_data(&mut test_runner, &user1, component);
    assert_eq!(data.2, total_supply);
    assert_eq!(data.1, _vault_amount - amount);
    _vault_amount -= amount;
    println!("vault amount decreased accordingly");

    let u1token_bal = user_balance(&mut test_runner, &user1, token_addr);
    println!("u1token_bal:{}", u1token_bal);
    assert_eq!(u1token_bal, _u1token_bal_t + amount);
    _u1token_bal_t += amount;
    println!("u1token_bal increased accordingly");

    //----------------== deposit
    let amount = dec!(37);
    let badge_amount = dec!(3);
    invoke_badge_access_with_bucket(
        &mut test_runner,
        &user1,
        component,
        "deposit_to_vault",
        amount,
        token_addr,
        admin_badge_addr,
        badge_amount,
    );
    let data = get_data(&mut test_runner, &user1, component);
    assert_eq!(data.2, total_supply);
    assert_eq!(data.1, _vault_amount + amount);
    _vault_amount += amount;
    println!("vault amount increased accordingly");

    let u1token_bal = user_balance(&mut test_runner, &user1, token_addr);
    println!("u1token_bal:{}", u1token_bal);
    assert_eq!(u1token_bal, _u1token_bal_t - amount);
    _u1token_bal_t -= amount;
    println!("u1token_bal decreased accordingly");

    //----------------== burn_in_vault
    let amount = dec!(100);
    let badge_amount = dec!(3);
    invoke_badge_access_decimal(
        &mut test_runner,
        &user1,
        component,
        "burn_in_vault",
        amount,
        admin_badge_addr,
        badge_amount,
    );
    let data = get_data(&mut test_runner, &user1, component);
    assert_eq!(data.2, total_supply - amount);
    total_supply -= amount;
    assert_eq!(data.1, _vault_amount - amount);
    _vault_amount -= amount;
    println!("vault amount decreased accordingly");

    //----------------== burn_in_bucket
    let amount = dec!(12);
    let badge_amount = dec!(3);
    invoke_badge_access_with_bucket(
        &mut test_runner,
        &user1,
        component,
        "burn_in_bucket",
        amount,
        token_addr,
        admin_badge_addr,
        badge_amount,
    );
    let data = get_data(&mut test_runner, &user1, component);
    assert_eq!(data.2, total_supply - amount);
    total_supply -= amount;
    assert_eq!(data.1, _vault_amount);
    println!("vault amount decreased accordingly");

    let u1token_bal = user_balance(&mut test_runner, &user1, token_addr);
    println!("u1token_bal:{}", u1token_bal);
    assert_eq!(u1token_bal, _u1token_bal_t - amount);
    _u1token_bal_t -= amount;
    println!("u1token_bal decreased accordingly");

    //----------------== read_metadata
    let function_name = "get_token_metadata";
    let txn_receipt = call_function(
        &mut test_runner,
        &user1,
        package_addr,
        blueprint_name,
        function_name,
        token_addr,
        keys_owned.clone(),
    );
    let data: (u8, NonFungibleIdType, Decimal, Vec<Option<String>>) = txn_receipt.output(1);
    println!("call_function output:{:?}\n", data);
    let data_values = vec_option_string(data.3);

    assert_eq!(data.0, 18);
    assert_eq!(data.1, NonFungibleIdType::U32);
    assert_eq!(data.2, total_supply);
    assert!(
        do_vecs_match(&data_values, &values_owned),
        "token metadata do not match"
    );
    println!("all metadata match accordingly");

    //----------------== update_metadata
    let badge_amount = dec!(3);
    let key = "name".to_owned();
    let value = "Gold Coin".to_owned();
    update_metadata(
        &mut test_runner,
        &user1,
        component,
        key.clone(),
        value.clone(),
        admin_badge_addr,
        badge_amount,
    );

    //----------------== read_metadata
    let function_name = "get_token_metadata";
    let txn_receipt = call_function(
        &mut test_runner,
        &user1,
        package_addr,
        blueprint_name,
        function_name,
        token_addr,
        keys_owned.clone(),
    );
    let data: (u8, NonFungibleIdType, Decimal, Vec<Option<String>>) = txn_receipt.output(1);
    println!("call_function output:{:?}\n", data);
    let data_values = vec_option_string(data.3);

    assert_eq!(data.0, 18);
    assert_eq!(data.1, NonFungibleIdType::U32);
    assert_eq!(data.2, total_supply);

    let (values_owned, _) = find_replace_two_vec(values_owned, &keys_owned, &key, value.clone());
    println!("new values_owned:{:?}", values_owned);
    assert!(
        do_vecs_match(&data_values, &values_owned),
        "token metadata do not match"
    );
    println!("all metadata match accordingly");

    //----------------== set_token_stage_three
    let badge_amount = dec!(3);
    invoke_badge_access(
        &mut test_runner,
        &user1,
        component,
        "set_token_stage_three",
        admin_badge_addr,
        badge_amount,
    );
}
