use scrypto::prelude::*;
mod deck;

#[blueprint]
mod card_deck_nft {
    struct NFTcards {
        /// This is the vault to store admin badge
        admin_badge: Vault,
        /// Resource address for all the Decks
        new_deck_resource_address: ResourceAddress,
        /// Bet price
        min_price: Decimal,
        /// A counter for ID generation
        deck_id_counter: u64,
        /// A vault that collects all XRD payments to mint the NFT
        xrd_vault: Vault,
    }

    impl NFTcards {
        pub fn instantiate_new() -> ComponentAddress {
            // Create the badge
            let admin_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "New Deck Mint Badge")
                .mint_initial_supply(1);

            // NFT data
            let new_deck_resource_address =
                // Take note that the Struct is in another file named deck
                // and the field values are a vector of another Struct <Card>
                ResourceBuilder::new_integer_non_fungible::<deck::Deck>()
                    .metadata("name", "Deck of Cards")
                    .metadata("symbol", "S52")
                    .mintable(
                        rule!(require(admin_badge.resource_address())),
                        LOCKED,
                    )
                    .burnable(
                        rule!(require(admin_badge.resource_address())),
                        LOCKED,
                    )
                    .updateable_non_fungible_data(
                        rule!(require(admin_badge.resource_address())),
                        LOCKED,
                    )
                    .create_with_no_initial_supply();

            // Instantiate the component
            Self {
                admin_badge: Vault::with_bucket(admin_badge),
                new_deck_resource_address,
                min_price: 5.into(),
                deck_id_counter: 0,
                xrd_vault: Vault::new(RADIX_TOKEN),
            }
            .instantiate()
            .globalize()
        }

        /// Mint a new deck of cards NFT
        pub fn mint_deck(&mut self, mut payment: Bucket) -> (Bucket, Bucket) {
            // Check if payment is made in XRD (This will only accept payments in XRD)
            assert!(
                payment.resource_address() == RADIX_TOKEN,
                "Will only accept XRD"
            );
            // Check if payment sent satisfies the min price set
            assert!(
                payment.amount() >= self.min_price,
                "Mint price is {} XRD. Payment insufficient.",
                self.min_price
            );
            // Take our price out of the payment bucket and add to vault
            self.xrd_vault.put(payment.take(self.min_price));

            // Create new Deck
            let data = deck::Deck::new();
            let nft_bucket = self.admin_badge.authorize(|| {
                borrow_resource_manager!(self.new_deck_resource_address).mint_non_fungible(
                    &NonFungibleLocalId::Integer(self.deck_id_counter.into()),
                    data,
                )
            });

            // This give an ID identifier to every new mint
            self.deck_id_counter += 1;

            // THIS BLOCK OF CODE IS ONLY TO SHOW THE CARD DECK FIELD cards
            // Just to verify that the NFT indeed has all 52 cards inside
            // This is not necessary but usefuly for debuging and can be deleted
            let view: deck::Deck = nft_bucket.non_fungible().data();
            //let show = view.cards;
            let show = view
                .cards
                .iter()
                .map(|card| card.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let count = view.cards.len();
            info!("Your Deck of Cards ({} cards): {}", count, show);
            // END BLOCK

            // Return the NFT and change if overpayment has been made
            (nft_bucket, payment)
        }

        /// Shuffle the cards
        // THIS FUNCTION IS NOT VERY USEFUL BUT IT DEMONSTRATES
        // HOW WE CAN TAKE THE DATA AND FROM A FIELD AND THEN MUTATE
        pub fn shuffle_cards(&mut self, nft_bucket: Bucket) -> Bucket {
            // Check how many NFTs were sent
            assert!(
                nft_bucket.amount() == dec!("1"),
                "You can only shuffle 1 Deck of Cards at a time!"
            );
            // Check if NFT sent is correct
            assert!(
                nft_bucket.resource_address() == self.new_deck_resource_address,
                "Passed a wrong NFT"
            );

            // THIS BLOCK IS JUST TO MAKE A PRINT OUT TO SHOW
            // THAT THE DECK HAS BEEN SHUFFLED
            let mut new_data: deck::Deck = nft_bucket.non_fungible().data();
            new_data.shuffle();
            info!("Shuffled Deck: {}", new_data);
            // END BLOCK

            // Return the NFT to the owner
            nft_bucket
        }

        /// Deal Cards
        // This function is deal N cards from the deck and give to player hand
        pub fn deal(&mut self, nft_bucket: Bucket, n: i8) -> Bucket {
            // Check how many NFTs were sent
            assert!(
                nft_bucket.amount() == dec!("1"),
                "You can only shuffle 1 Deck of Cards at a time!"
            );

            // Check if NFT sent is correct
            assert!(
                nft_bucket.resource_address() == self.new_deck_resource_address,
                "Passed a wrong NFT"
            );

            // Gets the cards data from the NFT
            let mut deck_data: deck::Deck = nft_bucket.non_fungible().data();

            // Instantiates a player's hand (0 cards)
            let mut player_hand = deck::Deck::player_hand();

            // This will deal N number of cards to the player taking from the stack
            for _ in 0..n {
                deck_data.shuffle(); // NOTE: IT WILL RESHUFFLE AFTER EVERY CARD DEALT
                if let Some(card) = deck_data.cards.pop() {
                    player_hand.cards.push(card);
                } else {
                    info!("No more cards to deal!")
                }

                /* THIS IS JUST TO DEBUG EACH ITERATION OF CARDS DEALT
                // Delete this after debug
                info!(
                    "Remaining deck: {} -- cards: {}",
                    deck_data.cards.len(),
                    deck_data
                );
                info!(
                    "Your hand {} cards: {}",
                    player_hand.cards.len(),
                    player_hand
                );
                //end line of delete
                */
            }

            // Print your hands on hand.
            info!(
                "Your hand {} cards: {}",
                player_hand.cards.len(),
                player_hand
            );

            // Return the NFT
            nft_bucket
        }
    }
}
