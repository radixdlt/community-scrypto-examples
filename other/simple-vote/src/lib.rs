use scrypto::blueprints::clock::TimePrecision;
use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData, Clone)]
pub struct CurrentVote {
    pub yes_votes: u64,
    pub no_votes: u64,
    pub name: String,
    pub end: Instant,
}

#[derive(ScryptoSbor, NonFungibleData)]
struct Member {
    #[mutable]
    voted_in: HashSet<String>,
}

#[derive(ScryptoSbor, radix_engine_common::ManifestSbor)]
pub enum VoteChoice {
    Yes,
    No,
}

#[blueprint]
mod vote {
    pub struct Vote {
        /// Name of the organization for which this component will be
        /// organizing votes.
        org_name: String,
        current_vote: Option<CurrentVote>,
        member_badge: ResourceAddress,
        admin_badge: ResourceAddress,
        component_badge: Vault,
        pub vote_results: Vault,
    }

    impl Vote {
        /// Returns the component address and a bucket containing the
        /// admin badge for the organization
        pub fn instantiate_vote(org_name: String) -> (ComponentAddress, Bucket) {
            let admin_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", format!("{org_name} Vote Admin"))
                .mint_initial_supply(1);

            let component_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", format!("{org_name} Component"))
                .mint_initial_supply(1);

            let require_admin = rule!(require(admin_badge.resource_address()));
            let require_component = rule!(require(component_badge.resource_address()));

            let org_name_char = org_name.chars().nth(0).expect("The organization name should not be empty.");

            let member_badge: ResourceAddress = ResourceBuilder::new_uuid_non_fungible::<Member>()
                .metadata("name", format!("{org_name} Voting Member"))
                .metadata("symbol", format!("{}VM", org_name_char))
                .mintable(require_admin.clone(), require_admin.clone())
                .burnable(require_admin.clone(), require_admin.clone())
                .recallable(require_admin.clone(), require_admin.clone())
                .restrict_deposit(require_admin.clone(), require_admin.clone())
                .updateable_non_fungible_data(require_component.clone(), require_admin.clone())
                .create_with_no_initial_supply();

            let vote_badge: ResourceAddress =
                ResourceBuilder::new_uuid_non_fungible::<CurrentVote>()
                .metadata("name", format!("{org_name} Vote Result"))
                .metadata("symbol", format!("{}VR", org_name_char))
                .mintable(require_component.clone(), require_admin.clone())
                .create_with_no_initial_supply();

            let access_rules_config = AccessRulesConfig::new()
                .method(
                    "become_member",
                    require_admin.clone(),
                    require_admin.clone(),
                )
                .method("new_vote", require_admin.clone(), require_admin.clone())
                .default(rule!(allow_all), require_admin.clone());

            let component = Self {
                member_badge,
                org_name,
                current_vote: None,
                admin_badge: admin_badge.resource_address(),
                component_badge: Vault::with_bucket(component_badge),
                vote_results: Vault::new(vote_badge),
            }
            .instantiate()
            .globalize_with_access_rules(access_rules_config);

            (component, admin_badge)
        }

        pub fn become_member(&self) -> Bucket {
            borrow_resource_manager!(self.member_badge).mint_uuid_non_fungible(Member {
                voted_in: HashSet::new(),
            })
        }

        /// Duration is given in minutes
        pub fn new_vote(&mut self, name: String, duration: i64) {
            assert!(self.current_vote.is_none(), "Vote is already in progress");
            assert!(duration > 0, "Must have a positive duration");
            let end = Clock::current_time_rounded_to_minutes()
                .add_minutes(duration)
                .expect("Error calculating end time");
            self.current_vote = Some(CurrentVote {
                yes_votes: 0,
                no_votes: 0,
                name,
                end,
            });
        }

        pub fn vote(&mut self, choice: VoteChoice, member_proof: Proof) {
            let current_vote = self.current_vote.as_mut().expect("No vote in progress");
            assert!(
                Clock::current_time_is_strictly_before(current_vote.end, TimePrecision::Minute),
                "Vote has expired"
            );
            let validated_member = member_proof
                .validate_proof(self.member_badge)
                .expect("Invalid member proof provided")
                .non_fungible();
            let mut member_data: Member = validated_member.data();
            assert!(
                !member_data.voted_in.contains(&current_vote.name),
                "Member has already voted"
            );
            member_data.voted_in.insert(current_vote.name.clone());
            self.component_badge.authorize(|| {
                borrow_resource_manager!(self.member_badge).update_non_fungible_data(
                    &validated_member.local_id(),
                    "voted_in",
                    member_data.voted_in,
                );
            });
            match choice {
                VoteChoice::Yes => current_vote.yes_votes += 1,
                VoteChoice::No => current_vote.no_votes += 1,
            };
        }

        pub fn end_vote(&mut self) {
            let current_vote = self.current_vote.as_ref().expect("No vote in progress");
            assert!(
                Clock::current_time_is_at_or_after(current_vote.end, TimePrecision::Minute),
                "Vote has not ended"
            );
            let vote_result = self.component_badge.authorize(|| {
                borrow_resource_manager!(self.vote_results.resource_address())
                    .mint_uuid_non_fungible(current_vote.clone())
            });
            self.current_vote = None;
            self.vote_results.put(vote_result);
        }
    }
}
