// TODO: Migrate this test file to v0.4 once scrypto-unit is updated

use radix_engine::ledger::*;
use scrypto::prelude::*;
use scrypto_unit::*;

const PACKAGE: &str = "dex-challenge";
const BLUEPRINT_DEX: &str = "Dex";

#[test]
fn test_ask_order_exact_fill() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let env = TestEnv::new(&mut ledger);
    let mut dex_fixture = setup_fixture(env);

    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("taker", dex_fixture.base_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("taker", dex_fixture.quote_resource, 1_000_000.into());

    dex_fixture.env.acting_as(dex_fixture.user_maker.name);
    let order_key = dex_fixture.new_limit_order(
        FungibleBucket(100_000.into(), dex_fixture.base_resource),
        10.into(),
    );

    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 900_000.into());

    dex_fixture.env.acting_as(dex_fixture.user_taker.name);
    dex_fixture.new_market_order(FungibleBucket(1_000_000.into(), dex_fixture.quote_resource));
    dex_fixture.assert_resource_amount("taker", dex_fixture.base_resource, 1_100_000.into());
    dex_fixture.assert_resource_amount("taker", dex_fixture.quote_resource, 0.into());

    dex_fixture.env.acting_as(dex_fixture.user_maker.name);
    dex_fixture.close_limit_order(NonFungibleBucket(order_key, dex_fixture.order_resource));
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 900_000.into());
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 2_000_000.into());
}

#[test]
fn test_bid_order_exact_fill() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let env = TestEnv::new(&mut ledger);
    let mut dex_fixture = setup_fixture(env);

    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("taker", dex_fixture.base_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("taker", dex_fixture.quote_resource, 1_000_000.into());

    dex_fixture.env.acting_as(dex_fixture.user_maker.name);
    let order_key = dex_fixture.new_limit_order(
        FungibleBucket(1_000_000.into(), dex_fixture.quote_resource),
        10.into(),
    );

    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 0.into());

    dex_fixture.env.acting_as(dex_fixture.user_taker.name);
    dex_fixture.new_market_order(FungibleBucket(100_000.into(), dex_fixture.base_resource));
    dex_fixture.assert_resource_amount("taker", dex_fixture.base_resource, 900_000.into());
    dex_fixture.assert_resource_amount("taker", dex_fixture.quote_resource, 2_000_000.into());

    dex_fixture.env.acting_as(dex_fixture.user_maker.name);
    dex_fixture.close_limit_order(NonFungibleBucket(order_key, dex_fixture.order_resource));
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 1_100_000.into());
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 0.into());
}

#[test]
fn test_ask_order_partial_fill() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let env = TestEnv::new(&mut ledger);
    let mut dex_fixture = setup_fixture(env);

    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("taker", dex_fixture.base_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("taker", dex_fixture.quote_resource, 1_000_000.into());

    dex_fixture.env.acting_as(dex_fixture.user_maker.name);
    // Order 1 is worse than order 2 and will be filled last
    let order1_key = dex_fixture.new_limit_order(
        FungibleBucket(50_000.into(), dex_fixture.base_resource),
        10.into(),
    );
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 950_000.into());

    // Order 2 is better than order 1 and will be filled first
    let order2_key = dex_fixture.new_limit_order(
        FungibleBucket(50_000.into(), dex_fixture.base_resource),
        5.into(),
    );
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 900_000.into());

    dex_fixture.env.acting_as(dex_fixture.user_taker.name);
    dex_fixture.new_market_order(FungibleBucket(700_000.into(), dex_fixture.quote_resource));
    dex_fixture.assert_resource_amount("taker", dex_fixture.base_resource, 1_095_000.into());
    dex_fixture.assert_resource_amount("taker", dex_fixture.quote_resource, 300_000.into());

    dex_fixture.env.acting_as(dex_fixture.user_maker.name);
    // Order 1 has only been filled partially
    dex_fixture.close_limit_order(NonFungibleBucket(order1_key, dex_fixture.order_resource));
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 905_000.into());
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 1_450_000.into());

    dex_fixture.close_limit_order(NonFungibleBucket(order2_key, dex_fixture.order_resource));
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 905_000.into());
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 1_700_000.into());
}

#[test]
fn test_bid_order_partial_fill() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let env = TestEnv::new(&mut ledger);
    let mut dex_fixture = setup_fixture(env);

    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("taker", dex_fixture.base_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("taker", dex_fixture.quote_resource, 1_000_000.into());

    dex_fixture.env.acting_as(dex_fixture.user_maker.name);
    // Order 1 is worse than order 2 and will be filled last
    let order1_key = dex_fixture.new_limit_order(
        FungibleBucket(500_000.into(), dex_fixture.quote_resource),
        5.into(),
    );
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 500_000.into());

    // Order 2 is better than order 1 and will be filled first
    let order2_key = dex_fixture.new_limit_order(
        FungibleBucket(500_000.into(), dex_fixture.quote_resource),
        10.into(),
    );
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 0.into());

    dex_fixture.env.acting_as(dex_fixture.user_taker.name);
    dex_fixture.new_market_order(FungibleBucket(70_000.into(), dex_fixture.base_resource));
    dex_fixture.assert_resource_amount("taker", dex_fixture.base_resource, 930_000.into());
    dex_fixture.assert_resource_amount("taker", dex_fixture.quote_resource, 1_600_000.into());

    dex_fixture.env.acting_as(dex_fixture.user_maker.name);
    // Order 1 has only been filled partially
    dex_fixture.close_limit_order(NonFungibleBucket(order1_key, dex_fixture.order_resource));
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 1_020_000.into());
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 400_000.into());

    dex_fixture.close_limit_order(NonFungibleBucket(order2_key, dex_fixture.order_resource));
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 1_070_000.into());
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 400_000.into());
}

#[test]
fn test_ask_order_fractional_numbers() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let env = TestEnv::new(&mut ledger);
    let mut dex_fixture = setup_fixture(env);

    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("taker", dex_fixture.base_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("taker", dex_fixture.quote_resource, 1_000_000.into());

    dex_fixture.env.acting_as(dex_fixture.user_maker.name);
    let order1_key = dex_fixture.new_limit_order(
        FungibleBucket(10.into(), dex_fixture.base_resource),
        Decimal::from(1) / Decimal::from(3),
    );
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 999_990.into());

    let order2_key = dex_fixture.new_limit_order(
        FungibleBucket(10.into(), dex_fixture.base_resource), Decimal::from(1) / Decimal::from(3),);
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 999_980.into());

    let order3_key = dex_fixture.new_limit_order(
        FungibleBucket(10.into(), dex_fixture.base_resource), Decimal::from(1) / Decimal::from(3),);
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 999_970.into());

    dex_fixture.env.acting_as(dex_fixture.user_taker.name);
    dex_fixture.new_market_order(FungibleBucket(6.into(), dex_fixture.quote_resource));
    dex_fixture.assert_resource_amount("taker", dex_fixture.base_resource, Decimal::from("1000018.000000000000000018"), );
    dex_fixture.assert_resource_amount("taker", dex_fixture.quote_resource, Decimal::from("999994.000000000000000001"),);

    dex_fixture.env.acting_as(dex_fixture.user_maker.name);
    dex_fixture.close_limit_order(NonFungibleBucket(order1_key, dex_fixture.order_resource));
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 999_970.into());
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, Decimal::from("1000003.33333333333333333"),);

    dex_fixture.close_limit_order(NonFungibleBucket(order2_key, dex_fixture.order_resource));
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, Decimal::from("999971.999999999999999982"),);
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, Decimal::from("1000005.999999999999999999").into(),);

    dex_fixture.close_limit_order(NonFungibleBucket(order3_key, dex_fixture.order_resource));
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, Decimal::from("999981.999999999999999982"),);
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, Decimal::from("1000005.999999999999999999"),);
}

#[test]
fn test_bid_order_fractional_numbers() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let env = TestEnv::new(&mut ledger);
    let mut dex_fixture = setup_fixture(env);

    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("taker", dex_fixture.base_resource, 1_000_000.into());
    dex_fixture.assert_resource_amount("taker", dex_fixture.quote_resource, 1_000_000.into());

    dex_fixture.env.acting_as(dex_fixture.user_maker.name);
    let order1_key = dex_fixture.new_limit_order(
        FungibleBucket(10.into(), dex_fixture.quote_resource),
        Decimal::from(1) / Decimal::from(30_000),
    );
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 999_990.into());

    let order2_key = dex_fixture.new_limit_order(
        FungibleBucket(10.into(), dex_fixture.quote_resource),
        Decimal::from(1) / Decimal::from(30_000),);
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 999_980.into());

    let order3_key = dex_fixture.new_limit_order(
        FungibleBucket(10.into(),dex_fixture.quote_resource), Decimal::from(1) / Decimal::from(30_000),);
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 999_970.into());

    dex_fixture.env.acting_as(dex_fixture.user_taker.name);
    dex_fixture.new_market_order(FungibleBucket(600_000.into(), dex_fixture.base_resource));
    dex_fixture.assert_resource_amount("taker", dex_fixture.base_resource, Decimal::from("400000.000000000000000001"),);
    dex_fixture.assert_resource_amount("taker", dex_fixture.quote_resource, Decimal::from("1000019.9999999999998"),);

    dex_fixture.env.acting_as(dex_fixture.user_maker.name);
    dex_fixture.close_limit_order(NonFungibleBucket(order1_key, dex_fixture.order_resource));
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, Decimal::from("1300000.000000003"),);
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, 999_970.into());

    dex_fixture.close_limit_order(NonFungibleBucket(order2_key, dex_fixture.order_resource));
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, Decimal::from("1599999.999999999999999999"),);
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, Decimal::from("999970.0000000000002").into(),);

    dex_fixture.close_limit_order(NonFungibleBucket(order3_key, dex_fixture.order_resource));
    dex_fixture.assert_resource_amount("maker", dex_fixture.base_resource, Decimal::from("1599999.999999999999999999"),);
    dex_fixture.assert_resource_amount("maker", dex_fixture.quote_resource, Decimal::from("999980.0000000000002"),);
}

fn setup_fixture<L: SubstateStore>(mut env: TestEnv<L>) -> DexFixture<L> {
    let maker_name = "maker";
    let taker_name = "taker";
    let _admin = env.create_user("admin");
    let maker = env.create_user(maker_name);
    let taker = env.create_user(taker_name);

    env.acting_as("admin");
    let resource_rusd = env.create_token(2_000_000.into());
    for user in vec![taker, maker] {
        let receipt = env.transfer_resource(1_000_000.into(), &resource_rusd, &user);
        assert!(receipt.result.is_ok());
    }

    env.publish_package(PACKAGE, include_code!("order_book_dex"));
    env.using_package(PACKAGE);

    let receipt = env.call_function(BLUEPRINT_DEX, "instantiate", vec![]);
    assert!(receipt.result.is_ok());
    let dex_component = receipt.component(0).unwrap();
    let admin_badge = receipt.resource_def(0).unwrap();

    let base_resource = RADIX_TOKEN;
    let quote_resource = resource_rusd.address();
    let receipt = env.call_method(
        &dex_component,
        "add_trading_pair",
        vec![
            format!("{}", base_resource),
            format!("{}", quote_resource),
            format!("1,{}", admin_badge),
        ],
    );
    assert!(receipt.result.is_ok());
    let trading_pair_component = receipt.component(0).unwrap();
    let order_resource = receipt.resource_def(1).unwrap();

    DexFixture {
        env,
        _dex_component: dex_component,
        trading_pair_component,
        base_resource,
        quote_resource,
        order_resource,
        user_maker: DexUser {
            account: maker.account,
            name: maker_name,
        },
        user_taker: DexUser {
            account: taker.account,
            name: taker_name,
        },
    }
}

struct DexUser {
    account: Address,
    name: &'static str,
}

struct DexFixture<'a, L: SubstateStore> {
    env: TestEnv<'a, L>,
    _dex_component: Address,
    trading_pair_component: Address,
    base_resource: Address,
    quote_resource: Address,
    order_resource: Address,
    user_maker: DexUser,
    user_taker: DexUser,
}

impl<'a, L: SubstateStore> DexFixture<'a, L> {
    fn new_limit_order(&mut self, funds: FungibleBucket, price: Decimal) -> NonFungibleKey {
        let order_keys_before = self.env.get_non_fungible_keys_for_rd(
            self.env.current_user.unwrap().account,
            self.order_resource,
        );
        let order_keys_before: HashSet<NonFungibleKey> =
            order_keys_before.iter().cloned().collect();

        let receipt = self.env.call_method(
            &self.trading_pair_component,
            "new_limit_order",
            vec![funds.to_string(), price.to_string()],
        );
        assert!(receipt.result.is_ok());

        let order_keys_now = self.env.get_non_fungible_keys_for_rd(
            self.env.current_user.unwrap().account,
            self.order_resource,
        );
        let order_keys_now: HashSet<NonFungibleKey> = order_keys_now.iter().cloned().collect();
        let new_order_keys: HashSet<NonFungibleKey> = order_keys_now
            .difference(&order_keys_before)
            .cloned()
            .collect();
        assert_eq!(new_order_keys.len(), 1);

        new_order_keys.into_iter().next().unwrap()
    }

    fn close_limit_order(&mut self, order: NonFungibleBucket) {
        let receipt = self.env.call_method(
            &self.trading_pair_component,
            "close_limit_order",
            vec![order.to_string()],
        );

        println!("{:?}", receipt);
        assert!(receipt.result.is_ok());
    }

    fn new_market_order(&mut self, funds: FungibleBucket) {
        let receipt = self.env.call_method(
            &self.trading_pair_component,
            "new_market_order",
            vec![funds.to_string()],
        );
        println!("{:?}", receipt);
        assert!(receipt.result.is_ok());
    }

    fn assert_resource_amount(&mut self, user: &str, resource: Address, expected_amount: Decimal) {
        let account = if user == self.user_maker.name {
            self.user_maker.account
        } else if user == self.user_taker.name {
            self.user_taker.account
        } else {
            panic!("Invalid user:  {}", user);
        };

        let actual_amount = self.env.get_amount_for_rd(account, resource);
        assert_eq!(
            expected_amount, actual_amount,
            "Unexpected amount for resource {}: expected={}, actual={}",
            resource, expected_amount, actual_amount
        );
    }
}

struct FungibleBucket(Decimal, Address);

impl FungibleBucket {
    fn to_string(&self) -> String {
        format!("{},{}", self.0, self.1)
    }
}

struct NonFungibleBucket(NonFungibleKey, Address);

impl NonFungibleBucket {
    fn to_string(&self) -> String {
        format!("#{},{}", self.0, self.1)
    }
}
