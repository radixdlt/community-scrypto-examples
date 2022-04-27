// TODO: Update the tests

use radix_engine::ledger::*;
use radix_engine::model::Receipt;
use radix_engine::transaction::*;
use scrypto::prelude::*;
use scrypto::types::Decimal;

#[test]
fn try_register_as_seller_must_be_succeeded() {
    // Set up environment.

    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN, Decimal::one(), Decimal::one());
    assert_eq!(
        test_env
            .get_balance(test_env.admin_account, RADIX_TOKEN)
            .unwrap(),
        Decimal::from("1000000")
    );

    // register as seller
    let (seller_key, seller_address) = test_env.new_account();
    let receipt = test_env.register_as_seller(seller_key, seller_address);
    assert!(receipt.result.is_ok());
}

#[test]
fn try_list_product_with_sufficient_fees_must_be_succeeded() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN, Decimal::one(), Decimal::one());
    assert_eq!(
        test_env
            .get_balance(test_env.admin_account, RADIX_TOKEN)
            .unwrap(),
        Decimal::from(1_000_000)
    );

    // register as seller
    let (seller_key, seller_address) = test_env.new_account();
    let receipt = test_env.register_as_seller(seller_key, seller_address);
    assert!(receipt.result.is_ok());

    let price: Decimal = Decimal::from(10);
    let fees: Decimal = Decimal::one();
    let product_name = format!("iphone 12");
    //list a product
    let list_product_receipt = test_env.list_product(
        seller_key,
        seller_address,
        product_name,
        price,
        fees,
        RADIX_TOKEN,
    );
    assert!(list_product_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(seller_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000) - 1
    );
}

#[test]
fn try_list_product_with_insufficient_fees_must_be_failed() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN, Decimal::one(), Decimal::one());
    assert_eq!(
        test_env
            .get_balance(test_env.admin_account, RADIX_TOKEN)
            .unwrap(),
        Decimal::from(1_000_000)
    );

    // register as seller
    let (seller_key, seller_address) = test_env.new_account();
    let receipt = test_env.register_as_seller(seller_key, seller_address);
    assert!(receipt.result.is_ok());

    let price: Decimal = Decimal::from(10);
    let fees: Decimal = Decimal::from_str("0.5").unwrap();
    let product_name = format!("iphone 12");
    //list a product
    let list_product_receipt = test_env.list_product(
        seller_key,
        seller_address,
        product_name,
        price,
        fees,
        RADIX_TOKEN,
    );
    assert!(!list_product_receipt.result.is_ok());
    let log_message = &list_product_receipt.logs.get(0).unwrap().1;
    assert!(log_message.starts_with("Panicked at 'the fees must be >= 1'"));
    assert_eq!(
        test_env.get_balance(seller_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000)
    );
}

#[test]
fn try_get_available_products_must_be_succeeded() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN, Decimal::one(), Decimal::one());
    assert_eq!(
        test_env
            .get_balance(test_env.admin_account, RADIX_TOKEN)
            .unwrap(),
        Decimal::from(1_000_000)
    );

    // register as seller
    let (seller_key, seller_address) = test_env.new_account();
    let receipt = test_env.register_as_seller(seller_key, seller_address);
    assert!(receipt.result.is_ok());

    let price: Decimal = Decimal::from(10);
    let fees: Decimal = Decimal::one();
    let product_name = format!("iphone 12");
    //list a product
    let list_product_receipt = test_env.list_product(
        seller_key,
        seller_address,
        product_name,
        price,
        fees,
        RADIX_TOKEN,
    );
    assert!(list_product_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(seller_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000) - 1
    );

    let product_name_2 = format!("iphone 13");
    test_env.list_product(
        seller_key,
        seller_address,
        product_name_2,
        price,
        fees,
        RADIX_TOKEN,
    );

    //get_available_products
    let (user_key, user_address) = test_env.new_account();
    let get_available_products_receipt = test_env.get_available_products(0, user_key, user_address);
    assert!(get_available_products_receipt.result.is_ok());
    let log_message = &get_available_products_receipt.logs.get(0).unwrap().1;
    println!("{:?}\n", log_message);
    assert!(log_message.starts_with("products : "));
    assert!(log_message.contains(";"));
}

#[test]
fn test_get_available_products_pagination() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN, Decimal::one(), Decimal::one());
    assert_eq!(
        test_env
            .get_balance(test_env.admin_account, RADIX_TOKEN)
            .unwrap(),
        Decimal::from(1_000_000)
    );

    // register as seller
    let (seller_key, seller_address) = test_env.new_account();
    let receipt = test_env.register_as_seller(seller_key, seller_address);
    assert!(receipt.result.is_ok());

    let price: Decimal = Decimal::from(10);
    let fees: Decimal = Decimal::one();

    //list a product
    for elem in 0..200 {
        let product_name = format!("iphone 12 - {}", elem);
        test_env.list_product(
            seller_key,
            seller_address,
            product_name,
            price,
            fees,
            RADIX_TOKEN,
        );
    }

    //get_available_products
    let (user_key, user_address) = test_env.new_account();

    let get_available_products_receipt = test_env.get_available_products(0, user_key, user_address);
    assert!(get_available_products_receipt.result.is_ok());
    let log_message = &get_available_products_receipt.logs.get(0).unwrap().1;
    println!("{:?}\n", log_message);
    assert!(log_message.contains(";"));
    let split = log_message.split(';').collect::<Vec<&str>>();
    assert!(split.len() == 100);

    let get_available_products_receipt_1 =
        test_env.get_available_products(1, user_key, user_address);
    assert!(get_available_products_receipt_1.result.is_ok());
    let log_message_1 = &get_available_products_receipt_1.logs.get(0).unwrap().1;
    println!("{:?}\n", log_message_1);
    assert!(log_message_1.contains(";"));
    let split_1 = log_message_1.split(';').collect::<Vec<&str>>();
    assert!(split_1.len() == 100);

    let get_available_products_receipt_2 =
        test_env.get_available_products(2, user_key, user_address);
    assert!(get_available_products_receipt_2.result.is_ok());
    let log_message_2 = &get_available_products_receipt_2.logs.get(0).unwrap().1;
    println!("{:?}\n", log_message_2);
    assert!(!log_message_2.contains(";"));
}

#[test]
fn try_buy_product_with_sufficient_amount_must_be_succeeded() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN, Decimal::one(), Decimal::one());
    assert_eq!(
        test_env
            .get_balance(test_env.admin_account, RADIX_TOKEN)
            .unwrap(),
        Decimal::from(1_000_000)
    );

    // register as seller
    let (seller_key, seller_address) = test_env.new_account();
    let receipt = test_env.register_as_seller(seller_key, seller_address);
    assert!(receipt.result.is_ok());

    let price: Decimal = Decimal::from(10);
    let fees: Decimal = Decimal::one();
    let product_name = format!("iphone 12");

    //seller list a product
    let list_product_receipt = test_env.list_product(
        seller_key,
        seller_address,
        product_name,
        price,
        fees,
        RADIX_TOKEN,
    );
    assert!(list_product_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(seller_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000) - 1
    );

    let product_name_2 = format!("iphone 13");
    test_env.list_product(
        seller_key,
        seller_address,
        product_name_2,
        price,
        fees,
        RADIX_TOKEN,
    );

    //user get_available_products
    let (user_key, user_address) = test_env.new_account();
    let get_available_products_receipt = test_env.get_available_products(0, user_key, user_address);
    assert!(get_available_products_receipt.result.is_ok());
    let log_message = &get_available_products_receipt.logs.get(0).unwrap().1;
    println!("{:?}\n", log_message);
    assert!(log_message.starts_with("products : "));
    assert!(log_message.contains(";"));

    //user Buy Product

    let split_logs: Vec<&str> = log_message.trim().split(':').collect();
    let split_products: Vec<&str> = split_logs[1].split(';').collect();
    let mut products: Vec<(String, String, Decimal)> = Vec::new();

    for elem in split_products {
        let item_tab: Vec<&str> = elem.split('|').collect();
        products.push((
            item_tab.get(0).unwrap().trim().to_string(),
            item_tab.get(1).unwrap().trim().to_string(),
            Decimal::from_str(item_tab.get(2).unwrap()).unwrap(),
        ));
    }

    let payment = products[0].2 + Decimal::one();
    let buy_receipt = test_env.buy_product(
        products[0].0.to_string(),
        format!("Abidjan"),
        format!("1 rue de yopougon siporex"),
        format!("1196"),
        user_key,
        user_address,
        payment,
        RADIX_TOKEN,
    );

    assert!(buy_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(user_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000) - payment
    );
}

#[test]
fn try_buy_product_with_insufficient_amount_must_be_failed() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN, Decimal::one(), Decimal::one());
    assert_eq!(
        test_env
            .get_balance(test_env.admin_account, RADIX_TOKEN)
            .unwrap(),
        Decimal::from(1_000_000)
    );

    // register as seller
    let (seller_key, seller_address) = test_env.new_account();
    let receipt = test_env.register_as_seller(seller_key, seller_address);
    assert!(receipt.result.is_ok());

    let price: Decimal = Decimal::from(10);
    let fees: Decimal = Decimal::one();
    let product_name = format!("iphone 12");

    //seller list a product
    let list_product_receipt = test_env.list_product(
        seller_key,
        seller_address,
        product_name,
        price,
        fees,
        RADIX_TOKEN,
    );
    assert!(list_product_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(seller_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000) - 1
    );

    let product_name_2 = format!("iphone 13");
    test_env.list_product(
        seller_key,
        seller_address,
        product_name_2,
        price,
        fees,
        RADIX_TOKEN,
    );

    //user get_available_products
    let (user_key, user_address) = test_env.new_account();
    let get_available_products_receipt = test_env.get_available_products(0, user_key, user_address);
    assert!(get_available_products_receipt.result.is_ok());
    let log_message = &get_available_products_receipt.logs.get(0).unwrap().1;
    println!("{:?}\n", log_message);
    assert!(log_message.starts_with("products : "));
    assert!(log_message.contains(";"));

    //user Buy Product

    let split_logs: Vec<&str> = log_message.trim().split(':').collect();
    let split_products: Vec<&str> = split_logs[1].split(';').collect();
    let mut products: Vec<(String, String, Decimal)> = Vec::new();

    for elem in split_products {
        let item_tab: Vec<&str> = elem.split('|').collect();
        products.push((
            item_tab.get(0).unwrap().trim().to_string(),
            item_tab.get(1).unwrap().trim().to_string(),
            Decimal::from_str(item_tab.get(2).unwrap()).unwrap(),
        ));
    }

    let payment = products[0].2;
    let buy_receipt = test_env.buy_product(
        products[0].0.to_string(),
        format!("Abidjan"),
        format!("1 rue de yopougon siporex"),
        format!("1196"),
        user_key,
        user_address,
        payment,
        RADIX_TOKEN,
    );

    assert!(!buy_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(user_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000)
    );
    let log_message = &buy_receipt.logs.get(0).unwrap().1;
    assert!(
        log_message.starts_with("Panicked at 'payment amount must be greather than or equal 11'")
    );
}

#[test]
fn try_collect_postal_stamp_when_product_has_been_purchased_must_be_succeeded() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN, Decimal::one(), Decimal::one());
    assert_eq!(
        test_env
            .get_balance(test_env.admin_account, RADIX_TOKEN)
            .unwrap(),
        Decimal::from(1_000_000)
    );

    // register as seller
    let (seller_key, seller_address) = test_env.new_account();
    let receipt = test_env.register_as_seller(seller_key, seller_address);
    assert!(receipt.result.is_ok());

    let price: Decimal = Decimal::from(10);
    let fees: Decimal = Decimal::one();
    let product_name = format!("iphone 12");

    //seller list a product
    let list_product_receipt = test_env.list_product(
        seller_key,
        seller_address,
        product_name,
        price,
        fees,
        RADIX_TOKEN,
    );
    assert!(list_product_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(seller_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000) - 1
    );

    //user get_available_products
    let (user_key, user_address) = test_env.new_account();
    let get_available_products_receipt = test_env.get_available_products(0, user_key, user_address);
    assert!(get_available_products_receipt.result.is_ok());
    let log_message = &get_available_products_receipt.logs.get(0).unwrap().1;
    println!("{:?}\n", log_message);
    assert!(log_message.starts_with("products : "));

    //user Buy Product

    let split_logs: Vec<&str> = log_message.trim().split(':').collect();
    let split_products: Vec<&str> = split_logs[1].split(';').collect();
    let mut products: Vec<(String, String, Decimal)> = Vec::new();

    for elem in split_products {
        let item_tab: Vec<&str> = elem.split('|').collect();
        products.push((
            item_tab.get(0).unwrap().trim().to_string(),
            item_tab.get(1).unwrap().trim().to_string(),
            Decimal::from_str(item_tab.get(2).unwrap()).unwrap(),
        ));
    }

    println!("buy product id : {}", products[0].0.to_string());

    let payment = products[0].2 + Decimal::one();
    let buy_receipt = test_env.buy_product(
        products[0].0.to_string(),
        format!("Abidjan"),
        format!("1 rue de yopougon siporex"),
        format!("1196"),
        user_key,
        user_address,
        payment,
        RADIX_TOKEN,
    );

    assert!(buy_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(user_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000) - payment
    );

    //Seller collect the postal stamp for send product to the buyer
    let collect_postal_stamp_receipt = test_env.collect_postal_stamp(seller_key, seller_address);
    assert!(collect_postal_stamp_receipt.result.is_ok());
}

#[test]
fn try_collect_postal_stamp_one_more_must_be_failed() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN, Decimal::one(), Decimal::one());
    assert_eq!(
        test_env
            .get_balance(test_env.admin_account, RADIX_TOKEN)
            .unwrap(),
        Decimal::from(1_000_000)
    );

    // register as seller
    let (seller_key, seller_address) = test_env.new_account();
    let receipt = test_env.register_as_seller(seller_key, seller_address);
    assert!(receipt.result.is_ok());

    let price: Decimal = Decimal::from(10);
    let fees: Decimal = Decimal::one();
    let product_name = format!("iphone 12");

    //seller list a product
    let list_product_receipt = test_env.list_product(
        seller_key,
        seller_address,
        product_name,
        price,
        fees,
        RADIX_TOKEN,
    );
    assert!(list_product_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(seller_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000) - 1
    );

    //user get_available_products
    let (user_key, user_address) = test_env.new_account();
    let get_available_products_receipt = test_env.get_available_products(0, user_key, user_address);
    assert!(get_available_products_receipt.result.is_ok());
    let log_message = &get_available_products_receipt.logs.get(0).unwrap().1;
    println!("{:?}\n", log_message);
    assert!(log_message.starts_with("products : "));

    //user Buy Product
    let split_logs: Vec<&str> = log_message.trim().split(':').collect();
    let split_products: Vec<&str> = split_logs[1].split(';').collect();
    let mut products: Vec<(String, String, Decimal)> = Vec::new();

    for elem in split_products {
        let item_tab: Vec<&str> = elem.split('|').collect();
        products.push((
            item_tab.get(0).unwrap().trim().to_string(),
            item_tab.get(1).unwrap().trim().to_string(),
            Decimal::from_str(item_tab.get(2).unwrap()).unwrap(),
        ));
    }

    println!("buy product id : {}", products[0].0.to_string());

    let payment = products[0].2 + Decimal::one();
    let buy_receipt = test_env.buy_product(
        products[0].0.to_string(),
        format!("Abidjan"),
        format!("1 rue de yopougon siporex"),
        format!("1196"),
        user_key,
        user_address,
        payment,
        RADIX_TOKEN,
    );

    assert!(buy_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(user_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000) - payment
    );

    //Seller collect the postal stamp for send product to the buyer
    let collect_postal_stamp_receipt = test_env.collect_postal_stamp(seller_key, seller_address);
    assert!(collect_postal_stamp_receipt.result.is_ok());

    let collect_postal_stamp_receipt_2 = test_env.collect_postal_stamp(seller_key, seller_address);
    assert!(!collect_postal_stamp_receipt_2.result.is_ok());

    let log_message = &collect_postal_stamp_receipt_2.logs.get(0).unwrap().1;
    assert!(log_message.starts_with("Panicked at 'postale stamp has already collected'"));
}

#[test]
fn try_collect_postal_stamp_for_product_not_purchase_must_be_failed() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN, Decimal::one(), Decimal::one());
    assert_eq!(
        test_env
            .get_balance(test_env.admin_account, RADIX_TOKEN)
            .unwrap(),
        Decimal::from(1_000_000)
    );

    // register as seller
    let (seller_key, seller_address) = test_env.new_account();
    let receipt = test_env.register_as_seller(seller_key, seller_address);
    assert!(receipt.result.is_ok());

    let price: Decimal = Decimal::from(10);
    let fees: Decimal = Decimal::one();
    let product_name = format!("iphone 12");

    //seller list a product
    let list_product_receipt = test_env.list_product(
        seller_key,
        seller_address,
        product_name,
        price,
        fees,
        RADIX_TOKEN,
    );
    assert!(list_product_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(seller_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000) - 1
    );

    //Seller collect the postal stamp
    let collect_postal_stamp_receipt = test_env.collect_postal_stamp(seller_key, seller_address);
    assert!(!collect_postal_stamp_receipt.result.is_ok());
    let log_message = &collect_postal_stamp_receipt.logs.get(0).unwrap().1;
    assert!(log_message.starts_with("Panicked at 'product must be purchased'"));
}

#[test]
fn try_send_purchase_product_must_be_succeded() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN, Decimal::one(), Decimal::one());
    assert_eq!(
        test_env
            .get_balance(test_env.admin_account, RADIX_TOKEN)
            .unwrap(),
        Decimal::from(1_000_000)
    );

    // register as seller
    let (seller_key, seller_address) = test_env.new_account();
    let receipt = test_env.register_as_seller(seller_key, seller_address);
    assert!(receipt.result.is_ok());

    let price: Decimal = Decimal::from(10);
    let fees: Decimal = Decimal::one();
    let product_name = format!("iphone 12");

    //seller list a product
    let list_product_receipt = test_env.list_product(
        seller_key,
        seller_address,
        product_name,
        price,
        fees,
        RADIX_TOKEN,
    );
    assert!(list_product_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(seller_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000) - 1
    );

    //user get_available_products
    let (user_key, user_address) = test_env.new_account();
    let get_available_products_receipt = test_env.get_available_products(0, user_key, user_address);
    assert!(get_available_products_receipt.result.is_ok());
    let log_message = &get_available_products_receipt.logs.get(0).unwrap().1;
    println!("{:?}\n", log_message);
    assert!(log_message.starts_with("products : "));

    //user Buy Product

    let split_logs: Vec<&str> = log_message.trim().split(':').collect();
    let split_products: Vec<&str> = split_logs[1].split(';').collect();
    let mut products: Vec<(String, String, Decimal)> = Vec::new();

    for elem in split_products {
        let item_tab: Vec<&str> = elem.split('|').collect();
        products.push((
            item_tab.get(0).unwrap().trim().to_string(),
            item_tab.get(1).unwrap().trim().to_string(),
            Decimal::from_str(item_tab.get(2).unwrap()).unwrap(),
        ));
    }

    println!("buy product id : {}", products[0].0.to_string());

    let payment = products[0].2 + Decimal::one();
    let buy_receipt = test_env.buy_product(
        products[0].0.to_string(),
        format!("Abidjan"),
        format!("1 rue de yopougon siporex"),
        format!("1196"),
        user_key,
        user_address,
        payment,
        RADIX_TOKEN,
    );

    assert!(buy_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(user_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000) - payment
    );

    //Seller collect the postal stamp to ship product to the buyer
    let collect_postal_stamp_receipt = test_env.collect_postal_stamp(seller_key, seller_address);
    assert!(collect_postal_stamp_receipt.result.is_ok());

    // Seller confirm the shippment of the product
    let send_product_receipt = test_env.send_product(seller_key, seller_address);
    assert!(send_product_receipt.result.is_ok());
}

#[test]
fn try_confirm_product_reception_must_be_succeded() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN, Decimal::one(), Decimal::one());
    assert_eq!(
        test_env
            .get_balance(test_env.admin_account, RADIX_TOKEN)
            .unwrap(),
        Decimal::from(1_000_000)
    );

    // register as seller
    let (seller_key, seller_address) = test_env.new_account();
    let receipt = test_env.register_as_seller(seller_key, seller_address);
    assert!(receipt.result.is_ok());

    let price: Decimal = Decimal::from(10);
    let fees: Decimal = Decimal::one();
    let product_name = format!("iphone 12");

    //seller list a product
    let list_product_receipt = test_env.list_product(
        seller_key,
        seller_address,
        product_name,
        price,
        fees,
        RADIX_TOKEN,
    );
    assert!(list_product_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(seller_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000) - 1
    );

    //user get_available_products
    let (user_key, user_address) = test_env.new_account();
    let get_available_products_receipt = test_env.get_available_products(0, user_key, user_address);
    assert!(get_available_products_receipt.result.is_ok());
    let log_message = &get_available_products_receipt.logs.get(0).unwrap().1;
    println!("{:?}\n", log_message);
    assert!(log_message.starts_with("products : "));

    //user Buy Product

    let split_logs: Vec<&str> = log_message.trim().split(':').collect();
    let split_products: Vec<&str> = split_logs[1].split(';').collect();
    let mut products: Vec<(String, String, Decimal)> = Vec::new();

    for elem in split_products {
        let item_tab: Vec<&str> = elem.split('|').collect();
        products.push((
            item_tab.get(0).unwrap().trim().to_string(),
            item_tab.get(1).unwrap().trim().to_string(),
            Decimal::from_str(item_tab.get(2).unwrap()).unwrap(),
        ));
    }

    println!("buy product id : {}", products[0].0.to_string());

    let payment = products[0].2 + Decimal::one();
    let buy_receipt = test_env.buy_product(
        products[0].0.to_string(),
        format!("Abidjan"),
        format!("1 rue de yopougon siporex"),
        format!("1196"),
        user_key,
        user_address,
        payment,
        RADIX_TOKEN,
    );

    assert!(buy_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(user_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000) - payment
    );

    //Seller collect the postal stamp to ship product to the buyer
    let collect_postal_stamp_receipt = test_env.collect_postal_stamp(seller_key, seller_address);
    assert!(collect_postal_stamp_receipt.result.is_ok());

    // Seller confirm the shippment of the product
    let send_product_receipt = test_env.send_product(seller_key, seller_address);
    assert!(send_product_receipt.result.is_ok());

    // buyer confirm product reception
    let nft_ids = test_env
        .get_nft_ids(user_address, test_env.seller_buyer_badge)
        .unwrap();
    let nft_id = nft_ids.get(0).unwrap();
    let confirm_reception_receipt =
        test_env.confirm_reception(user_key, user_address, nft_id.clone());
    assert!(confirm_reception_receipt.result.is_ok());
    let nft_ids_after_confirm = test_env
        .get_nft_ids(user_address, test_env.seller_buyer_badge)
        .unwrap();
    // check if Buyer nft is burn
    assert!(!nft_ids_after_confirm.contains(nft_id));
}

#[test]
fn try_collect_by_seller_must_be_succeeded() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN, Decimal::one(), Decimal::one());
    assert_eq!(
        test_env
            .get_balance(test_env.admin_account, RADIX_TOKEN)
            .unwrap(),
        Decimal::from(1_000_000)
    );

    // register as seller
    let (seller_key, seller_address) = test_env.new_account();
    let receipt = test_env.register_as_seller(seller_key, seller_address);
    assert!(receipt.result.is_ok());

    let price: Decimal = Decimal::from(10);
    let fees: Decimal = Decimal::one();
    let product_name = format!("iphone 12");

    //seller list a product
    let list_product_receipt = test_env.list_product(
        seller_key,
        seller_address,
        product_name,
        price,
        fees,
        RADIX_TOKEN,
    );
    assert!(list_product_receipt.result.is_ok());
    let seller_balance = Decimal::from(1_000_000) - 1;
    assert_eq!(
        test_env.get_balance(seller_address, RADIX_TOKEN).unwrap(),
        seller_balance
    );

    //user get_available_products
    let (user_key, user_address) = test_env.new_account();
    let get_available_products_receipt = test_env.get_available_products(0, user_key, user_address);
    assert!(get_available_products_receipt.result.is_ok());
    let log_message = &get_available_products_receipt.logs.get(0).unwrap().1;
    println!("{:?}\n", log_message);
    assert!(log_message.starts_with("products : "));

    //user Buy Product

    let split_logs: Vec<&str> = log_message.trim().split(':').collect();
    let split_products: Vec<&str> = split_logs[1].split(';').collect();
    let mut products: Vec<(String, String, Decimal)> = Vec::new();

    for elem in split_products {
        let item_tab: Vec<&str> = elem.split('|').collect();
        products.push((
            item_tab.get(0).unwrap().trim().to_string(),
            item_tab.get(1).unwrap().trim().to_string(),
            Decimal::from_str(item_tab.get(2).unwrap()).unwrap(),
        ));
    }

    println!("buy product id : {}", products[0].0.to_string());

    let payment = products[0].2 + Decimal::one();
    let buy_receipt = test_env.buy_product(
        products[0].0.to_string(),
        format!("Abidjan"),
        format!("1 rue de yopougon siporex"),
        format!("1196"),
        user_key,
        user_address,
        payment,
        RADIX_TOKEN,
    );

    assert!(buy_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(user_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000) - payment
    );

    //Seller collect the postal stamp to ship product to the buyer
    let collect_postal_stamp_receipt = test_env.collect_postal_stamp(seller_key, seller_address);
    assert!(collect_postal_stamp_receipt.result.is_ok());

    // Seller confirm the shippment of the product
    let send_product_receipt = test_env.send_product(seller_key, seller_address);
    assert!(send_product_receipt.result.is_ok());

    // buyer confirm product reception
    let nft_ids = test_env
        .get_nft_ids(user_address, test_env.seller_buyer_badge)
        .unwrap();
    let nft_id = nft_ids.get(0).unwrap();
    let confirm_reception_receipt =
        test_env.confirm_reception(user_key, user_address, nft_id.clone());
    assert!(confirm_reception_receipt.result.is_ok());
    let nft_ids_after_confirm = test_env
        .get_nft_ids(user_address, test_env.seller_buyer_badge)
        .unwrap();
    // check if Buyer nft is burn
    assert!(!nft_ids_after_confirm.contains(nft_id));

    // collect by seller
    test_env.collect_by_seller(seller_key, seller_address);
    assert_eq!(
        test_env.get_balance(seller_address, RADIX_TOKEN).unwrap(),
        seller_balance + products[0].2
    );
}

#[test]
fn try_collect_by_admin_must_be_succeeded() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger, RADIX_TOKEN, Decimal::one(), Decimal::one());
    assert_eq!(
        test_env
            .get_balance(test_env.admin_account, RADIX_TOKEN)
            .unwrap(),
        Decimal::from(1_000_000)
    );

    // register as seller
    let (seller_key, seller_address) = test_env.new_account();
    let receipt = test_env.register_as_seller(seller_key, seller_address);
    assert!(receipt.result.is_ok());

    let price: Decimal = Decimal::from(10);
    let fees: Decimal = Decimal::one();
    let product_name = format!("iphone 12");

    //seller list a product
    let list_product_receipt = test_env.list_product(
        seller_key,
        seller_address,
        product_name,
        price,
        fees,
        RADIX_TOKEN,
    );
    assert!(list_product_receipt.result.is_ok());
    let seller_balance = Decimal::from(1_000_000) - 1;
    assert_eq!(
        test_env.get_balance(seller_address, RADIX_TOKEN).unwrap(),
        seller_balance
    );

    //user get_available_products
    let (user_key, user_address) = test_env.new_account();
    let get_available_products_receipt = test_env.get_available_products(0, user_key, user_address);
    assert!(get_available_products_receipt.result.is_ok());
    let log_message = &get_available_products_receipt.logs.get(0).unwrap().1;
    println!("{:?}\n", log_message);
    assert!(log_message.starts_with("products : "));

    //user Buy Product

    let split_logs: Vec<&str> = log_message.trim().split(':').collect();
    let split_products: Vec<&str> = split_logs[1].split(';').collect();
    let mut products: Vec<(String, String, Decimal)> = Vec::new();

    for elem in split_products {
        let item_tab: Vec<&str> = elem.split('|').collect();
        products.push((
            item_tab.get(0).unwrap().trim().to_string(),
            item_tab.get(1).unwrap().trim().to_string(),
            Decimal::from_str(item_tab.get(2).unwrap()).unwrap(),
        ));
    }

    println!("buy product id : {}", products[0].0.to_string());

    let payment = products[0].2 + Decimal::one();
    let buy_receipt = test_env.buy_product(
        products[0].0.to_string(),
        format!("Abidjan"),
        format!("1 rue de yopougon siporex"),
        format!("1196"),
        user_key,
        user_address,
        payment,
        RADIX_TOKEN,
    );

    assert!(buy_receipt.result.is_ok());
    assert_eq!(
        test_env.get_balance(user_address, RADIX_TOKEN).unwrap(),
        Decimal::from(1_000_000) - payment
    );

    //Seller collect the postal stamp to ship product to the buyer
    let collect_postal_stamp_receipt = test_env.collect_postal_stamp(seller_key, seller_address);
    assert!(collect_postal_stamp_receipt.result.is_ok());

    // Seller confirm the shippment of the product
    let send_product_receipt = test_env.send_product(seller_key, seller_address);
    assert!(send_product_receipt.result.is_ok());

    // buyer confirm product reception
    let nft_ids = test_env
        .get_nft_ids(user_address, test_env.seller_buyer_badge)
        .unwrap();
    let nft_id = nft_ids.get(0).unwrap();
    let confirm_reception_receipt =
        test_env.confirm_reception(user_key, user_address, nft_id.clone());
    assert!(confirm_reception_receipt.result.is_ok());
    let nft_ids_after_confirm = test_env
        .get_nft_ids(user_address, test_env.seller_buyer_badge)
        .unwrap();
    // check if Buyer nft is burn
    assert!(!nft_ids_after_confirm.contains(nft_id));

    // collect by admin
    test_env.collect_by_admin();
    assert_eq!(
        test_env
            .get_balance(test_env.admin_account, RADIX_TOKEN)
            .unwrap(),
        Decimal::from(1_000_000) + 2
    );
}

struct TestEnv<'a> {
    executor: TransactionExecutor<'a, InMemorySubstateStore>,
    admin_key: EcdsaPublicKey,
    admin_account: Address,
    component: Address,
    admin_badge: Address,
    seller_buyer_badge: Address,
    seller_permanent_badge: Address,
}

impl<'a> TestEnv<'a> {
    pub fn new(
        ledger: &'a mut InMemorySubstateStore,
        token_type: Address,
        buy_fees: Decimal,
        sell_fees: Decimal,
    ) -> Self {
        let mut executor = TransactionExecutor::new(ledger, false);

        let package = executor
            .publish_package(include_code!("product_marketplace"))
            .unwrap();
        let admin_key = executor.new_public_key();
        let admin_account = executor.new_account(admin_key);

        let tx = TransactionBuilder::new(&executor)
            .call_function(
                package,
                "ProductMarketPlace",
                "new",
                vec![
                    token_type.to_string(),
                    format!("{}", sell_fees),
                    format!("{}", buy_fees),
                ],
                Some(admin_account),
            )
            .call_method_with_all_resources(admin_account, "deposit_batch")
            .build(vec![admin_key])
            .unwrap();
        let receipt = executor.run(tx).unwrap();
        println!("{:?}\n", receipt);
        assert!(receipt.result.is_ok());

        let admin_badge = receipt.resource_def(0).unwrap();
        let seller_buyer_badge = receipt.resource_def(3).unwrap();
        let seller_permanent_badge = receipt.resource_def(4).unwrap();

        Self {
            executor,
            admin_key,
            admin_account,
            component: receipt.component(0).unwrap(),
            admin_badge,
            seller_buyer_badge,
            seller_permanent_badge,
        }
    }

    pub fn new_account(&mut self) -> (EcdsaPublicKey, Address) {
        let key = self.executor.new_public_key();
        return (key, self.executor.new_account(key));
    }

    pub fn register_as_seller(
        &mut self,
        seller_key: EcdsaPublicKey,
        seller_address: Address,
    ) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "register_as_seller",
                vec![],
                Some(seller_address),
            )
            .call_method_with_all_resources(seller_address, "deposit_batch")
            .build(vec![seller_key])
            .unwrap();
        let receipt = self.executor.run(tx).unwrap();
        println!("{:?}\n", receipt);
        return receipt;
    }

    fn list_product(
        &mut self,
        seller_key: EcdsaPublicKey,
        seller_address: Address,
        name: String,
        price: Decimal,
        fees: Decimal,
        token_type: Address,
    ) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "list_product",
                vec![
                    name,
                    price.to_string(),
                    format!("{},{}", fees, token_type),
                    format!("1,{}", self.seller_permanent_badge),
                ],
                Some(seller_address),
            )
            .call_method_with_all_resources(seller_address, "deposit_batch")
            .build(vec![seller_key])
            .unwrap();
        let receipt = self.executor.run(tx).unwrap();
        println!("{:?}\n", receipt);
        return receipt;
    }

    fn get_available_products(
        &mut self,
        page_index: u32,
        user_key: EcdsaPublicKey,
        user_address: Address,
    ) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "get_available_products",
                vec![page_index.to_string()],
                Some(user_address),
            )
            .call_method_with_all_resources(user_address, "deposit_batch")
            .build(vec![user_key])
            .unwrap();

        let receipt = self.executor.run(tx).unwrap();
        println!("{:?}\n", receipt);
        return receipt;
    }

    fn buy_product(
        &mut self,
        product_id: String,
        city: String,
        zip_code: String,
        street: String,
        buyer_key: EcdsaPublicKey,
        buyer_address: Address,
        payment: Decimal,
        token_type: Address,
    ) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "buy_product",
                vec![
                    product_id,
                    city,
                    street,
                    zip_code,
                    format!("{},{}", payment, token_type),
                ],
                Some(buyer_address),
            )
            .call_method_with_all_resources(buyer_address, "deposit_batch")
            .build(vec![buyer_key])
            .unwrap();

        let receipt = self.executor.run(tx).unwrap();
        println!("{:?}\n", receipt);
        return receipt;
    }

    fn collect_postal_stamp(
        &mut self,
        seller_key: EcdsaPublicKey,
        seller_address: Address,
    ) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "collect_postal_stamp",
                vec![format!("1,{}", self.seller_buyer_badge)],
                Some(seller_address),
            )
            .call_method_with_all_resources(seller_address, "deposit_batch")
            .build(vec![seller_key])
            .unwrap();

        let receipt = self.executor.run(tx).unwrap();
        println!("{:?}\n", receipt);
        return receipt;
    }

    fn send_product(&mut self, seller_key: EcdsaPublicKey, seller_address: Address) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "send_product",
                vec![format!("1,{}", self.seller_buyer_badge)],
                Some(seller_address),
            )
            .call_method_with_all_resources(seller_address, "deposit_batch")
            .build(vec![seller_key])
            .unwrap();

        let receipt = self.executor.run(tx).unwrap();
        println!("{:?}\n", receipt);
        return receipt;
    }

    fn confirm_reception(
        &mut self,
        buyer_key: EcdsaPublicKey,
        buyer_address: Address,
        nft_id: NonFungibleKey,
    ) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "confirm_reception",
                vec![format!("#{},{}", nft_id, self.seller_buyer_badge)],
                Some(buyer_address),
            )
            .call_method_with_all_resources(buyer_address, "deposit_batch")
            .build(vec![buyer_key])
            .unwrap();

        let receipt = self.executor.run(tx).unwrap();
        println!("{:?}\n", receipt);
        return receipt;
    }

    fn collect_by_seller(
        &mut self,
        seller_key: EcdsaPublicKey,
        seller_address: Address,
    ) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "collect_by_seller",
                vec![format!("1,{}", self.seller_permanent_badge)],
                Some(seller_address),
            )
            .call_method_with_all_resources(seller_address, "deposit_batch")
            .build(vec![seller_key])
            .unwrap();

        let receipt = self.executor.run(tx).unwrap();
        println!("{:?}\n", receipt);
        return receipt;
    }

    fn collect_by_admin(&mut self) -> Receipt {
        let tx = TransactionBuilder::new(&self.executor)
            .call_method(
                self.component,
                "collect_by_admin",
                vec![format!("1,{}", self.admin_badge)],
                Some(self.admin_account),
            )
            .call_method_with_all_resources(self.admin_account, "deposit_batch")
            .build(vec![self.admin_key])
            .unwrap();

        let receipt = self.executor.run(tx).unwrap();
        println!("{:?}\n", receipt);
        return receipt;
    }

    fn get_balance(&self, account: Address, token: Address) -> Result<Decimal, String> {
        let component = self.executor.ledger().get_component(account).unwrap();
        let state = component.state();
        let maps = radix_engine::engine::validate_data(state)
            .unwrap()
            .lazy_maps;
        let resources = self
            .executor
            .ledger()
            .get_lazy_map(&account, &maps[0])
            .unwrap();
        let vault_id_data = resources.get_entry(&scrypto_encode(&token));

        match vault_id_data {
            Some(vault_id_data) => {
                let vault_id = scrypto_decode::<Vid>(&vault_id_data).unwrap();
                let vault = self
                    .executor
                    .ledger()
                    .get_vault(&account, &vault_id)
                    .unwrap();
                Ok(vault.amount())
            }
            None => Err(format!(
                "No vault found for token {} in account {}",
                token, account
            )),
        }
    }

    fn get_nft_ids(
        &self,
        account: Address,
        resource: Address,
    ) -> Result<Vec<NonFungibleKey>, String> {
        let component = self.executor.ledger().get_component(account).unwrap();
        let state = component.state();
        let maps = radix_engine::engine::validate_data(state)
            .unwrap()
            .lazy_maps;
        let resources = self
            .executor
            .ledger()
            .get_lazy_map(&account, &maps[0])
            .unwrap();
        let vault_id_data = resources.get_entry(&scrypto_encode(&resource));

        match vault_id_data {
            Some(vault_id_data) => {
                let vault_id = scrypto_decode::<Vid>(&vault_id_data).unwrap();
                let vault = self
                    .executor
                    .ledger()
                    .get_vault(&account, &vault_id)
                    .unwrap();
                Ok(vault.get_non_fungible_ids().unwrap())
            }
            None => Err(format!(
                "No vault found for token {} in account {}",
                resource, account
            )),
        }
    }
}
