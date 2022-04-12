use scrypto::prelude::*;
use sbor::*;
use std::cmp;

// Metadata associated with the badge that is assigned to a registered account.
#[derive(TypeId, Decode, Encode, Describe, NonFungibleData)]
struct UbiStatus {
    #[scrypto(mutable)]
    expiration_epoch: u64,
    #[scrypto(mutable)]
    last_mint_epoch: u64
}

blueprint! {

    struct UBI {
        ubi_token: ResourceDef,
        super_badge: Vault,
        admin_badge: Address,
        person_badge: ResourceDef,
        epochs_until_expiration: u64
    }

    impl UBI {
        pub fn new() -> (Component, Bucket) {

            // The super badge is held by the component and used to mint and transfer UBI tokens.
            let super_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "UBI super badge")
                .metadata("description", "Used by the UBI component to mint and transfer UBI tokens.")
                .initial_supply_fungible(1);

            // The admin badge is for the administrator who has the authority to grant UBI to other accounts.
            let admin_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "UBI admin badge")
                .metadata("description", "Authority to register accounts as people.")
                .initial_supply_fungible(1);

            // The person badge is given to anyone who is registered for UBI. It tracks when they last collected
            // UBI and the expiration epoch of the registration using NFT metadata.
            let person_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", "Person NFT")
                .metadata("description", "NFT granted to every verified account.")
                .flags(MINTABLE | INDIVIDUAL_METADATA_MUTABLE)
                .badge(super_badge.resource_address(), MAY_MINT | MAY_TRANSFER | MAY_CHANGE_INDIVIDUAL_METADATA)
                .no_initial_supply();

            // The main UBI token.
            let ubi_token = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "UBI Token")
                .metadata("description", "One token per epoch per person, 20% burn for each transaction.")
                .metadata("symbol", "UBI")
                .flags(MINTABLE | FREELY_BURNABLE)
                .badge(super_badge.resource_address(), MAY_MINT | MAY_TRANSFER)
                .no_initial_supply();
            info!("new: ubi token address: {}", ubi_token.address());
            info!("new: admin badge address: {} (given to caller, used to register accounts as people)", admin_badge.resource_address());
            info!("new: person badge address: {} (used to claim UBI)", person_badge.address());

            // Return the new component and the admin_badge so caller has authority to register accounts.
            (Self {
                ubi_token,
                super_badge: Vault::with_bucket(super_badge),
                admin_badge: admin_badge.resource_address(),
                person_badge,
                epochs_until_expiration : 40 * 365 // Average epoch is around 35 minutes, so this would be about a year.
            }
            .instantiate(), admin_badge)
        }

        #[auth(admin_badge)]
        pub fn register(&mut self, person: Address) {

            // Create a badge for the person that will start minting this epoch.
            let new_expiration_epoch = Context::current_epoch() + self.epochs_until_expiration;
            let nft_id = Uuid::generate();
            let non_fungible_key = NonFungibleKey::from(nft_id);
            let status = UbiStatus{expiration_epoch: new_expiration_epoch, last_mint_epoch: Context::current_epoch()};
            let person_badge = self.super_badge.authorize({
                |auth| self.person_badge.mint_non_fungible(&non_fungible_key, status, auth)
            });

            // Send the badge to the account.
            Component::from(person).call::<()>("deposit", vec![scrypto_encode(&person_badge)]);
            info!("register: created badge for {}, nft id: {}", person, nft_id);
        }

        #[auth(admin_badge)]
        pub fn update_expiration(&mut self, nft_id: u128) {
            // Converting the u128 nft_id to a NonFungibleKey
            let non_fungible_key = NonFungibleKey::from(nft_id);

            // Find the status data for the person.
            let mut status : UbiStatus = self.person_badge.get_non_fungible_data(&non_fungible_key);

            // Update the expiration epoch in their badge.
            let new_expiration_epoch = Context::current_epoch() + self.epochs_until_expiration;
            status.expiration_epoch = new_expiration_epoch;
            self.super_badge.authorize(
                |auth| self.person_badge.update_non_fungible_data(&non_fungible_key, status, auth)
            );
            info!("update_expiration: updated expiration for nft id: {}", nft_id);
        }

        #[auth(person_badge)]
        pub fn available_tokens(&mut self) -> u64 {
            // Converting the u128 nft_id to a NonFungibleKey
            let non_fungible_key = auth.get_non_fungible_key();

            // Find the status data for the person.
            let status : UbiStatus = self.person_badge.get_non_fungible_data(&non_fungible_key);

            // Compute the available tokens based on the expiration epoch and the current epoch.
            let available = (cmp::min(status.expiration_epoch, Context::current_epoch())) - status.last_mint_epoch;
            info!("available_tokens: {}", available);
            available
        }

        #[auth(person_badge)]
        pub fn collect_ubi(&mut self) -> Bucket {
            // Find the status data for the caller.
            let nft_id = auth.get_non_fungible_key();
            let non_fungible_key = NonFungibleKey::from(nft_id);
            let mut status : UbiStatus = self.person_badge.get_non_fungible_data(&non_fungible_key);

            // Mint the UBI tokens to give to the caller.
            let new_ubi = (cmp::min(status.expiration_epoch, Context::current_epoch())) - status.last_mint_epoch;
            let new_tokens = self.super_badge.authorize({
                |auth| self.ubi_token.mint(new_ubi, auth)
            });

            // Record the epoch of this UBI collection in the status data.
            status.last_mint_epoch = Context::current_epoch();
            self.super_badge.authorize({
                |auth| self.person_badge.update_non_fungible_data(&non_fungible_key, status, auth)
            });
            info!("collect_ubi: {}", new_tokens.amount());
            new_tokens
        }

        // TODO: Once RECALLABLE is implemented, disable transfer and force burning with updated version of this.
        pub fn send_tokens(&mut self, to: Address, mut tokens: Bucket) {

            // Burn 20% of the tokens being transfered.
            let to_burn = (tokens.amount() * dec!("20")) / dec!("100");
            tokens.take(to_burn).burn();

            // Deposit the remaining tokens in the recipient's account.
            info!("send_tokens: {} sent, burnt: {}", tokens.amount(), to_burn);
            Component::from(to).call::<()>("deposit", vec![scrypto_encode(&tokens)]);
        }
    }
}
