use scrypto::prelude::*;
use scrypto::blueprints::clock::TimePrecision;
use std::cmp::*;

#[derive(NonFungibleData, ScryptoSbor)]
struct Registration {
    identity_provider_name: String,
    #[mutable]
    last_mint: i64,
    #[mutable]
    expires: i64
}

#[derive(ScryptoSbor)]
pub struct IdentityProvider {
    name: String,
    start_time: i64,
    badge: ResourceAddress,
    provider_id: i64,
    active: bool
}

#[derive(ScryptoSbor)]
pub enum ProviderEventType {
    Add,
    Remove
}

#[derive(ScryptoSbor)]
pub struct ProviderEvent {
    event_type: ProviderEventType,
    active_provider_count: i64,
    event_time: i64,
    provider_id: i64
}

#[blueprint]
mod ubi_module {

    struct UBI {
        ubi_token: ResourceAddress,
        ubi_component_badge: Vault,
        super_badge: ResourceAddress,
        identity_providers: HashMap<String, IdentityProvider>,
        provider_events: Vec<ProviderEvent>
    }

    impl UBI {
        pub fn instantiate_ubi() -> (ComponentAddress, Bucket) {

            // Badge used by the component to mint UBI.
            let ubi_component_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "UBI component internal authority badge")
                .mint_initial_supply(1);

            // Badge for administrative functions like adding identity providers.
            let super_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "UBI component internal authority badge")
                .mint_initial_supply(1);

            // The main UBI token.
            let ubi_token = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "UBI Token")
                .metadata("description", "One thousand tokens per 30 days per person")
                .metadata("symbol", "UBI")
                .mintable(rule!(require(ubi_component_badge.resource_address())), LOCKED)
                .burnable(rule!(allow_all), LOCKED)
                .create_with_no_initial_supply();

            // Set up the access rules.
            let access_rules = AccessRulesConfig::new()
                .method("register", rule!(require(super_badge.resource_address())), AccessRule::DenyAll)
                .method("add_identity_provider", rule!(require(super_badge.resource_address())), AccessRule::DenyAll)
                .method("deactivate_identity_provider", rule!(require(super_badge.resource_address())), AccessRule::DenyAll)
                .default(AccessRule::AllowAll, AccessRule::DenyAll);

            let ubi_component = Self {
                ubi_component_badge: Vault::with_bucket(ubi_component_badge),
                super_badge: super_badge.resource_address(),
                ubi_token,
                identity_providers: HashMap::new(),
                provider_events: Vec::new()
            }.instantiate();
            (ubi_component.globalize_with_access_rules(access_rules), super_badge)
        }

        pub fn add_identity_provider(&mut self, identity_provider_name: String) {
            info!("add_identity_provider [{}]", identity_provider_name);
            assert!(self.identity_providers.get(&identity_provider_name).is_none(), "Provider already exists.");

            // Create the badge for the identity provider.
            let start_time = Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch;
            let badge_nft = ResourceBuilder::new_string_non_fungible::<Registration>()
                .metadata("name", format!("Badge: {}", identity_provider_name))
                .metadata("description", format!("Track unique people for provider: {}", identity_provider_name))
                .mintable(rule!(require(self.super_badge)), LOCKED)
                .recallable(rule!(require(self.super_badge)), LOCKED)
                .updateable_non_fungible_data(rule!(require(self.ubi_component_badge.resource_address())), LOCKED)
                .create_with_no_initial_supply();

            // Add the identity provider.
            let provider_id = self.identity_providers.len() as i64;
            let identity_provider = IdentityProvider {
                name: identity_provider_name.to_string(),
                start_time: start_time,
                badge: badge_nft,
                provider_id,
                active: true
            };
            self.identity_providers.insert(identity_provider_name, identity_provider);

            // Add the event.
            let active_provider_count = if self.provider_events.is_empty() { 1 } else {self.provider_events.last().unwrap().active_provider_count + 1};
            let event = ProviderEvent {
                event_type: ProviderEventType::Add,
                active_provider_count,
                event_time: Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch,
                provider_id
            };
            self.provider_events.push(event);
        }

        pub fn deactivate_identity_provider(&mut self, identity_provider_name: String) {
            info!("deactivate_identity_provider [{}]", identity_provider_name);

            // Find and deactivate the provider.
            let optional_provider: Option<&mut IdentityProvider> = self.identity_providers.get_mut(&identity_provider_name);
            assert!(optional_provider.is_some(), "Provider not found.");
            let provider = optional_provider.unwrap();
            provider.active = false;

            // Add an event to the provider events.
            let active_provider_count = self.provider_events.last().unwrap().active_provider_count - 1;
            let event = ProviderEvent {
                event_type: ProviderEventType::Remove,
                active_provider_count,
                event_time: Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch,
                provider_id: provider.provider_id
            };
            self.provider_events.push(event);
        }

        pub fn register(&mut self, identity_provider_name:String, id: String) -> Bucket {
            info!("register with provider [{}], id [{}]", identity_provider_name, id);

            // Find the provider.
            let provider: Option<&IdentityProvider> = self.identity_providers.get(&identity_provider_name);
            assert!(provider.is_some(), "Provider not found.");
            let provider_resource_manager = borrow_resource_manager!(provider.unwrap().badge);
            let nonfungible_id = NonFungibleLocalId::string(id).unwrap();

            // Mint the identity badge.
            let registration = Registration{
                identity_provider_name,
                last_mint: Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch,
                expires: Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch + 365 * 24 * 60 * 60
            };
            let identity_badge = self.ubi_component_badge.authorize({
                || provider_resource_manager.mint_non_fungible(&nonfungible_id, registration)
            });
            return identity_badge
        }

        pub fn collect_ubi(&mut self, identity_badge: Bucket) -> (Bucket, Bucket) {

            // Find the registration for the caller.
            let binding = identity_badge.non_fungible_local_ids();
            assert!(binding.len() == 1, "Invalid identity badge provided");
            let nft_id = binding.first().unwrap();
            let registration: Registration = borrow_resource_manager!(identity_badge.resource_address()).get_non_fungible_data(&nft_id);

            // Find the provider and make sure the identity badge comes from a valid provider.
            let provider: Option<&IdentityProvider> = self.identity_providers.get(&registration.identity_provider_name);
            assert!(provider.is_some(), "Provider not found.");
            assert_eq!(identity_badge.resource_address(), provider.unwrap().badge, "Invalid identity badge provided");
            assert_eq!(identity_badge.amount(), Decimal::one(), "Invalid identity badge amount");

            // Calculate the UBI based on the number of active providers. Scan backwards through the events
            // to figure out how much UBI to mint during the time intervals.
            let current_time = Clock::current_time(TimePrecision::Minute).seconds_since_unix_epoch;
            let mut accrued_ubi = Decimal::zero();
            let mut event_index = self.provider_events.len() - 1;
            // Skip over events until the provider is active.
            if !provider.unwrap().active {
                while event_index > 0 && self.provider_events[event_index].provider_id != provider.unwrap().provider_id {
                    event_index -= 1;
                }
            }
            let mut active_provider_count = self.provider_events[event_index].active_provider_count;
            let mut last_event_time = self.provider_events[event_index].event_time;
            let mut interval_start_time = max(last_event_time, registration.last_mint);
            let mut interval_end_time = min(current_time, registration.expires);
            while interval_end_time >= interval_start_time {
                // This rate is 1000 UBI every 30 days: 1000.0 / (30.0 * 24.0 * 60.0 * 60.0);
                let ubi_per_second = dec!("0.000385802469135802");
                let accrued_time = Decimal::from(interval_end_time - interval_start_time);
                accrued_ubi += (accrued_time * ubi_per_second) / active_provider_count;
                debug!("collect_ubi: UBI:[{}], Providers:[{}], For:[{}]", accrued_ubi, active_provider_count, provider.unwrap().name);

                // Find the next event.
                if event_index == 0 {
                    break;
                } else {
                    event_index -= 1;
                    active_provider_count = self.provider_events[event_index].active_provider_count;
                    last_event_time = self.provider_events[event_index].event_time;
                    interval_end_time = min(interval_start_time, registration.expires);
                    interval_start_time = max(last_event_time, registration.last_mint);
                }
            }

            // Mint the UBI tokens to give to the caller.
            let new_tokens = self.ubi_component_badge.authorize({
                || borrow_resource_manager!(self.ubi_token).mint(accrued_ubi)
            });

            // Record the time of this UBI collection in the status data.
            self.ubi_component_badge.authorize({
                || borrow_resource_manager!(provider.unwrap().badge).update_non_fungible_data(&nft_id, "last_mint", current_time)
            });
            info!("collect_ubi: [{}]", new_tokens.amount());
            (new_tokens, identity_badge)
        }
    }
}
