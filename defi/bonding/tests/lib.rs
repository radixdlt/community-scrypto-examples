use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;
use scrypto_unit::*;

fn setup_fixture<'a, L: SubstateStore>(env: &mut TestEnv<'a, L>) -> (User, User, ResourceAddress) {
    let _root = env.create_user("root");
    env.acting_as("root");

    // magic some reserve tokens into existance
    let reserve_def = env.create_token(dec!(2_000_000));

    // create an owner and an investor
    let owner = env.create_user("owner");
    let investor = env.create_user("investor");

    // transfer some reserve tokens to them to use in testing
    let receipt = env.transfer_resource(dec!(1_000_000), &reserve_def, &owner);
    println!("transfer 100k to owner: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());
    env.transfer_resource(dec!(1_000_000), &reserve_def, &investor);
    println!("transfer 100k to investor: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());

    // use a publisher to publish the package
    let _pubisher = env.create_user("publisher");
    env.acting_as("publisher");

    const PACKAGE: &str = "bonding";

    env.publish_package(PACKAGE, include_package!("scrypto_bonding"));
    env.using_package(PACKAGE);

    (owner, investor, reserve_def)
}


#[test]
fn test_1() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = TestEnv::new(&mut ledger);
    let (owner, investor, reserve_def) = setup_fixture(&mut env);

    const BLUEPRINT: &str = "BondingAMM";
    // switch to the owner to instantiate a new BondingAMM
    env.acting_as("owner");
    let user = owner;

    //pub fn new(initial_reserve: BucketOf<RESERVE>, continuous_name: String, continuous_symbol: String, bonding_curve: Option<Component>) -> Component {
    use radix_engine::transaction::TransactionBuilder;
    let mut receipt = env.call_function_aux(
        BLUEPRINT,
        "new_default",
        vec![
            get_param_bucket!(dec!(60000u64), reserve_def, env.current_user.unwrap().account),
            get_param_value!("Continuous"),
            get_param_value!("XC"),
            //        "".to_owned(), // hmm now do I pass 'None'
        ],
    );
    println!("new_default: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());

    // save off addresses
    // this is brittle checking the defs based on order...
    let _control_addr = receipt.new_resource_addresses[0]; // this should be the CONTROL
    let continuous_addr = receipt.new_resource_addresses[1]; // this should be the CONTINUOUS

    // reserve check
    let expected_reserve_in_account: Decimal = (1_000_000i128 - 60000i128).into();
    let reserve_in_account = env.get_amount_for_rd(user.account, reserve_def);
    println!(
        "expected reserve after new: {}",
        expected_reserve_in_account
    );
    println!("actual reserve after new: {}", reserve_in_account);
    assert_eq!(reserve_in_account, expected_reserve_in_account);

    // continuous check
    let expected_continuous_in_account = Decimal::from_str("300000").unwrap();
    let continuous_in_account = env.get_amount_for_rd(user.account, continuous_addr);
    assert_eq!(continuous_in_account, expected_continuous_in_account);

    // don't do this ...
    // let component = receipt.component(0).unwrap();
    // since it is not the returned Component but just the one that is created first.

    // instead do this
    let ret: (ComponentAddress, Bucket) = return_of_call_function(&mut receipt, BLUEPRINT);
    let amm = ret.0;

    // now switch to an investor
    env.acting_as("investor");
    let user = investor;

    // call buy w/ 300 reserve (just like in test_1) with no (0) minimum expected back
    let receipt = env.call_method_aux(
        amm,
        "buy",
        vec![ //format!("300,{}", reserve_def.address()), format!("0")],
            get_param_bucket!(dec!(300u64), reserve_def, env.current_user.unwrap().account),
            get_param_value!(dec!(0u32)),
        ]
    );
    println!("buy: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());

    // reserve
    let expected_reserve_in_account: Decimal = dec!(1_000_000u64) - dec!(300u64);
    let reserve_in_account = env.get_amount_for_rd(user.account, reserve_def);
    println!(
        "expected reserve after buy: {}",
        expected_reserve_in_account
    );
    println!("actual   reserve after buy: {}", reserve_in_account);
    assert_eq!(reserve_in_account, expected_reserve_in_account);
    // continuous
    let expected_continuous_in_account = Decimal::from_str("299.401793723844635041").unwrap();
    let continuous_in_account = env.get_amount_for_rd(user.account, continuous_addr);
    assert_eq!(continuous_in_account, expected_continuous_in_account);

    // now sell it back

    // call sell w/ same 299... received with no minimum back
    let receipt = env.call_method_aux(
        amm,
        "sell",
        vec![
            //format!("299.401793723844635041,{}", continuous_addr),
            //format!("0"),
            get_param_bucket!(dec!("299.401793723844635041"), continuous_addr, env.current_user.unwrap().account),
            get_param_value!(dec!(0u32)),
        ],
    );
    println!("sell: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());
    // check balances
    // reserve
    let expected_reserve_in_account: Decimal = expected_reserve_in_account + dec!(300u64);
    let reserve_in_account = env.get_amount_for_rd(user.account, reserve_def);
    println!(
        "expected reserve after sell: {}",
        expected_reserve_in_account
    );
    println!("actual   reserve after sell: {}", reserve_in_account);
    assert_eq!(reserve_in_account, expected_reserve_in_account);
    // continuous
    let expected_continuous_in_account = Decimal::from_str("0").unwrap();
    let continuous_in_account = env.get_amount_for_rd(user.account, continuous_addr);
    assert_eq!(continuous_in_account, expected_continuous_in_account);
}

#[test]
fn test_2_basic_curve() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = TestEnv::new(&mut ledger);
    let (owner, investor, reserve_def) = setup_fixture(&mut env);

    const BLUEPRINT: &str = "BondingAMM";
    const CURVE_BLUEPRINT: &str = "BasicBondingCurve";

    // switch to the owner to instantiate a new BondingAMM
    env.acting_as("owner");
    let user = owner;

    // create basic curve
    let mut receipt = env.call_function(CURVE_BLUEPRINT, "new", vec![]);
    // get the new compoent/address so it can be passed into `new_with_curve`
    let ret: ComponentAddress = return_of_call_function(&mut receipt, CURVE_BLUEPRINT);
    let basic_curve = ret;

    // instantiate amm with basic curve
    let mut receipt = env.call_function_aux(
        BLUEPRINT,
        "new_with_curve",
        vec![
            //format!("60000,{}", reserve_def),
            //"Continuous".to_owned(),
            //"XC".to_owned(),
            //format!("{}", basic_curve),
            get_param_bucket!(dec!(60000u64), reserve_def, env.current_user.unwrap().account),
            get_param_value!("Continuous"),
            get_param_value!("XC"),
            get_param_value!(basic_curve),
        ],
    );
    println!("new_default: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());

    // save off addresses
    // this is brittle checking the defs based on order...
    let _control_addr = receipt.new_resource_addresses[0]; // this should be the CONTROL
    let continuous_addr = receipt.new_resource_addresses[1]; // this should be the CONTINUOUS

    // reserve check
    let expected_reserve_in_account: Decimal = (1_000_000i128 - 60000i128).into();
    let reserve_in_account = env.get_amount_for_rd(user.account, reserve_def);
    println!(
        "expected reserve after new: {}",
        expected_reserve_in_account
    );
    println!("actual reserve after new: {}", reserve_in_account);
    assert_eq!(reserve_in_account, expected_reserve_in_account);

    // continuous check
    let expected_continuous_in_account = Decimal::from_str("60000").unwrap();
    let continuous_in_account = env.get_amount_for_rd(user.account, continuous_addr);
    assert_eq!(continuous_in_account, expected_continuous_in_account);

    // don't do this ...
    // let component = receipt.component(0).unwrap();
    // since it is not the returned Component but just the one that is created first.

    // do this instead
    let ret: (ComponentAddress, Bucket) = return_of_call_function(&mut receipt, BLUEPRINT);
    let amm = ret.0;

    // now switch to an investor
    env.acting_as("investor");
    let user = investor;

    // call buy w/ 300 reserve (just like in test_1) with no (0) minimum expected back
    let receipt = env.call_method_aux(
        amm,
        "buy",
        vec![ // format!("300,{}", reserve_def), format!("0")],
            get_param_bucket!(dec!(300u64), reserve_def, env.current_user.unwrap().account),
            get_param_value!(dec!(0u32)),
        ]
    );
    println!("buy: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());

    // check balances
    // reserve
    let expected_reserve_in_account: Decimal = (1_000_000i128 - 300i128).into();
    let reserve_in_account = env.get_amount_for_rd(user.account, reserve_def);
    println!(
        "expected reserve after buy: {}",
        expected_reserve_in_account
    );
    println!("actual   reserve after buy: {}", reserve_in_account);
    assert_eq!(reserve_in_account, expected_reserve_in_account);
    // continuous
    let expected_continuous_in_account = Decimal::from_str("300.0").unwrap(); // 1:1 basic curve should have same amount
    let continuous_in_account = env.get_amount_for_rd(user.account, continuous_addr);
    assert_eq!(continuous_in_account, expected_continuous_in_account);

    // now sell it back

    let receipt = env.call_method_aux(
        amm,
        "sell",
        vec![ // format!("300,{}", continuous_addr), format!("0")],
            get_param_bucket!(dec!(300u64), continuous_addr, env.current_user.unwrap().account),
            get_param_value!(dec!(0u32)),
        ]
    );
    println!("sell: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());

    // check balances
    // reserve
    let expected_reserve_in_account: Decimal = expected_reserve_in_account + 300u32;
    let reserve_in_account = env.get_amount_for_rd(user.account, reserve_def);
    println!(
        "expected reserve after sell: {}",
        expected_reserve_in_account
    );
    println!("actual   reserve after sell: {}", reserve_in_account);
    assert_eq!(reserve_in_account, expected_reserve_in_account);
    // continuous
    let expected_continuous_in_account = Decimal::from_str("0").unwrap();
    let continuous_in_account = env.get_amount_for_rd(user.account, continuous_addr);
    assert_eq!(continuous_in_account, expected_continuous_in_account);
}

#[test]
fn test_3_quotes() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = TestEnv::new(&mut ledger);
    let (owner, investor, reserve_def) = setup_fixture(&mut env);

    const BLUEPRINT: &str = "BondingAMM";
    const CURVE_BLUEPRINT: &str = "BasicBondingCurve";

    // switch to the owner to instantiate a new BondingAMM
    env.acting_as("owner");
    let _user = owner;

    // create basic curve
    let mut receipt = env.call_function(CURVE_BLUEPRINT, "new", vec![]);
    // get the new compoent/address so it can be passed into `new_with_curve`
    let ret: ComponentAddress = return_of_call_function(&mut receipt, CURVE_BLUEPRINT);
    let basic_curve = ret;

    // instantiate amm with basic curve
    let mut receipt = env.call_function_aux(
        BLUEPRINT,
        "new_with_curve",
        vec![
            // format!("60000,{}", reserve_def),
            // "Continuous".to_owned(),
            // "XC".to_owned(),
            // format!("{}", basic_curve),
            get_param_bucket!(dec!(60000u64), reserve_def, env.current_user.unwrap().account),
            get_param_value!("Continuous"),
            get_param_value!("XC"),
            get_param_value!(basic_curve),
        ],
    );
    println!("new_default: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());

    // save off addresses
    // this is brittle checking the defs based on order...
    let _control_addr = receipt.new_resource_addresses[0]; // this should be the CONTROL
    let _continuous_addr = receipt.new_resource_addresses[1]; // this should be the CONTINUOUS

    let ret: (ComponentAddress, Bucket) = return_of_call_function(&mut receipt, BLUEPRINT);
    let amm = ret.0;

    // now switch to an investor
    env.acting_as("investor");
    let user = investor;

    // now try to get a buy quote amount and see if it's right
    let mut receipt = env.call_method(amm, "get_buy_quote_amount", vec![scrypto_encode(&dec!("300"))]);
    println!("buy_quote: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());
    let quote: Decimal = return_of_call_method(&mut receipt, "get_buy_quote_amount");
    assert_eq!(quote, dec!(300u64));

    // now try to get a sell quote amount and see if it's right
    let mut receipt = env.call_method(
        amm,
        "get_sell_quote_amount",
        vec![scrypto_encode(&dec!("300"))],
    );
    println!("sell_quote_amount: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());
    let quote: Decimal = return_of_call_method(&mut receipt, "get_sell_quote_amount");
    assert_eq!(quote, dec!(300u64));

    /* this test no longer works in v0.3.0 because we can't return a BucketRef.
     * The code is still a good example of something another blueprint might use
     * but we just can't easily test it.  Leave here to update later

    // now try to get a seel quote (BucketRef) ...but can't check it easily outside of wasm
    let receipt = env.call_method(&amm.address(), "get_sell_quote", vec![
        format!("300"),
    ]);
    println!("sell_quote: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());

    // cant introspect the BucketRef, but at least the tx succeeded

    */

    // verify we didn't somehow actually get any RESERVE back from the BucketRef stuff
    let expected_reserve_in_account: Decimal = dec!(1_000_000u64);
    let reserve_in_account = env.get_amount_for_rd(user.account, reserve_def);
    assert_eq!(reserve_in_account, expected_reserve_in_account);
}
