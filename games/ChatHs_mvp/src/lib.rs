use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct ExchangeNFT {
    
    prompt: String,
    price_paid: Decimal,
    start_time: u64,
    end_time: u64,
    #[mutable]
    response: String,
    complete: bool,
    
}

#[derive(NonFungibleData)]
pub struct ReceiptNFT {
    
    exchange_nft_id: NonFungibleLocalId,

}

#[blueprint]

mod chaths {

    struct ChatHs {
        
        internal_nft_mint_badge: Vault,
        exchange_nfts: Vault,
        receipt_nft_resource_address: ResourceAddress,
        thread_time: u64,
        xrd_payments: Vault,
    }

    impl ChatHs {

        pub fn instantiate_with_bio(bio: String) -> (ComponentAddress, Bucket) {

            // Accepts 'bio' argument and returns 'account' badge

            let account = ResourceBuilder::new_fungible()
            .divisibility(DIVISIBILITY_NONE)
            .metadata("name", "ChatHs account badge")
            .metadata("bio", bio)
            .restrict_withdraw(rule!(deny_all), LOCKED)
            .mint_initial_supply(1);
        
            let rules: AccessRules = AccessRules::new()
            .method("respond", rule!(require(account.resouce_address())), LOCKED)
            .default(rule!(allow_all), LOCKED);

            // mints the soulbound 'account' badge that will be returned to the caller
            // and sets the rule requiring the account-holder to have this badge in 
            // order to be able to call the 'respond' method and respond to prompts.

            // ***does the 'account' resource address need to be stored in a variable?***

            let internal_nft_mint_badge = ResourceBuilder::new_fungible()
            .divisibility(DIVISIBILITY_NONE)
            .metadata("name", "Internal NFT Mint Badge")
            .mint_initial_supply(1);

            // Mints the 'internal nft mint badge' required to mint and edit 'exchange' NFTs
            // and 'receipt' NFTs

            let exchange_nft_resource_address = ResourceBuilder::new_uuid_non_fungible()
                .metadata("name", "Exchange NFT")
                .mintable(
                    rule!(require(internal_nft_mint_badge.resource_address())),
                    LOCKED,
                )
                .burnable(
                    rule!(require(internal_nft_mint_badge.resource_address())),
                    LOCKED,
                )
                .updateable_non_fungible_data(
                    rule!(require(internal_nft_mint_badge.resource_address())),
                    LOCKED,
                )
                .create_with_no_initial_supply();

            // Creates the exchange NFT resource and puts its resource address into a
            // variable

            let receipt_nft_resource_address = ResourceBuilder::new_uuid_non_fungible()
                .metadata("name", "Receipt NFT")
                .mintable(
                    rule!(require(internal_nft_mint_badge.resource_address())),
                    LOCKED,
                )
                .burnable(
                    rule!(require(internal_nft_mint_badge.resource_address())),
                    LOCKED,
                )
                .updateable_non_fungible_data(
                    rule!(require(internal_nft_mint_badge.resource_address())),
                    LOCKED,
                )
                .create_with_no_initial_supply();

            // Creates the receipt NFT resource and puts its resource address into a
            // variable

                let mut chaths_account_component = Self {
                    
                    internal_nft_mint_badge: Vault::with_bucket(internal_nft_mint_badge),
                    exchange_nfts: Vault::new(exchange_nft_resource_address),
                    receipt_nft_resource_address: receipt_nft_resource_address,
                    thread_time: 0,
                    xrd_payments: Vault::new(RADIX_TOKEN),
                }
                .instantiate();

                chaths_account_component.add_access_check(rules);
                let chaths_account_address = chaths_account_component.globalize();

                (chaths_account_address, account)

                
        }

        pub fn prompt(&mut self, payment: Bucket, prompt: String) -> (Bucket, Bucket) {

            // Accepts a 'prompt' string together with a payment in XRD (from anyone)
            // Mints a partial 'exchange' NFT and 
            // Returns a bucket containing a 'receipt' NFT and a bucket with any change.

            Let time: u64 = Runtime::current_epoch();

            // gets the current time

            If time >= thread_time Let start_time: u64 = time // pseudocode
            Else Let start_time = thread_time // pseudocode

            // sets start_time at current time if it's later than the thread_time or
            // sets start_time as the thread_time
            
            Let end_time: u64 = start time + 24 // pseudocode

            // sets end_time to 24h after start_time

            Let required_payment_exponent =[ [ (end_time - time) / 24 ] rounded-up! ] -1 // pseudocode

            // calcutates the exponent for the required payment. Rounding it up ensures that
            // the required_payment will depend on the number of pending 'exchange' NFTs

            // ***will need to change its type to use in required_payment calc?***

            Let required_payment: Decimal = 20 * (2^required_payment_exponent) // pseudocode

            // calculates the required_payment by multiplying 20 by the required payment
            // multiplier which is calculated by 2^required_payment_exponent

            self.xrd_payments.put(payment.take(required_payment));

            // OR PANIC WITH MESSAGE INDICATING REQUIRED PAYMENT
 
            let new_pending_prompt = ExchangeNFT {
                prompt: prompt,
                price_paid: required_payment,
                start_time: start_time,
                end_time: end_time,
                response: // Empty String or omit field ??!?
                complete: FALSE,
            };

            let pending_prompt_bucket = self.internal_nft_mint_badge.authorize(|| {
                borrow_resource_manager!(self.exchange_nfts.resouce_address).mint_uuid_non_fungible(
                    new_pending_prompt)                    
            });

            // mints a new exchange NFT and puts it in a bucket

            let exchange_nft_id: NonFungibleLocalId = pending_prompt_bucket.non_fungible_local_id();

            // gets the uuid from the newly-minted exchange NFT
            
            self.exchange_nfts.put(pending_prompt_bucket);

            // puts the pending exchange NFT into the component's vault
            
            let new_receipt = ReceiptNFT {
                exchange_nft_id: exchange_nft_id,
            };

            let new_receipt_bucket = self.internal_nft_mint_badge.authorize(|| {
                borrow_resource_manager!(self.receipt_nft_resource_address).mint_uuid_non_fungible(
                    new_receipt)                    
            });

            // mints a new receipt NFT to return to the prompter

            Let mut thread_time = end_time // pseudocode

            // updates the thread_time because of the new pending 'exchange' NFT

            Return payment,
            Return new_receipt_bucket,

        }

        pub fn respond(&mut self, response: String) -> (Bucket) {

            // System-level authentication of responder holding soulbound 'account' badge
            // Accepts 'response' string and returns payment provided the response is submitted within
            // the appropriate timeframe
            //
            // Edits the 'exchange' NFT to append the 'response' string and change the 'complete' boolean to
            // 'TRUE'.
            //
            // ***how to get the component to select the correct exchange NFT to append the response to
            // without requiring the caller to include its ID as an argument to the method??!??***
            
        }

        pub fn claim_nft(receipt: Bucket) -> (Bucket) {

            // Accepts 'receipt' NFT and returns the appropriate 'exchange' NFT (if the 'exchange NFT is marked
            // complete). 

        }

        pub fn claim_refund(receipt: Bucket) -> (Bucket) {

        }

        pub fn remove_commission()
    }
}

