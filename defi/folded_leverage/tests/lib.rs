use scrypto::crypto::{EcdsaPrivateKey, EcdsaPublicKey};
use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

use FoldedLeverage::User;

struct TestEnv<'a, L: SubstateStore> {
    executor: TransactionExecutor<'a, L>,
    key: EcdsaPublicKey,
    priv_key: EcdsaPrivateKey,
    account: ComponentAddress,
    usd: ResourceAddress,
    lending_pool: ComponentAddress,
}

fn set_up_test_env<'a, L: SubstateStore>(ledger: &'a mut L) -> TestEnv<'a, L> {
    let mut executor = TransactionExecutor::new(ledger, false);
    let (key, priv_key, account) = executor.new_account();
    let package = executor.publish_package(compile_package!()).unwrap();

    let receipt = executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .new_token_fixed(HashMap::new(), dec!("1000000"))
                .call_method_with_all_resources(account, "deposit_batch")
                .build(executor.get_nonce([key]))
                .sign([&priv_key]),
        )
        .unwrap();
    let usd = receipt.new_resource_addresses[0];

    let receipt = executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .call_function(
                    package,
                    "AutoLend",
                    "instantiate_autolend",
                    vec![scrypto_encode(&usd), scrypto_encode("USD")],
                )
                .call_method_with_all_resources(account, "deposit_batch")
                .build(executor.get_nonce([key]))
                .sign([&priv_key]),
        )
        .unwrap();
    let lending_pool = receipt.new_component_addresses[0];

    TestEnv {
        executor,
        key,
        priv_key,
        account,
        usd,
        lending_pool,
    }
}

fn create_user<'a, L: SubstateStore>(env: &mut TestEnv<'a, L>) -> ResourceAddress {
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .call_method(env.lending_pool, "new_user", args![])
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.key]))
                .sign([&env.priv_key]),
        )
        .unwrap();
    assert!(receipt.result.is_ok());
    receipt.new_resource_addresses[0]
}

fn get_user_state<'a, L: SubstateStore>(env: &mut TestEnv<'a, L>, user_id: ResourceAddress) -> User {
    let mut receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .call_method(
                    env.lending_pool,
                    "get_user",
                    vec![scrypto_encode(&user_id)],
                )
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.key]))
                .sign([&env.priv_key]),
        )
        .unwrap();
    assert!(receipt.result.is_ok());
    let encoded = receipt.outputs.swap_remove(0).raw;
    scrypto_decode(&encoded).unwrap()
}

#[test]
fn test_deposit_and_redeem() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = set_up_test_env(&mut ledger);

    let user_id = create_user(&mut env);

    // First, deposit 100 USD
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .withdraw_from_account_by_amount(dec!("1"), user_id, env.account)
                .withdraw_from_account_by_amount(dec!("100"), env.usd, env.account)
                .take_from_worktop(user_id, |builder, user_id_bucket_id| {
                    builder.create_proof_from_bucket(user_id_bucket_id, |builder, user_id_proof_id| {
                        builder.take_from_worktop(env.usd, |builder, usd_bucket_id| {
                            builder.call_method(
                                env.lending_pool,
                                "deposit",
                                vec![
                                    scrypto_encode(&scrypto::resource::Proof(user_id_proof_id)),
                                    scrypto_encode(&scrypto::resource::Bucket(usd_bucket_id)),
                                ],
                            )
                        })
                    })
                })
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.key]))
                .sign([&env.priv_key]),
        )
        .unwrap();
    println!("{:?}", receipt);
    let user_state = get_user_state(&mut env, user_id);
    assert_eq!(
        user_state,
        User {
            deposit_balance: "100".parse().unwrap(),
            deposit_interest_rate: "0.01".parse().unwrap(),
            deposit_last_update: 0,
            borrow_balance: "0".parse().unwrap(),
            borrow_interest_rate: "0".parse().unwrap(),
            borrow_last_update: 0
        }
    );

    // Then, increase deposit interest rate to 5%
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .call_method(
                    env.lending_pool,
                    "set_deposit_interest_rate",
                    vec![scrypto_encode(&dec!("0.05"))],
                )
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.key]))
                .sign([&env.priv_key]),
        )
        .unwrap();
    println!("{:?}", receipt);

    // After that, deposit another 100 USD
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .withdraw_from_account_by_amount(dec!("1"), user_id, env.account)
                .withdraw_from_account_by_amount(dec!("100"), env.usd, env.account)
                .take_from_worktop(user_id, |builder, user_id_bucket_id| {
                    builder.create_proof_from_bucket(user_id_bucket_id, |builder, user_id_proof_id| {
                        builder.take_from_worktop(env.usd, |builder, usd_bucket_id| {
                            builder.call_method(
                                env.lending_pool,
                                "deposit",
                                vec![
                                    scrypto_encode(&scrypto::resource::Proof(user_id_proof_id)),
                                    scrypto_encode(&scrypto::resource::Bucket(usd_bucket_id)),
                                ],
                            )
                        })
                    })
                })
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.key]))
                .sign([&env.priv_key]),
        )
        .unwrap();
    println!("{:?}", receipt);
    let user_state = get_user_state(&mut env, user_id);
    assert_eq!(
        user_state,
        User {
            deposit_balance: "201".parse().unwrap(),
            deposit_interest_rate: "0.02990049751243781".parse().unwrap(),
            deposit_last_update: 0,
            borrow_balance: "0".parse().unwrap(),
            borrow_interest_rate: "0".parse().unwrap(),
            borrow_last_update: 0
        }
    );

    // Finally, redeem with 150 aUSD
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .create_proof_from_account_by_amount(dec!("1"), user_id, env.account)
                .create_proof_from_auth_zone(user_id, |builder, proof_id| {
                    builder.call_method(
                        env.lending_pool,
                        "redeem",
                        vec![
                            scrypto_encode(&scrypto::resource::Proof(proof_id)),
                            scrypto_encode(&dec!("150")),
                        ],
                    )
                })
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.key]))
                .sign([&env.priv_key]),
        )
        .unwrap();
    println!("{:?}", receipt);
    let user_state = get_user_state(&mut env, user_id);
    assert_eq!(
        user_state,
        User {
            deposit_balance: "51".parse().unwrap(),
            deposit_interest_rate: "0.02990049751243781".parse().unwrap(),
            deposit_last_update: 0,
            borrow_balance: "0".parse().unwrap(),
            borrow_interest_rate: "0".parse().unwrap(),
            borrow_last_update: 0
        }
    );
}

#[test]
fn test_borrow_and_repay() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut env = set_up_test_env(&mut ledger);

    let user_id = create_user(&mut env);

    // First, deposit 1000 USD
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .withdraw_from_account_by_amount(dec!("1"), user_id, env.account)
                .withdraw_from_account_by_amount(dec!("1000"), env.usd, env.account)
                .take_from_worktop(user_id, |builder, user_id_bucket_id| {
                    builder.create_proof_from_bucket(user_id_bucket_id, |builder, user_id_proof_id| {
                        builder.take_from_worktop(env.usd, |builder, usd_bucket_id| {
                            builder.call_method(
                                env.lending_pool,
                                "deposit",
                                vec![
                                    scrypto_encode(&scrypto::resource::Proof(user_id_proof_id)),
                                    scrypto_encode(&scrypto::resource::Bucket(usd_bucket_id)),
                                ],
                            )
                        })
                    })
                })
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.key]))
                .sign([&env.priv_key]),
        )
        .unwrap();
    println!("{:?}", receipt);
    let user_state = get_user_state(&mut env, user_id);
    assert_eq!(
        user_state,
        User {
            deposit_balance: "1000".parse().unwrap(),
            deposit_interest_rate: "0.01".parse().unwrap(),
            deposit_last_update: 0,
            borrow_balance: "0".parse().unwrap(),
            borrow_interest_rate: "0".parse().unwrap(),
            borrow_last_update: 0
        }
    );

    // Then, borrow 100 USD
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .create_proof_from_account_by_amount(dec!("1"), user_id, env.account)
                .create_proof_from_auth_zone(user_id, |builder, proof_id| {
                    builder.call_method(
                        env.lending_pool,
                        "borrow",
                        vec![
                            scrypto_encode(&scrypto::resource::Proof(proof_id)),
                            scrypto_encode(&dec!("100")),
                        ],
                    )
                })
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.key]))
                .sign([&env.priv_key]),
        )
        .unwrap();
    println!("{:?}", receipt);
    let user_state = get_user_state(&mut env, user_id);
    assert_eq!(
        user_state,
        User {
            deposit_balance: "1000".parse().unwrap(),
            deposit_interest_rate: "0.01".parse().unwrap(),
            deposit_last_update: 0,
            borrow_balance: "100".parse().unwrap(),
            borrow_interest_rate: "0.02".parse().unwrap(),
            borrow_last_update: 0
        }
    );

    // Then, increase borrow interest rate to 5%
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .call_method(
                    env.lending_pool,
                    "set_borrow_interest_rate",
                    vec![scrypto_encode(&dec!("0.05"))],
                )
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.key]))
                .sign([&env.priv_key]),
        )
        .unwrap();
    println!("{:?}", receipt);

    // After that, borrow another 100 USD
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .create_proof_from_account_by_amount(dec!("1"), user_id, env.account)
                .create_proof_from_auth_zone(user_id, |builder, prood_id| {
                    builder.call_method(
                        env.lending_pool,
                        "borrow",
                        vec![
                            scrypto_encode(&scrypto::resource::Proof(prood_id)),
                            scrypto_encode(&dec!("100")),
                        ],
                    )
                })
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.key]))
                .sign([&env.priv_key]),
        )
        .unwrap();
    println!("{:?}", receipt);
    let user_state = get_user_state(&mut env, user_id);
    assert_eq!(
        user_state,
        User {
            deposit_balance: "1000".parse().unwrap(),
            deposit_interest_rate: "0.01".parse().unwrap(),
            deposit_last_update: 0,
            borrow_balance: "202".parse().unwrap(),
            borrow_interest_rate: "0.034851485148514851".parse().unwrap(),
            borrow_last_update: 0
        }
    );

    // Finally, repay with 150 USD
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .create_proof_from_account_by_amount(dec!("1"), user_id, env.account)
                .withdraw_from_account_by_amount(dec!("150"), env.usd, env.account)
                .create_proof_from_auth_zone(user_id, |builder, user_id_proof_id| {
                    builder.take_from_worktop(env.usd, |builder, usd_bucket_id| {
                        builder.call_method(
                            env.lending_pool,
                            "repay",
                            vec![
                                scrypto_encode(&scrypto::resource::Proof(user_id_proof_id)),
                                scrypto_encode(&scrypto::resource::Bucket(usd_bucket_id)),
                            ],
                        )
                    })
                })
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.key]))
                .sign([&env.priv_key]),
        )
        .unwrap();
    println!("{:?}", receipt);
    let user_state = get_user_state(&mut env, user_id);
    assert_eq!(
        user_state,
        User {
            deposit_balance: "1000".parse().unwrap(),
            deposit_interest_rate: "0.01".parse().unwrap(),
            deposit_last_update: 0,
            borrow_balance: "59.039999999999999902".parse().unwrap(),
            borrow_interest_rate: "0.034851485148514851".parse().unwrap(),
            borrow_last_update: 0
        }
    );

    // F*k it, repay everything
    let receipt = env
        .executor
        .validate_and_execute(
            &TransactionBuilder::new()
                .create_proof_from_account_by_amount(dec!("1"), user_id, env.account)
                .withdraw_from_account_by_amount(dec!("1000"), env.usd, env.account)
                .create_proof_from_auth_zone(user_id, |builder, user_id_proof_id| {
                    builder.take_from_worktop(env.usd, |builder, usd_bucket_id| {
                        builder.call_method(
                            env.lending_pool,
                            "repay",
                            vec![
                                scrypto_encode(&scrypto::resource::Proof(user_id_proof_id)),
                                scrypto_encode(&scrypto::resource::Bucket(usd_bucket_id)),
                            ],
                        )
                    })
                })
                .call_method_with_all_resources(env.account, "deposit_batch")
                .build(env.executor.get_nonce([env.key]))
                .sign([&env.priv_key]),
        )
        .unwrap();
    println!("{:?}", receipt);
    let user_state = get_user_state(&mut env, user_id);
    assert_eq!(
        user_state,
        User {
            deposit_balance: "1000".parse().unwrap(),
            deposit_interest_rate: "0.01".parse().unwrap(),
            deposit_last_update: 0,
            borrow_balance: "0".parse().unwrap(),
            borrow_interest_rate: "0".parse().unwrap(),
            borrow_last_update: 0
        }
    );
}
