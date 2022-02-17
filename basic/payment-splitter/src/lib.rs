use scrypto::prelude::*;

/// Defines the data of the shareholders badge which is an NFT.
#[derive(NonFungibleData)]
pub struct ShareHolder {
    /// Defines the account address of the shareholder. While this address is never used after the airdrop of the NFTs,
    /// we still keep track of it just in case.
    pub address: Address,

    /// The number of shares that this shareholder owns.
    pub shares: Decimal,

    /// A mutable variable which defines the total amount of XRD that the payment splitter owes to the shareholder
    #[scrypto(mutable)]
    pub total_xrd_owed: Decimal,

    /// A mutable variables which defines the total amount of XRD that the shareholder has withdrawn so far from the
    /// payment splitter. We keep track of this to ensure that the shareholder does not withdraw more XRD than the total
    /// amount which we owe him.
    #[scrypto(mutable)]
    pub total_xrd_withdrawn: Decimal,
}

blueprint! {
    /// A PaymentSplitter is a Scrypto blueprint which allows for a way for funds to be distributed among shareholders
    /// in a project depending on the amount of shares that each of the shareholders own.
    struct PaymentSplitter {
        /// This is the vault where all of the XRD that is to be splitted is stored
        xrd_vault: Vault,

        /// The resource definition of the admin badge. This admin badge is used to mint the NFTs which give the 
        /// shareholders to authenticate them and keep track of how much they took so far from the splitter.
        admin_badge_def: ResourceDef,

        /// The vault containing the internal admin badge. This internal admin badge is mainly used for the updating
        /// of NFT metadata and specifically in the `deposit_xrd` function when we want to update the quantity that
        /// the shareholders are owed.
        internal_admin_badge: Vault,

        /// The resource definition of the shareholders NFT. This NFT is used to authenticate shareholders to allow
        /// for the withdrawal of funds from the payment splitter and is also used to keep track of information about
        /// this shareholder, the number of shares that they own, and the amount of XRD which they've been given so 
        /// far.
        shareholder_def: ResourceDef,

        /// This decimal number is used to keep track of the number of shareholders that we have so far under this
        /// payment splitter. This is used to allow for us to easily index all of the Shareholder NFTs.
        number_of_shareholders: u128,
    }

    impl PaymentSplitter {
        /// Creates a new Payment Splitter and returns it along with the admin badge back to the creator of the
        /// payment splitter.
        /// 
        /// # Returns
        /// 
        /// * `Component` - A PaymentSplitter component
        /// * `Bucket` - A bucket of the admin badge. This badge may be used to add shareholders to the PaymentSplitter.
        pub fn new() -> (Component, Bucket) {
            // Creating the admin badge
            let admin_badge: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Admin Badge")
                .metadata("symbol", "adm")
                .initial_supply_fungible(1);

            // Creating the internal admin badge
            let internal_admin_badge: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
            .metadata("name", "Internal Admin Badge")
            .metadata("symbol", "iadm")
            .initial_supply_fungible(1);

            // Creating the shareholders NFT Resource Definition. We will not mint any of them at this point
            // of time as the minting and creation of these tokens happens when the `add_shareholder` function
            // is called
            let shareholder_def: ResourceDef = ResourceBuilder::new_non_fungible()
                .metadata("name", "Shareholder")
                .metadata("symbol", "shr")
                .flags(MINTABLE | INDIVIDUAL_METADATA_MUTABLE)
                .badge(admin_badge.resource_address(), MAY_MINT)
                .badge(internal_admin_badge.resource_address(), MAY_CHANGE_INDIVIDUAL_METADATA)
                .no_initial_supply();

            // Creating the payment splitter component
            let payment_splitter: Component = Self {
                xrd_vault: Vault::new(RADIX_TOKEN),
                admin_badge_def: admin_badge.resource_def(),
                internal_admin_badge: Vault::with_bucket(internal_admin_badge),
                shareholder_def: shareholder_def,
                number_of_shareholders: 0
            }.instantiate();

            // Returning the PaymentSplitter component and the admin badge
            return (payment_splitter, admin_badge);
        }

        /// Adds a shareholder to the PaymentSplitter
        /// 
        /// This is an authenticated method which needs to be called and presented the admin badge (obtained from creating the component).
        /// This method is used to add a shareholder to the PaymentSplitter by minting an NFT to authenticate and keep track of the shareholder
        /// data.
        /// 
        /// # Arguments
        /// 
        /// * `shareholder_address` - The address of the shareholder. This address will be airdropped the NFT.
        /// * `shareholder_shares` - The amount of shares that belong to this shareholder.
        #[auth(admin_badge_def)]
        pub fn add_shareholder(
            &mut self,
            shareholder_address: Address,
            shareholder_shares: Decimal
        ) -> Bucket {
            // Creating this NFT for this specific shareholder and storing it in a bucket
            let nft_bucket: Bucket = self.shareholder_def.mint_non_fungible(
                &NonFungibleKey::from(self.number_of_shareholders),
                ShareHolder {
                    address: shareholder_address,
                    shares: shareholder_shares,
                    total_xrd_owed: Decimal(0),
                    total_xrd_withdrawn: Decimal(0)
                },
                auth
            );
            self.number_of_shareholders += 1;

            // Logging the addition of the shareholder to the payment splitter
            info!("Added shareholder {} with shares {} to the splitter. Number of current shareholders: {}", shareholder_address, shareholder_shares, self.number_of_shareholders);
            
            // Now that we have created the shareholder NFTs, there two two main ways we can get these NFTs to the shareholders:
            //
            // 1- We could airdrop them to the shareholders through the account component.
            // 2- We could return the shareholder's badge as a bucket and trust that the caller would send them their badges.
            //
            // Method 1 seems somewhat better as there is no need to trust the admin to send the badges when they're created and 
            // admin can't go back on their word one the badges are created. However, this would go against the asset oriented 
            // style of coding used in Radix. So, if you want to directly airdrop the tokens instead of returning them in a bucket
            // then you can use the line of code below:
            // Component::from(shareholder_address).call::<()>("deposit", vec![scrypto_encode(&nft_bucket)]);

            // Returning the bucket of shareholder NFTs back to the caller
            return nft_bucket;
        }

        /// Used to deposit XRD into the XRD vault controlled by this component.
        /// 
        /// This method is used to deposit XRD into the vault that this component controls. When this XRD is deposited into the Vault,
        /// we update the NFT metadata to add XRD to the balance of the shareholders that they may withdraw at any point of time.
        /// 
        /// # Arguments
        /// 
        /// * `xrd_bucket` - A bucket of XRD that we're depositing into our component.
        pub fn deposit_xrd(&mut self, xrd_bucket: Bucket) {
            // Getting the amount of XRD that passed in the XRD bucket
            let xrd_amount: Decimal = xrd_bucket.amount();
            info!("Depositing XRD of amount: {}", xrd_amount);

            // Taking the XRD bucket passed and depositing it into the XRD vault
            self.xrd_vault.put(xrd_bucket);

            // Getting the total number of shares that we have across all of our shareholders
            let mut total_quantity_of_shares: Decimal = Decimal(0);
            for i in 0..self.number_of_shareholders {
                let shareholder: ShareHolder = self.shareholder_def.get_non_fungible_data(&NonFungibleKey::from(i));
                total_quantity_of_shares += shareholder.shares;
            }
            info!("Total number of shares: {}", total_quantity_of_shares);

            // Updating the total_xrd_owed for all of the shareholders since the PaymentSplitter 
            // has just received a new payment
            for i in 0..self.number_of_shareholders {
                // Creating a non fungible key from the variable i
                let key: NonFungibleKey = NonFungibleKey::from(i);

                // Loading in the data for this specific shareholder
                let mut shareholder: ShareHolder = self.shareholder_def.get_non_fungible_data(&key);
                shareholder.total_xrd_owed += xrd_amount * shareholder.shares / total_quantity_of_shares;

                info!("XRD owed to {} is {}", i, shareholder.total_xrd_owed);

                // Updating the metadata of the NFT with the newly calculated information
                self.internal_admin_badge.authorize(|auth| {
                    self.shareholder_def.update_non_fungible_data(&key, shareholder, auth);
                })
            }
        }

        /// Withdraws the amount of XRD that is owed to the shareholder.
        /// 
        /// This is an authenticated method that only a valid shareholder can call by presenting their shareholder badge when calling
        /// this method. This method withdraws the total amount that we currently owe the shareholder from the XRD vault and returns
        /// it back to them in a bucket. If this specific shareholder has already withdrawn all of their owed amount then an empty 
        /// bucket of XRD is returned.
        /// 
        /// # Returns
        /// 
        /// * `Bucket` - A bucket containing the XRD owed to the shareholder.
        #[auth(shareholder_def)]
        pub fn withdraw_xrd(&mut self) -> Bucket {
            // Getting the ID of the NFT passed in the badge and then loading in the nft data
            let nft_id: NonFungibleKey = auth.get_non_fungible_key(); 
            let mut shareholder: ShareHolder = self.shareholder_def.get_non_fungible_data(&nft_id);
            info!("Shareholder is entitled to: {}", shareholder.total_xrd_owed);

            // Creating a bucket with the maximum amount of XRD that this shareholder may withdraw
            let xrd_bucket: Bucket = self.xrd_vault.take(shareholder.total_xrd_owed - shareholder.total_xrd_withdrawn);
            shareholder.total_xrd_withdrawn += xrd_bucket.amount();
            info!("Withdrawing: {}", xrd_bucket.amount());

            // Updating the NFT metadata to indicate that the withdraw has taken place and that this shareholder
            // has been given this portion of their shares
            self.internal_admin_badge.authorize(|auth| {
                self.shareholder_def.update_non_fungible_data(&nft_id, shareholder, auth);
            });

            // Return the XRD bucket back to the shareholder
            return xrd_bucket;
        }
    }
}