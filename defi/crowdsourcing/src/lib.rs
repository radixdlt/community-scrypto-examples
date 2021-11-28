use scrypto::prelude::*;

blueprint! {
    struct CrowdsourcingCampaign {
        // Collected XRD for the crowdsourcing campaign.
        collected_xrd: Vault,
        // Definition of fundraisers badge.
        fundraiser_badge_def: ResourceDef,
        // Used to mint mutable badges.
        patron_mint_badge: Vault,
        // The amount that has to be collected for the contract to be fullfilled.
        goal: Decimal,
        // Last epoch of the campaign.
        last_epoch: u64,
        // Address book of all patron badges, and the amount they pledged.
        patron_entries: HashMap<Address, Decimal>
    }

    impl CrowdsourcingCampaign {
        /* 
        Setup the campaign contract, assign goal of XRD required and
        epoch duration for the campaign to be finished.
        */
        pub fn new(goal: Decimal, campaign_duration_epochs: u64) -> (Component, Bucket) {
            // Create a badge for the fundraiser of the crowdsourcing campaign.
            let fundraiser_badge = ResourceBuilder::new().metadata("name", "fundraiser_badge").new_badge_fixed(1);

            // Get the number of the last epoch.
            let last_epoch = Context::current_epoch() + campaign_duration_epochs;

            // Patron badge is used to mint and burn patron badges.
            let patron_mint_badge = ResourceBuilder::new().metadata("name", "patron_mint_badge").new_badge_fixed(1);

            // Instantiate the CrowdsourcingCampaign component.
            let component = Self {
                collected_xrd: Vault::new(RADIX_TOKEN),
                fundraiser_badge_def: fundraiser_badge.resource_def(),
                patron_mint_badge: Vault::with_bucket(patron_mint_badge),
                goal: goal,
                last_epoch: last_epoch,
                patron_entries: HashMap::new()
            }
            .instantiate();

            // Return fundraiser_badge to initiator of the campaign.
            (component, fundraiser_badge)
        }

        /* 
        Get status of the campaign.
        */
        pub fn status(&self) {
            let mut pledged = Decimal::zero();
            for (_, value) in self.patron_entries.iter() {
                pledged = pledged + *value;
            }

            info!("{} XRD collected from {} patrons", pledged, self.patron_entries.keys().len());

            if !(Context::current_epoch() > self.last_epoch) {
                info!("campaign ends in {} epochs", self.last_epoch - Context::current_epoch());
            } else {
                info!("campaign has ended. campaign holds {} unclaimed XRD.", self.collected_xrd.amount());
            }
        }

        /*
        Pledge XRD and become a patron.
        */
        pub fn pledge(&mut self, payment: Bucket) -> Bucket {
            assert!(payment.amount() != Decimal::zero(), "you need to pay at least one XRD to become a patron.");
            assert!(Context::current_epoch() < self.last_epoch, "campaign has already ended.");

            // Create a burnable badge.
            let patron_badge_resource = ResourceBuilder::new().metadata("name", "patron_badge").new_badge_mutable(self.patron_mint_badge.resource_def());
            let patron_badge = self.patron_mint_badge.authorize(|badge| patron_badge_resource.mint(1, badge));

            // Add badge and value to patron entries for this pledge.
            self.patron_entries.insert(patron_badge.resource_address(), payment.amount());

            // Put payment in collected XRD.
            self.collected_xrd.put(payment);

            // Return badge
            patron_badge
        }

        /*
        Recall pledge as a patron. It is allowed as long as goal hasn't been reached and last_epoch hasn't been passed.
         */
        pub fn recall_pledge(&mut self, patron_badge: Bucket) -> Bucket {
            info!("current epoch {}, goal epoch {}, current collected {}, goal collected {}", Context::current_epoch(), self.last_epoch, self.collected_xrd.amount(), self.goal);
            assert!(!(Context::current_epoch() > self.last_epoch && self.collected_xrd.amount() > self.goal), "campaign was successful and has ended.");
            
            let refund = Bucket::new(RADIX_TOKEN);
            match self.patron_entries.get(&patron_badge.resource_address()) {
                Some(&value) => {
                    // Put XRD into refund bucket.
                    refund.put(self.collected_xrd.take(value));
                    // Remove patron entry.
                    self.patron_entries.remove(&patron_badge.resource_address());
                    // Authorize to burn patron badge.
                    self.patron_mint_badge.authorize(|mint_badge| patron_badge.burn(mint_badge));
                }
                _ => scrypto_abort("no pledge found with provided badge")
            }

            refund
        }

        /*
        As fundraiser, withdraw collected XRD if goal has passed, and the last_epoch has passed.
        */
        #[auth(fundraiser_badge_def)]
        pub fn withdraw(&self) -> Bucket {
            assert!(Context::current_epoch() > self.last_epoch, "campaign has not ended yet.");
            assert!(self.collected_xrd.amount() > self.goal, "campaign did not reach it's goal.");

            self.collected_xrd.take_all()
        }
    }
}
