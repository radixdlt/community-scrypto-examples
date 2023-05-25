use scrypto::prelude::*;
use scrypto_unit::*;

pub struct TestLib {
    pub test_runner: TestRunner,
}

impl TestLib {
    pub fn new() -> TestLib {
        Self { test_runner: TestRunner::builder().build() }
    }

    pub fn expect_string_metadata(&mut self, addr: &ResourceAddress, key: &str) -> Option<String> {
        let name = self.test_runner
            .get_metadata(Address::Resource(*addr), key)?;
        use scrypto::api::node_modules::metadata::{MetadataEntry, MetadataValue};
        match name {
            MetadataEntry::Value(MetadataValue::String(n)) => Some(n),
            _ => panic!("Resource {} metadata is of wrong type", key),
        }
    }
}
