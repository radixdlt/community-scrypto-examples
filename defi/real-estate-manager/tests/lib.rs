use radix_engine::ledger::*;
use scrypto::prelude::*;
use scrypto_unit::*;

fn setup_fixture<'a, L: SubstateStore>(env: &mut TestEnv<'a, L>) -> (User, Vec<User>, User, User, ResourceAddress) {

    let _root = env.create_user("root");
    env.acting_as("root");

    // create new resoure
    let resoure = env.create_token("8000000".into());

    // create testing users
    let authority = env.create_user("authority");
    let user1 = env.create_user("user1");
    let user2 = env.create_user("user2");
    let user3 = env.create_user("user3");
    let user4 = env.create_user("user4");
    let user5 = env.create_user("user5");
    let market_host = env.create_user("market_host");
    let construction_institute = env.create_user("construction_institute");

    // transfer some reserve tokens to them to use in testing
    let receipt = env.transfer_resource("1000000".into(), &resoure, &authority);
    println!("transfer 100k to owner: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());

    let receipt = env.transfer_resource("1000000".into(), &resoure, &user1);
    println!("transfer 100k to investor: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());

    let receipt = env.transfer_resource("1000000".into(), &resoure, &user2);
    println!("transfer 100k to investor: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());


    let receipt = env.transfer_resource("1000000".into(), &resoure, &user3);
    println!("transfer 100k to investor: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());


    let receipt = env.transfer_resource("1000000".into(), &resoure, &user4);
    println!("transfer 100k to investor: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());


    let receipt = env.transfer_resource("1000000".into(), &resoure, &user5);
    println!("transfer 100k to investor: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());


    let receipt = env.transfer_resource("1000000".into(), &resoure, &market_host);
    println!("transfer 100k to investor: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());


    let receipt = env.transfer_resource("1000000".into(), &resoure, &construction_institute);
    println!("transfer 100k to investor: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());

    const PACKAGE: &str = "real-estate-manager";

    let package = compile_package!();

    let addr = env.executor.publish_package(package).unwrap();
    env.packages.insert(String::from(PACKAGE), addr);
    env.using_package(PACKAGE);

    (authority, [user1, user2, user3, user4, user5].into(), market_host, construction_institute, resoure)

}

#[test]
fn test() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = TestEnv::new(&mut ledger);
    let (authority, users, market_host, construction_institute, resource) = setup_fixture(&mut env);

    const BLUEPRINT: &str = "RealEstateService";
    // switch to the authority to instantiate a new Real estate service
    env.acting_as("authority");

    //pub fn new(name: String, tax: Decimal, rate: Decimal, medium_token: ResourceAddress) -> (ComponentAddress, Bucket) {
    let mut receipt = env.call_function(
        BLUEPRINT,
        "new",
        vec![
            scrypto_encode("Hello"),
            dec!("2").to_vec(),
            dec!("100").to_vec(),
            resource.to_vec()
        ]);

    println!("new_default: receipt: {:?}", receipt);
    assert!(receipt.result.is_ok());

}
