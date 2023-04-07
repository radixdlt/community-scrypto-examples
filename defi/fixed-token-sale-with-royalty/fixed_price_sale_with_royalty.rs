use scrypto::prelude::*;

struct Royalty {
    /// This is the value of the royalty recipient.
    recipient: Vault,

    /// This is the percentage of the sale price that will be paid to the royalty recipient.
    percentage: Decimal,
}

#[blueprint]
mod fixed_price_sale_with_royalty {
    /// This blueprint defines the state and logic involved in a fixed-price non-fungible token sale. People who
    /// instantiate components from this blueprint, signify their intent at selling their NFT(s) at a fixed price of
    /// their choosing and for a token of their choosing.
    ///
    /// This blueprint allows multiple NFTs to be sold at once as a collection instead of requiring that these NFTs
    /// be sold separately. In addition to that, this blueprint allows XRD payments as well as non-XRD payments for
    /// sellers who opt to accept non-XRD tokens.
    struct FixedPriceSale {
        /// These are the vaults where the NFTs will be stored. Since this blueprint allows for multiple NFTs to be sold
        /// at once, this HashMap is used to store all of these NFTs with the hashmap key being the resource address of
        /// these NFTs if they are not all of the same _kind_.
        nft_vaults: HashMap<ResourceAddress, Vault>,

        /// This is the vault which stores the payment of the NFTs once it has been made. This vault may contain XRD or
        /// other tokens depending on the `ResourceAddress` of the accepted payment token specified by the instantiator
        payment_vault: Vault,

        /// This blueprint accepts XRD as well as non-XRD payments. This variable here is the resource address of the
        /// fungible token that will be used for payments to the component.
        accepted_payment_token: ResourceAddress,

        /// This is the price of the bundle of NFTs being sold. This price is in the `accepted_payment_token` which
        /// could be XRD or any other fungible token.
        price: Decimal,

        /// The list of royalties that will be paid out.
        royalties: Vec<Royalty>,
    }

    impl FixedPriceSaleWithRoyalty {
        /// Instantiates a new fixed-price sale for the passed NFTs.
        ///
        /// This function is used to instantiate a new fixed-price sale for the passed bucket of NFTs. The fixed price
        /// sale can be done for a single NFT or a bundle of NFTs which the seller intends to sell together. The tokens
        /// may be sold for XRD or for any other fungible token of the instantiator's choosing.
        ///
        /// This function performs a number of checks before the `FixedPriceSale` component is created:
        ///
        /// * **Check 1:** Checks that the passed buckets of tokens are all non-fungible tokens.
        /// * **Check 2:** Checks that the `accepted_payment_token` is a fungible token.
        /// * **Check 3:** Checks that the price is non-negative.
        /// * **Check 4:** Checks that the royalty percentages are non-negative and add up to less than 100%.
        ///
        /// # Arguments:
        ///
        /// * `non_fungible_tokens` (Vec<Bucket>) - A vector of buckets of the non-fungible tokens that the instantiator
        /// wishes to sell.
        /// * `accepted_payment_token` (ResourceAddress) - Payments may be accepted in XRD or non-XRD tokens. This
        /// argument specifies the resource address of the token the instantiator wishes to accept for payment.
        /// * `price` (Decimal) - The price of the bundle of NFTs given in `accepted_payment_token`.
        /// * `royalties` (Vec<Royalty>) - A vector of royalties that will be paid out.
        ///
        /// # Returns:
        ///
        /// This function returns a tuple which has the following format:
        /// * `ComponentAddress` - A component address of the instantiated `FixedPriceSale` component.
        /// * `Bucket` - A bucket containing an ownership badge which entitles the holder to the assets in this
        /// component.
        pub fn instantiate_fixed_price_sale(
            non_fungible_tokens: Vec<Bucket>,
            accepted_payment_token: ResourceAddress,
            price: Decimal,
            royalties: Vec<Royalty>,
        ) -> (ComponentAddress, Bucket) {
            // Performing checks to ensure that the creation of the component can go through
            assert!(
                !non_fungible_tokens.iter().any(|x| !matches!(
                    borrow_resource_manager!(x.resource_address()).resource_type(),
                    ResourceType::NonFungible { id_type: _ }
                )),
                "[Instantiation]: Can not perform a sale for fungible tokens."
            );
            assert!(
                !matches!(
                    borrow_resource_manager!(accepted_payment_token).resource_type(),
                    ResourceType::NonFungible { id_type: _ }
                ),
                "[Instantiation]: Only payments of fungible resources are accepted."
            );
            assert!(
                price >= Decimal::zero(),
                "[Instantiation]: The tokens can not be sold for a negative amount."
            );
            assert!(
                !royalties.iter().any(|x| x.percentage < Decimal::zero()),
                "[Instantiation]: Royalty percentages can not be negative."
            )
            assert!(
                royalties.iter().fold(Decimal::zero(), |acc, x| acc + x.percentage) <= Decimal::from(100),
                "[Instantiation]: Royalty percentages can not add up to more than 100%."
            );

            // At this point we know that the component creation can go through.

            // Create a new HashMap of vaults and aggregate all of the tokens in the buckets into the vaults of this
            // HashMap. This means that if somebody passes multiple buckets of the same resource, then they would end
            // up in the same vault.
            let mut nft_vaults: HashMap<ResourceAddress, Vault> = HashMap::new();
            for bucket in non_fungible_tokens.into_iter() {
                nft_vaults
                    .entry(bucket.resource_address())
                    .or_insert(Vault::new(bucket.resource_address()))
                    .put(bucket)
            }

            // When the owner of the NFT(s) instantiates a new fixed-price sale component, their tokens are taken away
            // from them and they're given an ownership NFT which is used to authenticate them and as proof of ownership
            // of the NFTs. This ownership badge can be used to either withdraw the funds from the token sale or the
            // NFTs if the seller is no longer interested in selling their tokens.
            let ownership_badge: Bucket = ResourceBuilder::new_fungible()
                .metadata("name", "Ownership Badge")
                .metadata(
                    "description",
                    "An ownership badge used to authenticate the owner of the NFT(s).",
                )
                .metadata("symbol", "OWNER")
                .mint_initial_supply(1);

            // Setting up the access rules for the component methods such that only the owner of the ownership badge can
            // make calls to the protected methods.
            let access_rule: AccessRule = rule!(require(ownership_badge.resource_address()));
            let access_rules: AccessRules = AccessRules::new()
                .method("cancel_sale", access_rule.clone(), AccessRule::DenyAll)
                .method("change_price", access_rule.clone(), AccessRule::DenyAll)
                .method("withdraw_payment", access_rule.clone(), AccessRule::DenyAll)
                .default(rule!(allow_all), AccessRule::DenyAll);

            // Instantiating the fixed price sale component
            let mut fixed_price_sale: FixedPriceSaleComponent = Self {
                nft_vaults,
                payment_vault: Vault::new(accepted_payment_token),
                accepted_payment_token,
                price,
                royalties,
            }
            .instantiate();
            fixed_price_sale.add_access_check(access_rules);
            let fixed_price_sale: ComponentAddress = fixed_price_sale.globalize();

            return (fixed_price_sale, ownership_badge);
        }

        /// Used for buying the NFT(s) controlled by this component.
        ///
        /// This method takes in the payment for the non-fungible tokens being sold, verifies that the payment matches
        /// the expected resource addresses and amounts and then permits the exchange of NFTs and payment.
        ///
        /// This method performs a number of checks before the purchase goes through:
        ///
        /// * **Check 1:** Checks that the payment was provided in the required token.
        /// * **Check 1:** Checks that enough tokens were provided to cover the price of the NFT(s).
        ///
        /// # Arguments:
        ///
        /// * `payment` (Bucket) - A bucket of the tokens used for the payment
        ///
        /// # Returns:
        ///
        /// * `Vec<Bucket>` - A vector of buckets of the non-fungible tokens which were being sold.
        pub fn buy(&mut self, mut payment: Bucket) -> Vec<Bucket> {
            // Checking if the appropriate amount of the payment token was provided before approving the token sale
            assert_eq!(
                payment.resource_address(),
                self.accepted_payment_token,
                "[Buy]: Invalid tokens were provided as payment. Payment are only allowed in {:?}",
                self.accepted_payment_token
            );
            assert!(
                payment.amount() >= self.price,
                "[Buy]: Invalid quantity was provided. This sale can only go through when {} tokens are provided.",
                self.price
            );

            // At this point we know that the sale of the tokens can go through.

            // Track the total paid in royalties
            let mut total_royalties_paid: Decimal = Decimal::zero();

            // Loop through royalty percentages and pay out the royalties
            for royalty in self.royalties.iter() {
                let royalty_amount: Decimal = royalty.percentage * self.price;
                royalty.recipient.put(payment.take(royalty_amount));

                // Add the royalty amount to the total paid in royalties
                total_royalties_paid += royalty_amount;
            }

            // Taking the price of the NFT(s) and putting it in the payment vault
            self.payment_vault.put(payment.take(self.price - total_royalties_paid));

            // Creating a vector of buckets of all of the NFTs that the component has, then adding to it the remaining
            // tokens from the payment
            let resource_addresses: Vec<ResourceAddress> =
                self.nft_vaults.keys().cloned().collect();
            let mut tokens: Vec<Bucket> = vec![payment];
            for resource_address in resource_addresses.into_iter() {
                tokens.push(
                    self.nft_vaults
                        .get_mut(&resource_address)
                        .unwrap()
                        .take_all(),
                )
            }

            return tokens;
        }

        /// Cancels the sale of the tokens and returns the tokens back to their owner.
        ///
        /// This method performs a single check before canceling the sale:
        ///
        /// * **Check 1:** Checks that the tokens have not already been sold.
        ///
        /// # Returns:
        ///
        /// * `Vec<Bucket>` - A vector of buckets of the non-fungible tokens which were being sold.
        ///
        /// # Note:
        ///
        /// * This is an authenticated method which may only be called by the holder of the `ownership_badge`.
        /// * There is no danger in not checking if the sale has occurred or not and attempting to return the tokens
        /// anyway. This is because we literally lose the tokens when they're sold so even if we attempt to give them
        /// back after they'd been sold we return a vector of empty buckets.
        pub fn cancel_sale(&mut self) -> Vec<Bucket> {
            // Checking if the tokens have been sold or not.
            assert!(
                !self.is_sold(),
                "[Cancel Sale]: Can not cancel the sale after the tokens have been sold"
            );

            // Taking out all of the tokens from the vaults and returning them back to the caller.
            let resource_addresses: Vec<ResourceAddress> =
                self.nft_vaults.keys().cloned().collect();
            let mut tokens: Vec<Bucket> = Vec::new();
            for resource_address in resource_addresses.into_iter() {
                tokens.push(
                    self.nft_vaults
                        .get_mut(&resource_address)
                        .unwrap()
                        .take_all(),
                )
            }

            return tokens;
        }

        /// Withdraws the payment owed from the sale.
        ///
        /// This method performs a single check before canceling the sale:
        ///
        /// * **Check 1:** Checks that the tokens have already been sold.
        ///
        /// # Returns:
        ///
        /// * `Bucket` - A bucket containing the payment.
        ///
        /// # Note:
        ///
        /// * This is an authenticated method which may only be called by the holder of the `ownership_badge`.
        /// * There is no danger in not checking if the sale has occurred or not and attempting to return the tokens
        /// anyway. If we do not have the payment tokens then the worst case scenario would be that an empty bucket is
        /// returned. This is bad from a UX point of view but does not pose any security risk.
        pub fn withdraw_payment(&mut self) -> Bucket {
            // Checking if the tokens have been sold or not.
            assert!(
                self.is_sold(),
                "[Withdraw Payment]: Can not withdraw the payment when no sale has happened yet."
            );
            return self.payment_vault.take_all();
        }

        /// Updates price of the NFT(s) to a new price.
        ///
        /// This method updates the price of the NFT or NFT bundle to a new price. It should be noted that this price is
        /// assumed to be in the `accepted_payment_token`.
        ///
        /// # Arguments:
        ///
        /// * `price` (Decimal) - The new price of the non-fungible tokens.
        ///
        /// # Note:
        ///
        /// This is an authenticated method which may only be called by the holder of the `ownership_badge`.
        pub fn change_price(&mut self, price: Decimal) {
            // Checking that the new price can be set
            assert!(
                price >= Decimal::zero(),
                "[Change Price]: The tokens can not be sold for a negative amount."
            );
            self.price = price;
        }

        // =============================================================================================================
        // The following are read-only methods which query the state of the fixed price sale and information about it
        // without performing any state changes. These are useful when connecting a web interface with the component.
        // =============================================================================================================

        /// Returns the price of the tokens being sold.
        ///
        /// # Returns:
        ///
        /// This method returns a tuple in the following format
        ///
        /// * `ResourceAddress` - The resource address of the accepted payment token.
        /// * `Decimal` - A decimal value of the price of the NFT(s) in terms of the `accepted_payment_token`.
        pub fn price(&self) -> (ResourceAddress, Decimal) {
            return (self.accepted_payment_token, self.price);
        }

        /// Checks if the NFTs have been sold or not.
        ///
        /// This method checks whether the NFTs have been sold or not through the `payment_vault`. If the payment vault
        /// is empty then it means that the tokens have not been sold. On the other hand, if there are funds in the
        /// payment vault then the exchange has gone through and the tokens have been sold.
        ///
        /// # Returns:
        ///
        /// * `bool` - A boolean of whether the tokens have been sold or not. Returns `true` if the tokens have been
        /// sold and `false` if they have not been sold.
        pub fn is_sold(&self) -> bool {
            return !self.payment_vault.is_empty();
        }

        /// Returns a HashMap of the NFTs being sold through this component.
        ///
        /// This method returns a `HashMap` of the NFTs being sold through this component. The key of the HashMap is the
        /// `ResourceAddress` of the resource and the value is a vector of `NonFungibleIds` belonging to this
        /// `ResourceAddress` that are being sold.
        ///
        /// # Returns:
        ///
        /// * `bool` - A HashMap of the non-fungible-ids of the tokens being sold.
        pub fn non_fungible_ids(&self) -> HashMap<ResourceAddress, Vec<NonFungibleLocalId>> {
            // Creating the hashmap which we will use to store the resource addresses and the non-fungible-ids.
            let mut mapping: HashMap<ResourceAddress, Vec<NonFungibleLocalId>> = HashMap::new();

            // Adding the entires to the mapping
            let resource_addresses: Vec<ResourceAddress> =
                self.nft_vaults.keys().cloned().collect();
            for resource_address in resource_addresses.into_iter() {
                mapping.insert(
                    resource_address.clone(),
                    self.nft_vaults
                        .get(&resource_address)
                        .unwrap()
                        .non_fungible_local_ids()
                        .into_iter()
                        .collect::<Vec<NonFungibleLocalId>>(),
                );
            }

            return mapping;
        }

        /// Returns a `NonFungibleAddress` vector of the NFTs being sold.
        ///
        /// # Returns:
        ///
        /// * `Vec<NonFungibleAddress>` - A Vector of `NonFungibleAddress`es of the NFTs being sold.
        pub fn non_fungible_addresses(&self) -> Vec<NonFungibleGlobalId> {
            // Creating the vector which will contain the NonFungibleAddresses of the tokens
            let mut vec: Vec<NonFungibleGlobalId> = Vec::new();

            // Iterate over the items in the hashmap of non-fungible-ids and create the `NonFungibleAddress`es through
            // them
            for (resource_address, non_fungible_ids) in self.non_fungible_ids().iter() {
                vec.append(
                    &mut non_fungible_ids
                        .iter()
                        .map(|x| NonFungibleGlobalId::new(resource_address.clone(), x.clone()))
                        .collect::<Vec<NonFungibleGlobalId>>(),
                )
            }

            return vec;
        }
    }
}