use scrypto::prelude::*;
use sbor::*;
use std::cmp;

#[derive(TypeId, Decode, Encode, Describe, NftData)]
struct UbiStatus {
    person: Address,
    #[scrypto(mutable)]
    expiration_epoch: u64,
    #[scrypto(mutable)]
    last_mint_epoch: u64
}

blueprint! {

    struct UBI {
        ubi_token: Vault,
        super_badge: Vault,
        admin_badge: Address,
        person_badge: ResourceDef,
        ubi_nft_ids: LazyMap<Address, u128>,
        epochs_until_expiration: u64
    }

    impl UBI {
        pub fn new() -> (Component, Bucket) {
            let super_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "UBI super badge")
                .metadata("description", "Used by the UBI component to mint and transfer tokens.")
                .initial_supply_fungible(1);
            let admin_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "UBI admin badge")
                .metadata("description", "Authority to register accounts as people.")
                .initial_supply_fungible(1);
            let person_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", "Person NFT")
                .metadata("description", "NFT granted to every verified account.")
                .flags(MINTABLE | INDIVIDUAL_METADATA_MUTABLE)
                .badge(super_badge.resource_address(), MAY_MINT | MAY_TRANSFER | MAY_CHANGE_INDIVIDUAL_METADATA)
                .no_initial_supply();
            let token_bucket: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "UBI Token")
                .metadata("description", "One token per epoch per person, 20% burn for each transaction.")
                .metadata("symbol", "UBI")
                .flags(MINTABLE | FREELY_BURNABLE)
                .badge(super_badge.resource_address(), MAY_MINT | MAY_TRANSFER)
                .initial_supply_fungible(0);
            info!("new: ubi token address: {}", token_bucket.resource_address());
            info!("new: admin badge address: {} (given to caller, used to register accounts as people)", admin_badge.resource_address());
            info!("new: person badge address: {} (used to claim UBI)", person_badge.address());

            (Self {
                ubi_token: Vault::with_bucket(token_bucket),
                super_badge: Vault::with_bucket(super_badge),
                admin_badge: admin_badge.resource_address(),
                person_badge,
                ubi_nft_ids: LazyMap::new(),
                epochs_until_expiration : 40 * 365 // Average epoch is around 35 minutes, so this would be about a year.
            }
            .instantiate(), admin_badge)
        }

        #[auth(admin_badge)]
        pub fn register(&mut self, person: Address) {
            let new_expiration_epoch = Context::current_epoch() + self.epochs_until_expiration;
            let result: Option<u128> = self.ubi_nft_ids.get(&person);
            match result {
                Some(nft_id) => {
                    let mut status : UbiStatus = self.person_badge.get_nft_data(nft_id);
                    status.expiration_epoch = new_expiration_epoch;
                    self.super_badge.authorize(
                        |auth| self.person_badge.update_nft_data(nft_id, status, auth)
                    );
                    info!("register: updated expiration {}, nft id: {}", person, nft_id);
                }
                None => {
                    let nft_id = Uuid::generate();
                    let status = UbiStatus{person, expiration_epoch: new_expiration_epoch, last_mint_epoch: Context::current_epoch()};
                    self.ubi_nft_ids.insert(person, nft_id);
                    let super_badge = self.super_badge.take_all();
                    let person_badge = self.person_badge.mint_nft(nft_id, status, super_badge.present());
                    self.super_badge.put(super_badge);
                    Account::from(person).deposit(person_badge);
                    info!("register: created badge for {}, nft id: {}", person, nft_id);
                }
            }
        }

        pub fn available_tokens(&mut self, person_badge: BucketRef) -> u64 {
            let nft_id = person_badge.get_nft_id();
            person_badge.drop();
            let status : UbiStatus = self.person_badge.get_nft_data(nft_id);
            let available = (cmp::min(status.expiration_epoch, Context::current_epoch())) - status.last_mint_epoch;
            info!("available_tokens: {}", available);
            available
        }

        #[auth(person_badge, keep_auth)]
        pub fn collect_ubi(&mut self) -> Bucket {
            let nft_id = auth.get_nft_id();
            auth.drop();
            let mut status : UbiStatus = self.person_badge.get_nft_data(nft_id);
            let new_ubi = Context::current_epoch() - status.last_mint_epoch;
            status.last_mint_epoch = Context::current_epoch();
            let super_badge = self.super_badge.take_all();
            self.person_badge.update_nft_data(nft_id, status, super_badge.present());
            let new_tokens = self.ubi_token.resource_def().mint(new_ubi, super_badge.present());
            self.super_badge.put(super_badge);
            info!("collect_ubi: {}", new_tokens.amount());
            new_tokens
        }

        // Once RECALLABLE is implemented, disable transfer and force burning with updated version of this.
        pub fn send_tokens(&mut self, to: Address, tokens: Bucket) {
            let to_burn = (tokens.amount() * 20) / 100;
            tokens.take(to_burn).burn();
            info!("send_tokens: {} sent, burnt: {}", tokens.amount(), to_burn);
            Account::from(to).deposit(tokens);
        }
    }
}
