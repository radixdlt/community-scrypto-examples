## Scrypto Unit: A lightweight testing framework for Scrypto
Contribute to plymth/scrypto-unit development by creating an account on GitHub.
[Scrypto Unit Github](https://github.com/plymth/scrypto-unit)

##3 Getting Started
Add scrypto-unit as a depenency in your `Cargo.toml`
```toml
[dependencies]
scrypto = { git = "https://github.com/radixdlt/radixdlt-scrypto", tag = "v0.3.0" }
```
Import scrypto-unit into your `tests/lib.rs` integration test
```rust
use scrypto_unit::*;
```
Create users for your test environment
```rust
#[test]
fn test_create_user() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut test_env = TestEnv::new(&mut ledger);

    test_env.create_user("alice");
    test_env.create_user("bob");
    test_env.create_user("carol");

    assert!(test_env.users.contains_key("alice"));
    assert!(test_env.users.contains_key("bob"));
    assert!(test_env.users.contains_key("carol"));
    assert_eq!(test_env.users.len(), 3);
}
```

### Resources
- [Unescape: Rust helper](https://lib.rs/install/unescape)