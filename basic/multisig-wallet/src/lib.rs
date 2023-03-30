use scrypto::prelude::*;

#[derive(NonFungibleData)]

struct OwnerBadge {}

#[derive(LegacyDescribe, ScryptoEncode, ScryptoDecode, ScryptoCategorize)]
pub struct Transfer {
    to: ComponentAddress,
    amount: Decimal,
    resource_address: ResourceAddress,
    approved: HashSet<NonFungibleLocalId>,
    executed: bool,
    function_name: String,
}

#[blueprint]
mod multiSigAccount {

    struct MultiSigAccount {
        vaults: KeyValueStore<ResourceAddress, Vault>,
        admin_badge: Vault,
        owner_badges: ResourceAddress,
        approval_threshold: Decimal,
        issued_badges: HashSet<NonFungibleLocalId>,
        transfers: Vec<Transfer>,
    }

    impl MultiSigAccount {
        pub fn instantiate(
            num_owner_badges: u64,
            approval_threshold: Decimal,
        ) -> (ComponentAddress, Bucket) {
            assert_ne!(num_owner_badges, 0, "Must have at least one owner");
            assert!(
                approval_threshold > Decimal::from(0) && approval_threshold <= Decimal::from(1),
                "Approval threshold must be between 0 and 1"
            );

            let mut badge_data: Vec<OwnerBadge> = Vec::new();

            for _i in 0..num_owner_badges {
                badge_data.push(OwnerBadge {});
            }
            let internal_admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Internal Admin Badge")
                .metadata("description", "A badge used to manage the bidder badges")
                .metadata("symbol", "IADMIN")
                .mint_initial_supply(1);

            // Creating the bidder's badge which will be used to track the bidder's information and bids.
            let owner_badges: Bucket = ResourceBuilder::new_uuid_non_fungible()
                .metadata("name", "Owner Badge")
                .metadata(
                    "description",
                    "A badge provided to the owners of this multi-sig account",
                )
                .metadata("symbol", "OWNER")
                .mintable(
                    rule!(require(internal_admin_badge.resource_address())),
                    LOCKED,
                )
                .burnable(
                    rule!(require(internal_admin_badge.resource_address())),
                    LOCKED,
                )
                .updateable_non_fungible_data(
                    rule!(require(internal_admin_badge.resource_address())),
                    LOCKED,
                )
                .mint_initial_supply(badge_data);

            let mut issued_badges: HashSet<NonFungibleLocalId> = HashSet::new();

            for local_id in owner_badges.non_fungible_local_ids() {
                issued_badges.insert(local_id);
            }

            let multi_sig_account: MultiSigAccountComponent = Self {
                vaults: KeyValueStore::new(),
                admin_badge: Vault::with_bucket(internal_admin_badge),
                owner_badges: owner_badges.resource_address(),
                approval_threshold,
                issued_badges,
                transfers: Vec::new(),
            }
            .instantiate();

            let multi_sig_account_address: ComponentAddress = multi_sig_account.globalize();

            (multi_sig_account_address, owner_badges)
        }

        pub fn balance(&self, resource_address: ResourceAddress) -> Decimal {
            self.vaults
                .get(&resource_address)
                .map(|v| v.amount())
                .unwrap_or_default()
        }

        /// Deposits resource into this account.
        pub fn deposit(&mut self, bucket: Bucket) {
            let resource_address = bucket.resource_address();
            if self.vaults.get(&resource_address).is_none() {
                let v = Vault::with_bucket(bucket);
                self.vaults.insert(resource_address, v);
            } else {
                let mut v = self.vaults.get_mut(&resource_address).unwrap();
                v.put(bucket);
            }
        }

        /// Deposit a batch of buckets into this account
        pub fn deposit_batch(&mut self, buckets: Vec<Bucket>) {
            for bucket in buckets {
                self.deposit(bucket);
            }
        }

        pub fn add_transaction(
            &mut self,
            to: ComponentAddress,
            resource_address: ResourceAddress,
            amount: Decimal,
            function_name: String,
        ) {
            self.transfers.push(Transfer {
                to,
                amount,
                resource_address,
                approved: HashSet::new(),
                executed: false,
                function_name,
            });
        }

        pub fn add_transaction_with_proof(
            &mut self,
            to: ComponentAddress,
            resource_address: ResourceAddress,
            amount: Decimal,
            proof: Proof,
            function_name: String,
        ) {
            let validated_proof: ValidatedProof = proof
                .validate_proof(ProofValidationMode::ValidateResourceAddress(
                    self.owner_badges,
                ))
                .expect("[Add transaction with proof]: Invalid proof provided");

            self.add_transaction(to, resource_address, amount, function_name);

            for local_id in validated_proof.non_fungible_local_ids() {
                self._approve_transaction(self.transfers.len() - 1, local_id);
            }
        }

        fn _approve_transaction(&mut self, transaction_index: usize, local_id: NonFungibleLocalId) {
            self.transfers[transaction_index].approved.insert(local_id);
        }

        pub fn approve_transaction(&mut self, transaction_index: usize, proof: Proof) {
            let validated_proof: ValidatedProof = proof
                .validate_proof(ProofValidationMode::ValidateResourceAddress(
                    self.owner_badges,
                ))
                .expect("[Approve transaction]: Invalid proof provided");

            for local_id in validated_proof.non_fungible_local_ids() {
                self._approve_transaction(transaction_index, local_id);
            }
        }

        pub fn execute_transaction(&mut self, transaction_index: usize) {
            assert_eq!(
                self.transfers[transaction_index].executed, false,
                "[Execute transaction] Transaction already executed"
            );
            assert!(
                self.transfers.len() > transaction_index,
                "[Execute transaction] Transaction index out of bounds"
            );

            let transfer: &Transfer = self.transfers.get(transaction_index).unwrap();
            let num_approved: usize = self.transfers[transaction_index].approved.len();

            let num_owners: usize = self.issued_badges.len();

            let approval_percentage: Decimal =
                Decimal::from(num_approved) / Decimal::from(num_owners);

            assert!(
                approval_percentage >= self.approval_threshold,
                "[Execute transaction] Not enough approvals"
            );

            let bucket: Bucket = self
                .vaults
                .get_mut(&transfer.resource_address)
                .unwrap()
                .take(transfer.amount);

            let to_account: &GlobalComponentRef = borrow_component!(transfer.to);

            to_account.call::<()>(transfer.function_name.as_str(), args![bucket]);

            self.transfers[transaction_index].executed = true;
        }

        pub fn disapprove_transaction(&mut self, transaction_index: usize, proof: Proof) {
            let validated_proof: ValidatedProof = proof
                .validate_proof(ProofValidationMode::ValidateResourceAddress(
                    self.owner_badges,
                ))
                .expect("[Disapprove transaction]: Invalid proof provided");

            for local_id in validated_proof.non_fungible_local_ids() {
                self.transfers[transaction_index].approved.remove(&local_id);
            }
        }

        pub fn get_transaction(&self, transaction_index: usize) -> Transfer {
            Transfer {
                to: self.transfers[transaction_index].to,
                amount: self.transfers[transaction_index].amount,
                resource_address: self.transfers[transaction_index].resource_address,
                approved: self.transfers[transaction_index].approved.clone(),
                executed: self.transfers[transaction_index].executed,
                function_name: self.transfers[transaction_index].function_name.clone(),
            }
        }
    }
}
