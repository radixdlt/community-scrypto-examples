use scrypto::prelude::*;
#[derive(NonFungibleData)]
struct MailNft {
    // The mailbox to recieve message.
    recipient: ComponentAddress,

    // The mailbox sending the message.
    sender: ComponentAddress,

    // Subject of mail.
    subject: String,

    // Main body, could be plain text, encrypted text, a document address etc...
    body: String,

    // A map of attachments, the idea here is that the mapped values would be addresses to downloads such as documents, high-res images etc...
    attachments: HashMap<String, String>, // Perhaps at the given address a cryptographic proof of the NFT is required to download.

    // Any other metadata the sender would like to include, that do not fit into the other fields.
    metadata: HashMap<String, String>
}

#[blueprint]
mod xrdmail {
    struct Mailbox {
        // Inbox for MailNfts
        inbox: KeyValueStore<ResourceAddress, Vault>,
        // Auth badge required for minting stamps and mail.
        auth_badge: Vault,
        // Vault for collecting fees if a stamp price has been set.
        fee_vault: Vault,
        // Stamp price if no arguments for a pricing tier have been provided. Set at 0 upon instantiation.
        default_stamp_price: Decimal,
        // A hashmap for storing pricing tiers.
        pricing_tiers: HashMap<String, (Decimal, HashSet<ResourceAddress>)>,
        // For storing other mailbox metadata.
        metadata: HashMap<String, String>,
        // Resources for methods to refer to.
        mail_rsrc: ResourceAddress,
        stamp_rsrc: ResourceAddress,
        owner_badge_rsrc: ResourceAddress,
        mailbox_address: Option<ComponentAddress>,
    }

    impl Mailbox {
        // Constructor
        pub fn instantiate_mailbox(get_stamp_rule: Option<ResourceAddress>) -> (ComponentAddress, Bucket) {
            info!("Setting up new mailbox!");
            // Get LocalComponent and Owner Badge.
            let (mut mailbox, owner_badge) = Self::instantiate_mailbox_local();

            // Add mandatory access rules.
            let accsrule_owner = rule!(require(owner_badge.resource_address()));
            let access_rules = AccessRules::new()
                .method("collect_fees", accsrule_owner.clone(), LOCKED)
                .method("create_stamp_price_tier", accsrule_owner.clone(), LOCKED)
                .method("set_default_stamp_price", accsrule_owner.clone(), LOCKED)
                .method("delete_stamp_price_tier", accsrule_owner.clone(), LOCKED)
                .method("add_metadata", accsrule_owner.clone(), LOCKED)
                .method("delete_metadata", accsrule_owner.clone(), LOCKED)
                .method("compose", accsrule_owner.clone(), LOCKED)
                .method("withdraw_mail", accsrule_owner.clone(), LOCKED)
                .method("create_mail_proofs", accsrule_owner.clone(), LOCKED)
                .default(AccessRule::AllowAll, LOCKED);
            mailbox.add_access_check(access_rules);

            // Add optional `get_stamp` method access rule.
            if get_stamp_rule.is_some() {
                let get_stamp_access_rule = AccessRules::new()
                    .method("get_stamp", rule!(require(get_stamp_rule.unwrap())), AccessRule::DenyAll);
                mailbox.add_access_check(get_stamp_access_rule);
            }

            mailbox.metadata("Protocol", "XRDmail");

            // Set ComponentAddress
            let globalized_mailbox = mailbox.globalize();
            let global_ref: MailboxGlobalComponentRef = globalized_mailbox.into();
            global_ref.set_component_address(globalized_mailbox);

            // Set owner_badge mailbox metadata and lock.
            let owner_badge_rsrc_manager = borrow_resource_manager!(owner_badge.resource_address());
            let mbx_hex: String = globalized_mailbox.to_hex();
            owner_badge_rsrc_manager.set_metadata("mailbox".to_string(), mbx_hex);
            owner_badge_rsrc_manager.lock_updateable_metadata();

            // Return globalized Mailbox and owner badge.
            (globalized_mailbox, owner_badge)
        }

        // Instantiate mailbox LocalComponent helper method.
        fn instantiate_mailbox_local() -> (MailboxComponent, Bucket) {
            // Create auth badge
            let auth_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);
            // Create owner badge
            let owner_badge = ResourceBuilder::new_fungible()
                .metadata("name", "Mailbox Owner Badge")
                .metadata("description", "Badge for accessing owner methods on an XRDmail protocol mailbox.")
                .metadata("symbol", "XRDMBX")
                .updateable_metadata(AccessRule::AllowAll, AccessRule::AllowAll)
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);
            // Create stamp resource
            let stamp_rsrc = ResourceBuilder::new_fungible()
                .metadata("name", "Stamp")
                .metadata("description", "A stamp for sending mail.")
                .metadata("symbol", "STAMP")
                .mintable(rule!(require(auth_badge.resource_address())), LOCKED)
                // Make so only mailbox `post` method can burn a stamp.
                .burnable(rule!(require(auth_badge.resource_address())), LOCKED)
                // Make transient.
                .restrict_deposit(AccessRule::DenyAll, LOCKED)
                .divisibility(DIVISIBILITY_NONE)
                .create_with_no_initial_supply();
            // Create MailNft resource
            let mail_rsrc = ResourceBuilder::new_uuid_non_fungible()
                .metadata("name", "XRD Mail")
                .metadata("description", "A MailNft for the XRDmail protocol")
                .metadata("symbol", "MAIL")
                .mintable(rule!(require(auth_badge.resource_address())), LOCKED)
                // Anyone can burn mail in a container they own.
                .burnable(AccessRule::AllowAll, LOCKED)
                .create_with_no_initial_supply();
            // Return MailboxComponent and Owner Badge
            (Self {
                inbox: KeyValueStore::new(),
                fee_vault: Vault::new(RADIX_TOKEN),
                auth_badge: Vault::with_bucket(auth_badge),
                default_stamp_price: Decimal::zero(),
                pricing_tiers: HashMap::new(),
                metadata: HashMap::new(),
                mail_rsrc,
                stamp_rsrc,
                owner_badge_rsrc: owner_badge.resource_address(),
                mailbox_address: None,
            }
            .instantiate(), owner_badge)
        }

        // Collect stamp fees.
        pub fn collect_fees(&mut self) -> Bucket {
            self.fee_vault.take_all()
        } // ACCESS CONTROL: Owner badge.

        // Create a new stamp price tier.
        pub fn create_stamp_price_tier(&mut self, tier_name: String, tier_value: (Decimal, HashSet<ResourceAddress>)) {
            self.pricing_tiers.insert(tier_name, tier_value);
        } // ACCESS CONTROL: Owner badge.

        // Delete a price tier.
        pub fn delete_stamp_price_tier(&mut self, tier_name: String) {
            self.pricing_tiers.remove(&tier_name).expect("Could not find pricing tier.");
        } // ACCESS CONTROL: Owner badge.

        // Set a default fee for getting a stamp.
        pub fn set_default_stamp_price(&mut self, new_price: Decimal) {
            self.default_stamp_price = new_price;
        } // ACCESS CONTROL: Owner badge.

        // Create a new item for the Mailbox metadata HashMap.
        // For example ("name", "Business Mailbox") &/ ("pgp_public_key", "xxx...")
        pub fn add_metadata(&mut self, key: String, value: String) {
            self.metadata.insert(key, value);
        } // ACCESS CONTROL: Owner badge.

        // Delete a metadata item.
        pub fn delete_metadata(&mut self, key: String) {
            self.metadata.remove(&key).expect("Could not find metadata.");
        } // ACCESS CONTROL: Owner badge.

        // Withdraw mail for use cases.
        pub fn withdraw_mail(&mut self, rsrc: ResourceAddress, id: NonFungibleLocalId) -> Bucket {
            self.inbox.get_mut(&rsrc).unwrap().take_non_fungible(&id)
        } // ACCESS CONTROL: Owner badge.

        // Push proof of specific mail/s to AuthZone for use cases.
        pub fn create_mail_proofs(&self, rsrc: ResourceAddress, ids: BTreeSet<NonFungibleLocalId>) -> Proof {
            self.inbox.get(&rsrc).unwrap().create_proof_by_ids(&ids)
        } // ACCESS CONTROL: Owner badge.

        // Create mail.
        pub fn compose(
            &self, 
            // Parameters required for MailNft construction.
            recipient: ComponentAddress, 
            subject: String, 
            body: String, 
            attachments: HashMap<String, String>, 
            metadata: HashMap<String, String>,
            // Proof of authentic stamp required so it is not possible to mint a MailNft without a valid stamp.
            stamp: Proof, 
            ) -> Bucket {
            info!("Writing mail.");
            let recipient_mailbox_ref: MailboxGlobalComponentRef = recipient.into();
            // Authenticate recipient address.
            assert_eq!(recipient_mailbox_ref.package_address(), self.get_pkg_addr(), "Recipient mailbox not of XRDmail protocol");
            // Authenticate stamp.
            assert_eq!(recipient_mailbox_ref.get_stamp_rsrc(), stamp.resource_address(), "Incorrect stamp for recipient mailbox.");
            // Construct NFdata
            let data = MailNft {
                recipient,
                sender: self.mailbox_address.unwrap(),
                subject,
                body,
                attachments,
                metadata,
            };
            // Mint MailNft
            let mail = self.auth_badge.authorize(|| {borrow_resource_manager!(self.mail_rsrc).mint_uuid_non_fungible(data)});
            // Post MailNft
            mail
        } // ACCESS CONTROL: Owner badge.

        // Get a stamp for this mailbox.
        pub fn get_stamp(&mut self, fee_bucket: Option<Bucket>, price_tier: Option<(String, Vec<Proof>)>) -> (Bucket, Option<Bucket>) {
            info!("Getting stamp for recipient mailbox.");
            // If no stamp fee has been set by Mailbox owner.
            if self.default_stamp_price == Decimal::zero() {
                // Return stamp and change bucket if fee given.
                (self.mint_stamp(), 
                if fee_bucket.is_some() {
                    fee_bucket
                } else {
                    None
                })
            } 
            // If a price tier has been given.
            else if price_tier.is_some() {
                // Unwrap `price_tier` object.
                let (tier_name, given_proofs) = price_tier.unwrap();
                // Get a hashset of given proof resource addresses
                let given_proofs_addrs : HashSet<ResourceAddress> = given_proofs.iter()
                .map(|proof| proof.resource_address())
                .collect();
                // Get price tier.
                let price_tier: (Decimal, HashSet<ResourceAddress>) = self.pricing_tiers.get(&tier_name).cloned().expect("Unable to get price tier.");
                // Get required proofs for the given tier name.
                let required_proofs = price_tier.1;
                // assert `given_proofs` have all `required_proofs`.
                assert!(required_proofs.difference(&given_proofs_addrs).collect::<Vec<_>>().len() == 0, "Missing required proofs.");
                // Get tier price.
                let tier_price = price_tier.0;
                // If 0 fee for this tier.
                if tier_price == Decimal::zero() {
                    // Return stamp and change bucket if fee given.
                    (self.mint_stamp(), 
                    if fee_bucket.is_some() {
                        fee_bucket
                    } else {
                        None
                    })
                } else {
                    (self.mint_stamp(), Some(self.collect_fee(fee_bucket.unwrap(), tier_price)))
                }
            } 
            // If no price tier has been given.
            else if fee_bucket.is_some() {
                // Return Stamp and change.
                (self.mint_stamp(), Some(self.collect_fee(fee_bucket.unwrap(), self.default_stamp_price)))
            }
            // If caller has not given a fee bucket.
            else {
                panic!("A fee is required to get a stamp from this mailbox.");
            }
        }

        pub fn post(&mut self, mail_nft: Bucket, stamp: Bucket, sender: ComponentAddress) {
            info!("Posting mail.");
            // Authenticate stamp.
            assert_eq!(stamp.resource_address(), self.stamp_rsrc, "Incorrect stamp for this mailbox.");
            // Get mail nft data.
            let mail_nft_data: MailNft = mail_nft.non_fungible().data();
            assert_eq!(mail_nft_data.sender, sender, "Sender address on mail NFT and sender address passed as an argument do not match.");
            let sender_mailbox_ref: MailboxGlobalComponentRef = mail_nft_data.sender.into();
            // Authenticate sender mailbox.
            assert_eq!(sender_mailbox_ref.package_address(), self.get_pkg_addr(), "Sender mailbox not of XRDmail protocol");
            let given_mail_rsrc = mail_nft.resource_address();
            // Authenticate MailNft
            assert_eq!(given_mail_rsrc, sender_mailbox_ref.get_mail_rsrc(), "Inauthentic MailNft");
            // Authenticate MailNft data recipient address.
            assert_eq!(mail_nft_data.recipient, self.mailbox_address.unwrap(), "Incorrect recipient address on MailNft for this mailbox.");
            // Put mail in inbox and burn transient stamp resource.
            info!("Putting mail into inbox.");
            let inbox_vault = self.inbox.get_mut(&given_mail_rsrc);
            if inbox_vault.is_some() {
                inbox_vault.unwrap().put(mail_nft);
            } else {
                self.inbox.insert(given_mail_rsrc, Vault::new(given_mail_rsrc));
                self.inbox.get_mut(&given_mail_rsrc).unwrap().put(mail_nft);
            }
            // Burn transient stamp resource.
            self.auth_badge.authorize(|| {stamp.burn()});
        }
        
        // Helper method for collecting determined fee from `fee_bucket`.
        fn collect_fee(&mut self, mut fee_bucket: Bucket, price: Decimal) -> Bucket {
            assert!(fee_bucket.resource_address() == self.fee_vault.resource_address(), "Fee must be paid in XRD.");
            assert!(fee_bucket.amount() >= price, "Insufficient funds given.");
            // Deposit fee.
            self.fee_vault.put(fee_bucket.take(price));
            // Return any change
            fee_bucket
        }
        
        // Helper method for minting a new stamp.
        fn mint_stamp(&self) -> Bucket {
            self.auth_badge.authorize(|| {
                borrow_resource_manager!(self.stamp_rsrc).mint(1)
            })
        }

        // Helper method for setting `mailbox_address` upon instantiation.
        pub fn set_component_address(&mut self, address: ComponentAddress) {
            match self.mailbox_address {
                None => self.mailbox_address = Some(address),
                Some(_) => panic!("Mailbox address already set.")
            }
        }

        // methods used in authentication
        pub fn get_stamp_rsrc(&self) -> ResourceAddress {
            self.stamp_rsrc
        }
        pub fn get_mail_rsrc(&self) -> ResourceAddress {
            self.mail_rsrc
        }
        fn get_pkg_addr(&self) -> PackageAddress {
            let mailbox_ref: MailboxGlobalComponentRef = self.mailbox_address.expect("invalid mailbox component address").into();
            mailbox_ref.package_address()
        }
    }
}