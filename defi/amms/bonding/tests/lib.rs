use scrypto::prelude::*;
use radix_engine::ledger::*;
use scrypto_unit::*;

fn setup_fixture<'a, L: Ledger>(env: &mut TestEnv<'a, L>) -> (User, User, ResourceDef) {
    let _root = env.create_user("root");
    env.acting_as("root");

    // magic some reserve tokens into existance
    let reserve_def = env.create_token(2_000_000.into());

    // create an owner and an investor
    let owner = env.create_user("owner");
    let investor = env.create_user("investor");

    // transfer some reserve tokens to them to use in testing
    let receipt = env.transfer_resource(1_000_000.into(), &reserve_def, &owner);
    println!("transfer 100k to owner: receipt: {:?}", receipt);
    assert!(receipt.success);
    env.transfer_resource(1_000_000.into(), &reserve_def, &investor);
    println!("transfer 100k to investor: receipt: {:?}", receipt);
    assert!(receipt.success);

    // use a publisher to publish the package
    let _pubisher = env.create_user("publisher");
    env.acting_as("publisher");

    const PACKAGE: &str = "bonding";

    env.publish_package(PACKAGE, include_code!("scrypto_bonding"));
    env.using_package(PACKAGE);

    (owner, investor, reserve_def)
}

#[test]
fn test_1() {
    let mut ledger = InMemoryLedger::with_bootstrap();
    let mut env = TestEnv::new(&mut ledger);
    let (owner, investor, reserve_def) = setup_fixture(&mut env);

    const BLUEPRINT: &str = "BondingAMM";
    // switch to the owner to instantiate a new BondingAMM
    env.acting_as("owner");
    let user = owner;

    //pub fn new(initial_reserve: BucketOf<RESERVE>, continuous_name: String, continuous_symbol: String, bonding_curve: Option<Component>) -> Component {
    let mut receipt = env.call_function(BLUEPRINT, "new_default", vec![
        format!("60000,{}", reserve_def.address()),
        "Continuous".to_owned(),
        "XC".to_owned(),
//        "".to_owned(), // hmm now do I pass 'None'
        ]);
    println!("new_default: receipt: {:?}", receipt);
    assert!(receipt.success);

    // save off addresses
    // this is brittle checking the defs based on order...
    let _control_addr = receipt.resource_def(0).unwrap(); // this should be the CONTROL
    let continuous_addr = receipt.resource_def(1).unwrap(); // this should be the CONTINUOUS

    // reserve check
    let expected_reserve_in_account: Decimal = (1_000_000i128 - 60000i128).into();
    let reserve_in_account = env.get_amount_for_rd(user.account, reserve_def.address());  
    println!("expected reserve after new: {}", expected_reserve_in_account);
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
    let ret: (Component, Bucket) = return_of_call_function(&mut receipt, BLUEPRINT);
    let amm = ret.0;

    // now switch to an investor
    env.acting_as("investor");
    let user = investor;

    // call buy w/ 300 reserve (just like in test_1) with no (0) minimum expected back
    let receipt = env.call_method(&amm.address(), "buy", vec![
        format!("300,{}", reserve_def.address()),
        format!("0"),
    ]);
    println!("buy: receipt: {:?}", receipt);
    assert!(receipt.success);

    // reserve
    let expected_reserve_in_account: Decimal = Decimal::from(1_000_000) - 300;
    let reserve_in_account = env.get_amount_for_rd(user.account, reserve_def.address());  
    println!("expected reserve after buy: {}", expected_reserve_in_account);
    println!("actual   reserve after buy: {}", reserve_in_account);
    assert_eq!(reserve_in_account, expected_reserve_in_account);
    // continuous
    let expected_continuous_in_account = Decimal::from_str("299.401793723844635041").unwrap();
    let continuous_in_account = env.get_amount_for_rd(user.account, continuous_addr);
    assert_eq!(continuous_in_account, expected_continuous_in_account);

    // now sell it back

    // call sell w/ same 299... received with no minimum back
    let receipt = env.call_method(&amm.address(), "sell", vec![
        format!("299.401793723844635041,{}", continuous_addr),
        format!("0"),
    ]);
    println!("sell: receipt: {:?}", receipt);
    assert!(receipt.success);
    // check balances
    // reserve
    let expected_reserve_in_account: Decimal = expected_reserve_in_account + 300;
    let reserve_in_account = env.get_amount_for_rd(user.account, reserve_def.address());  
    println!("expected reserve after sell: {}", expected_reserve_in_account);
    println!("actual   reserve after sell: {}", reserve_in_account);
    assert_eq!(reserve_in_account, expected_reserve_in_account);
    // continuous
    let expected_continuous_in_account = Decimal::from_str("0").unwrap();
    let continuous_in_account = env.get_amount_for_rd(user.account, continuous_addr);
    assert_eq!(continuous_in_account, expected_continuous_in_account);
}

#[test]
fn test_2_basic_curve() {
    let mut ledger = InMemoryLedger::with_bootstrap();
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
    let ret: Component = return_of_call_function(&mut receipt, CURVE_BLUEPRINT);
    let basic_curve = ret;

    // instantiate amm with basic curve
    let mut receipt = env.call_function(BLUEPRINT, "new_with_curve", vec![
        format!("60000,{}", reserve_def.address()),
        "Continuous".to_owned(),
        "XC".to_owned(),
        format!("{}", basic_curve.address()),
        ]);
    println!("new_default: receipt: {:?}", receipt);
    assert!(receipt.success);

    // save off addresses
    // this is brittle checking the defs based on order...
    let _control_addr = receipt.resource_def(0).unwrap(); // this should be the CONTROL
    let continuous_addr = receipt.resource_def(1).unwrap(); // this should be the CONTINUOUS

    // reserve check
    let expected_reserve_in_account: Decimal = (1_000_000i128 - 60000i128).into();
    let reserve_in_account = env.get_amount_for_rd(user.account, reserve_def.address());  
    println!("expected reserve after new: {}", expected_reserve_in_account);
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
    let ret: (Component, Bucket) = return_of_call_function(&mut receipt, BLUEPRINT);
    let amm = ret.0;

    // now switch to an investor
    env.acting_as("investor");
    let user = investor;

    // call buy w/ 300 reserve (just like in test_1) with no (0) minimum expected back
    let receipt = env.call_method(&amm.address(), "buy", vec![
        format!("300,{}", reserve_def.address()),
        format!("0"),
    ]);
    println!("buy: receipt: {:?}", receipt);
    assert!(receipt.success);

    // check balances
    // reserve
    let expected_reserve_in_account: Decimal = (1_000_000i128 - 300i128).into();
    let reserve_in_account = env.get_amount_for_rd(user.account, reserve_def.address());  
    println!("expected reserve after buy: {}", expected_reserve_in_account);
    println!("actual   reserve after buy: {}", reserve_in_account);
    assert_eq!(reserve_in_account, expected_reserve_in_account);
    // continuous
    let expected_continuous_in_account = Decimal::from_str("300.0").unwrap(); // 1:1 basic curve should have same amount
    let continuous_in_account = env.get_amount_for_rd(user.account, continuous_addr);
    assert_eq!(continuous_in_account, expected_continuous_in_account);

    // now sell it back

    let receipt = env.call_method(&amm.address(), "sell", vec![
        format!("300,{}", continuous_addr),
        format!("0"),
    ]);
    println!("sell: receipt: {:?}", receipt);
    assert!(receipt.success);

    // check balances
    // reserve
    let expected_reserve_in_account: Decimal = expected_reserve_in_account + 300;
    let reserve_in_account = env.get_amount_for_rd(user.account, reserve_def.address());  
    println!("expected reserve after sell: {}", expected_reserve_in_account);
    println!("actual   reserve after sell: {}", reserve_in_account);
    assert_eq!(reserve_in_account, expected_reserve_in_account);
    // continuous
    let expected_continuous_in_account = Decimal::from_str("0").unwrap();
    let continuous_in_account = env.get_amount_for_rd(user.account, continuous_addr);
    assert_eq!(continuous_in_account, expected_continuous_in_account);
}

#[test]
fn test_3_quotes() {
    let mut ledger = InMemoryLedger::with_bootstrap();
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
    let ret: Component = return_of_call_function(&mut receipt, CURVE_BLUEPRINT);
    let basic_curve = ret;

    // instantiate amm with basic curve
    let mut receipt = env.call_function(BLUEPRINT, "new_with_curve", vec![
        format!("60000,{}", reserve_def.address()),
        "Continuous".to_owned(),
        "XC".to_owned(),
        format!("{}", basic_curve.address()),
        ]);
    println!("new_default: receipt: {:?}", receipt);
    assert!(receipt.success);

    // save off addresses
    // this is brittle checking the defs based on order...
    let _control_addr = receipt.resource_def(0).unwrap(); // this should be the CONTROL
    let _continuous_addr = receipt.resource_def(1).unwrap(); // this should be the CONTINUOUS

    let ret: (Component, Bucket) = return_of_call_function(&mut receipt, BLUEPRINT);
    let amm = ret.0;

    // now switch to an investor
    env.acting_as("investor");
    let user = investor;

    // now try to get a buy quote amount and see if it's right
    let mut receipt = env.call_method(&amm.address(), "get_buy_quote_amount", vec![
        format!("300"),
    ]);
    println!("buy_quote: receipt: {:?}", receipt);
    assert!(receipt.success);
    let quote: Decimal = return_of_call_method(&mut receipt, "get_buy_quote_amount");
    assert_eq!(quote, 300.into());

    // now try to get a sell quote amount and see if it's right
    let mut receipt = env.call_method(&amm.address(), "get_sell_quote_amount", vec![
        format!("300"),
    ]);
    println!("sell_quote_amount: receipt: {:?}", receipt);
    assert!(receipt.success);
    let quote: Decimal = return_of_call_method(&mut receipt, "get_sell_quote_amount");
    assert_eq!(quote, 300.into());

    // now try to get a seel quote (BucketRef) ...but can't check it easily outside of wasm
    let receipt = env.call_method(&amm.address(), "get_sell_quote", vec![
        format!("300"),
    ]);
    println!("sell_quote: receipt: {:?}", receipt);
    assert!(receipt.success);

    // cant introspect the BucketRef, but at least the tx succeeded

    // verify we didn't somehow actually get any RESERVE back from the BucketRef stuff
    let expected_reserve_in_account: Decimal = 1_000_000.into();
    let reserve_in_account = env.get_amount_for_rd(user.account, reserve_def.address());  
    assert_eq!(reserve_in_account, expected_reserve_in_account);
}
