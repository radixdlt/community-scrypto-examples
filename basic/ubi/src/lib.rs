use scrypto::prelude::*;
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
        ubi_token: ResourceAddress,
        super_badge: Vault,
        admin_badge: ResourceAddress,
        person_badge: ResourceAddress,
        epochs_until_expiration: u64
    }

    impl UBI {
        pub fn new() -> (ComponentAddress, Bucket) {

            // The super badge is held by the component and used to mint and transfer UBI tokens.
            let super_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "UBI super badge")
                .metadata("description", "Used by the UBI component to mint and transfer UBI tokens.")
                .initial_supply(1);

            // The admin badge is for the administrator who has the authority to grant UBI to other accounts.
            let admin_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "UBI admin badge")
                .metadata("description", "Authority to register accounts as people.")
                .initial_supply(1);

            // The person badge is given to anyone who is registered for UBI. It tracks when they last collected
            // UBI and the expiration epoch of the registration using NFT metadata.
            let person_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", "Person NFT")
                .metadata("description", "NFT granted to every verified account.")
                .mintable(rule!(require(super_badge.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(super_badge.resource_address())), LOCKED)
                .no_initial_supply();

            // The main UBI token.
            let ubi_token = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "UBI Token")
                .metadata("description", "One token per epoch per person, 20% burn for each transaction.")
                .metadata("symbol", "UBI")
                .mintable(rule!(require(super_badge.resource_address())), LOCKED)
                .burnable(rule!(allow_all), LOCKED)
                .no_initial_supply();
            info!("new: ubi token address: {}", ubi_token);
            info!("new: admin badge address: {} (given to caller, used to register accounts as people)", admin_badge.resource_address());
            info!("new: person badge address: {} (used to claim UBI)", person_badge);

            // Creating the access rules which will govern the component
            let access_check = AccessRules::new()
                .method("register", rule!(require(admin_badge.resource_address())))
                .default(rule!(allow_all));

            // Return the new component and the admin_badge so caller has authority to register accounts.
            (
                Self {
                    ubi_token,
                    super_badge: Vault::with_bucket(super_badge),
                    admin_badge: admin_badge.resource_address(),
                    person_badge,
                    epochs_until_expiration: 40 * 365 // Average epoch is around 35 minutes, so this would be about a year.
                }
                .instantiate()
                .add_access_check(access_check)
                .globalize(), 
                admin_badge
            )
        }

        pub fn register(&mut self, person: ComponentAddress) {

            // Create a badge for the person that will start minting this epoch.
            let new_expiration_epoch = Runtime::current_epoch() + self.epochs_until_expiration;
            let non_fungible_id = NonFungibleId::random();
            let status = UbiStatus{expiration_epoch: new_expiration_epoch, last_mint_epoch: Runtime::current_epoch()};
            let person_badge = self.super_badge.authorize({
                || borrow_resource_manager!(self.person_badge).mint_non_fungible(&non_fungible_id, status)
            });

            // Send the badge to the account.
            borrow_component!(person).call::<()>("deposit", vec![scrypto_encode(&person_badge)]);
            info!("register: created badge for {}, nft id: {}", person, non_fungible_id);
        }

        pub fn update_expiration(&mut self, nft_id: NonFungibleId, auth: Proof) {
            assert_eq!(auth.resource_address(), self.admin_badge, "Invalid Proof provided");
            assert_eq!(auth.amount(), Decimal::one(), "Invalid Proof amount");

            // Find the status data for the person.
            let mut status: UbiStatus = borrow_resource_manager!(self.person_badge)
                .get_non_fungible_data(&nft_id);

            // Update the expiration epoch in their badge.
            let new_expiration_epoch = Runtime::current_epoch() + self.epochs_until_expiration;
            status.expiration_epoch = new_expiration_epoch;
            self.super_badge.authorize(
                || borrow_resource_manager!(self.person_badge).update_non_fungible_data(&nft_id, status)
            );
            info!("update_expiration: updated expiration for nft id: {}", nft_id);
        }

        pub fn available_tokens(&mut self, auth: Proof) -> u64 {
            assert_eq!(auth.resource_address(), self.person_badge, "Invalid Proof provided");
            assert_eq!(auth.amount(), Decimal::one(), "Invalid Proof amount");

            // Converting the u64 nft_id to a NonFungibleId
            let non_fungible_id = auth.non_fungible::<UbiStatus>().id();

            // Find the status data for the person.
            let status: UbiStatus = borrow_resource_manager!(self.person_badge).get_non_fungible_data(&non_fungible_id);

            // Compute the available tokens based on the expiration epoch and the current epoch.
            let available = (cmp::min(status.expiration_epoch, Runtime::current_epoch())) - status.last_mint_epoch;
            info!("available_tokens: {}", available);
            available
        }

        pub fn collect_ubi(&mut self, auth: Proof) -> Bucket {
            assert_eq!(auth.resource_address(), self.person_badge, "Invalid Proof provided");
            assert_eq!(auth.amount(), Decimal::one(), "Invalid Proof amount");

            // Find the status data for the caller.
            let nft_id = auth.non_fungible::<UbiStatus>().id();
            let non_fungible_id = NonFungibleId::from(nft_id);
            let mut status: UbiStatus = borrow_resource_manager!(self.person_badge).get_non_fungible_data(&non_fungible_id);

            // Mint the UBI tokens to give to the caller.
            let new_ubi = (cmp::min(status.expiration_epoch, Runtime::current_epoch())) - status.last_mint_epoch;
            let new_tokens = self.super_badge.authorize({
                || borrow_resource_manager!(self.ubi_token).mint(new_ubi)
            });

            // Record the epoch of this UBI collection in the status data.
            status.last_mint_epoch = Runtime::current_epoch();
            self.super_badge.authorize({
                || borrow_resource_manager!(self.person_badge).update_non_fungible_data(&non_fungible_id, status)
            });
            info!("collect_ubi: {}", new_tokens.amount());
            new_tokens
        }

        // TODO: Once RECALLABLE is implemented, disable transfer and force burning with updated version of this.
        pub fn send_tokens(&mut self, to: ComponentAddress, mut tokens: Bucket) {

            // Burn 20% of the tokens being transferred.
            let to_burn = (tokens.amount() * dec!("20")) / dec!("100");
            info!("UBI Token RA: {}", self.ubi_token);
            info!("Sent Token RA: {}", tokens.resource_address());
            tokens.take(to_burn).burn();

            // Deposit the remaining tokens in the recipient's account.
            info!("send_tokens: {} sent, burnt: {}", tokens.amount(), to_burn);
            borrow_component!(to).call::<()>("deposit", vec![scrypto_encode(&tokens)]);
        }
    }
}
