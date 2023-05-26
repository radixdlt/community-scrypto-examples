use radix_engine::transaction::{CommitResult, TransactionReceipt, TransactionResult};
use scrypto::prelude::*;
use scrypto_unit::*;

pub struct TestLib {
    pub test_runner: TestRunner,
}

impl TestLib {
    pub fn new() -> TestLib {
        Self {
            test_runner: TestRunner::builder().build(),
        }
    }

    pub fn expect_string_metadata(&mut self, addr: &ResourceAddress, key: &str) -> Option<String> {
        let name = self
            .test_runner
            .get_metadata(Address::Resource(*addr), key)?;
        use scrypto::api::node_modules::metadata::{MetadataEntry, MetadataValue};
        match name {
            MetadataEntry::Value(MetadataValue::String(n)) => Some(n),
            _ => panic!("Resource {} metadata is of wrong type", key),
        }
    }

    pub fn expect_error_log(&mut self, receipt: &TransactionReceipt, msg: &str) {
        match receipt.result {
            TransactionResult::Commit(CommitResult {
                ref application_logs,
                ..
            }) => {
                assert!(
                    application_logs
                        .iter()
                        .any(|(level, log)| { level == &Level::Error && log.contains(msg) }),
                    "Application logs did not contain expected message, got logs: {:?}",
                    application_logs
                );
            }
            _ => panic!("Expected transaction commit, but was rejected or aborted"),
        };
    }
}
