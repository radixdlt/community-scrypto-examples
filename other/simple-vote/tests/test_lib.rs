use radix_engine::{
    blueprints::resource::NonFungibleResourceManagerSubstate,
    ledger::ReadableSubstateStore,
    transaction::{CommitResult, TransactionReceipt, TransactionResult},
};
use scrypto::{api::component::ComponentStateSubstate, prelude::*};
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

    pub fn expect_error_log(receipt: &TransactionReceipt, msg: &str) {
        match receipt.result {
            TransactionResult::Commit(CommitResult {
                ref application_logs,
                ..
            }) => {
                assert!(
                    application_logs
                        .iter()
                        .any(|(level, log)| { level == &Level::Error && log.contains(msg) }),
                    "Application logs did not contain expected message '{}', got logs: {:?}",
                    msg,
                    application_logs,
                );
            }
            _ => panic!("Expected transaction commit, but was rejected or aborted"),
        };
    }

    /// Based off https://discord.com/channels/417762285172555786/1046881787906887680/1092013443755810838
    pub fn get_component_state<T: ScryptoDecode>(
        &mut self,
        component_address: &ComponentAddress,
    ) -> T {
        let component_state: ComponentStateSubstate = self
            .test_runner
            .substate_store()
            .get_substate(&SubstateId(
                RENodeId::GlobalObject(Address::Component(*component_address)),
                NodeModuleId::SELF,
                SubstateOffset::Component(ComponentOffset::State0),
            ))
            .map(|s| s.substate.to_runtime().into())
            .unwrap();
        let raw_state = IndexedScryptoValue::from_scrypto_value(component_state.0);
        raw_state.as_typed::<T>().unwrap()
    }

    /// Based off https://discord.com/channels/417762285172555786/1043115427099856916/1091828075559407769
    pub fn get_non_fungible_data<T: NonFungibleData>(
        &mut self,
        resource: &ResourceAddress,
        id: &NonFungibleLocalId,
    ) -> T {
        let store = &self.test_runner.substate_store();
        let resource_manager: Option<NonFungibleResourceManagerSubstate> = store
            .get_substate(&SubstateId(
                RENodeId::GlobalObject((*resource).into()),
                NodeModuleId::SELF,
                SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
            ))
            .map(|s| s.substate.to_runtime().into());

        let ids = resource_manager.unwrap().non_fungible_table;
        let nft: ScryptoValue = store
            .get_substate(&SubstateId(
                RENodeId::KeyValueStore(ids),
                NodeModuleId::SELF,
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(
                    scrypto_encode(&id).unwrap(),
                )),
            ))
            .map(|s| Into::<Option<ScryptoValue>>::into(s.substate.to_runtime()))
            .unwrap()
            .expect("non fungible not found");

        IndexedScryptoValue::from_scrypto_value(nft)
            .as_typed::<T>()
            .unwrap()
    }
}
