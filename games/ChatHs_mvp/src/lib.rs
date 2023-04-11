use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct ExchangeNFT {
    
    pub prompt: String,
    pub price_paid: Decimal,
    pub start_time: u64,
    pub end_time: u64,
    #[mutable]
    pub response: String,
    #[mutable]
    pub complete: bool,
    
}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct ReceiptNFT {
    
    pub exchange_nft_id: NonFungibleLocalId,
    pub due_time: u64,

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
        
            let rules = AccessRulesConfig::new()
            .method("respond", rule!(require(account.resource_address())), LOCKED)
            .default(rule!(allow_all), LOCKED);

            // mints the soulbound 'account' badge that will be returned to the caller
            // and sets the rule requiring the account-holder to have this badge in 
            // order to be able to call the 'respond' method and respond to prompts.

            let internal_nft_mint_badge = ResourceBuilder::new_fungible()
            .divisibility(DIVISIBILITY_NONE)
            .metadata("name", "Internal NFT Mint Badge")
            .mint_initial_supply(1);

            // Mints the 'internal nft mint badge' required to mint and edit 'exchange' NFTs
            // and 'receipt' NFTs

            let exchange_nft_resource_address = ResourceBuilder::new_uuid_non_fungible::<ExchangeNFT>()
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

            let receipt_nft_resource_address = ResourceBuilder::new_uuid_non_fungible::<ReceiptNFT>()
                .metadata("name", "Receipt NFT")
                .mintable(
                    rule!(require(internal_nft_mint_badge.resource_address())),
                    LOCKED,
                )
                .burnable(
                    rule!(require(internal_nft_mint_badge.resource_address())),
                    LOCKED,
                )
                .create_with_no_initial_supply();

            // Creates the receipt NFT resource and puts its resource address into a
            // variable

                let chaths_account_component = Self {
                    
                    internal_nft_mint_badge: Vault::with_bucket(internal_nft_mint_badge),
                    exchange_nfts: Vault::new(exchange_nft_resource_address),
                    receipt_nft_resource_address: receipt_nft_resource_address,
                    thread_time: 0,
                    xrd_payments: Vault::new(RADIX_TOKEN),
                }
                .instantiate();

                let chaths_account_address = chaths_account_component.globalize_with_access_rules(rules);

                (chaths_account_address, account)

                // Instantiates the component and returns the soulbound account-NFT

        }

        pub fn prompt(&mut self, mut payment: Bucket, prompt: String) -> (Bucket, Bucket) {

            // Accepts a 'prompt' string together with a payment in XRD (from anyone)
            // Mints a partial 'exchange' NFT and 
            // Returns a bucket containing a 'receipt' NFT and a bucket with any change.

            let time: u64 = Runtime::current_epoch();

            // gets the current time

            let start_time: u64 = if time >= self.thread_time {
                time
            } else {
                self.thread_time
            };

            // sets start_time at current time if it's later than the thread_time or
            // sets start_time as the thread_time
            
            let end_time: u64 = start_time + 24u64;

            // sets end_time to ~24h after start_time

            let time_to_due: u32 = end_time as u32 - time as u32;

            let required_payment_exponent: u32 = ( (time_to_due + 23u32) / 24u32 ) -1;

            // calcutates the exponent for the required payment. The '+23' is '+' the divisor
            // '-1' and is intended to produce a ceiling division, rounding the exponent *up*.
            // 
            // Rounding it up ensures that the required_payment will depend on the number of pending
            // 'exchange' NFTs
            
            let required_payment: u64 = ( 2u64.pow(required_payment_exponent) ) * 20u64;

            // calculates the required_payment by multiplying 20 by the required payment
            // multiplier which is calculated by 2^required_payment_exponent
            // note that the 'pow' for u64 requires a u32 exponent

            let required_payment: Decimal = required_payment.into();
            
            // puts the required_payment in a Decimal variable for later use

            let take_payment = payment.take(required_payment);
            
            self.xrd_payments.put(take_payment);

            // OR PANIC WITH MESSAGE INDICATING REQUIRED PAYMENT
 
            let new_pending_prompt = ExchangeNFT {
                prompt: prompt,
                price_paid: required_payment,
                start_time: start_time,
                end_time: end_time,
                response: "".to_string(),
                complete: false,
            };

            let pending_prompt_bucket = self.internal_nft_mint_badge.authorize(|| {
                borrow_resource_manager!(self.exchange_nfts.resource_address()).mint_uuid_non_fungible(
                    new_pending_prompt)                    
            });

            // mints a new exchange NFT and puts it in a bucket

            let exchange_nft_id: NonFungibleLocalId = pending_prompt_bucket.non_fungible_local_id();

            // gets the uuid from the newly-minted exchange NFT
            
            self.exchange_nfts.put(pending_prompt_bucket);

            // puts the pending exchange NFT into the component's vault
            
            let new_receipt = ReceiptNFT {
                exchange_nft_id: exchange_nft_id,
                due_time: end_time,
            };

            let new_receipt_bucket = self.internal_nft_mint_badge.authorize(|| {
                borrow_resource_manager!(self.receipt_nft_resource_address).mint_uuid_non_fungible(
                    new_receipt)                    
            });

            // mints a new receipt NFT to return to the prompter

            self.thread_time = end_time;

            // updates the thread_time because of the new pending 'exchange' NFT

            (payment, new_receipt_bucket)

            // returns any excess payment and the receipt

        }

        pub fn respond(&mut self, nft_id: NonFungibleLocalId, response: String) -> Bucket {

            // System-level authentication of responder holding soulbound 'account' badge
            // Accepts 'response' string and returns payment provided the response is submitted within
            // the appropriate timeframe
            //
            // Edits the 'exchange' NFT to append the 'response' string and change the 'complete' boolean to
            // 'true'.

            let exchange_nft_bucket = self.exchange_nfts.take_non_fungible(&nft_id);

            assert!(!(exchange_nft_bucket.is_empty()),
                "This NFT is not in the vault."
            );
            
            // takes the appropriate exchange NFT from the exchange_nfts vault
            
            let non_fungible_data: ExchangeNFT = exchange_nft_bucket.non_fungible().data();
            
            let time: u64 = Runtime::current_epoch();

            assert!(
                time >= non_fungible_data.start_time,
                "You cannot respond to this prompt yet."
            );

            assert!(
                time <= non_fungible_data.end_time,
                "You can no longer respond to this prompt."
            );

            assert!(
                non_fungible_data.complete == false,
                "You have already responded to this prompt."
            );

            // checks that the response is submitted within the appropriate timeframe
            // and that another response has not already been submitted


            let payment_amount: Decimal = non_fungible_data.price_paid * dec!("0.9");

            // calculates the payment to return, minus 10% commission
            
            self.internal_nft_mint_badge.authorize(|| {
                borrow_resource_manager!(self.exchange_nfts.resource_address()).update_non_fungible_data(
                    &nft_id,
                    "response",
                    response
                )                    
            });

            self.internal_nft_mint_badge.authorize(|| {
                borrow_resource_manager!(self.exchange_nfts.resource_address()).update_non_fungible_data(
                    &nft_id,
                    "complete",
                    true
                )                    
            });

            // appends the response to the 'exchange NFT' and marks it as complete

            let payment_bucket: Bucket = self.xrd_payments.take(payment_amount);

            // puts the payment in a bucket

            self.exchange_nfts.put(exchange_nft_bucket);

            // puts the completed 'exchange NFT' back in the vault ready to be claimed.

            payment_bucket

            // returns the payment

        }

        pub fn claim_nft(&mut self, receipt: Bucket) -> Bucket {

        // Accepts 'receipt' NFT and returns the appropriate 'exchange' NFT (if the 'exchange NFT' is marked
        // complete). 

            assert!(
                receipt.resource_address() == self.receipt_nft_resource_address,
                "Please submit a vaild receipt."
            );

            // checks that a vaild receipt has been submitted
            
            let time: u64 = Runtime::current_epoch();

            let non_fungible_data: ReceiptNFT = receipt.non_fungible().data();

            assert!(
                time > non_fungible_data.due_time,
                "This NFT is not claimable yet."
            );
            
            let exchange_nft_id: NonFungibleLocalId = non_fungible_data.exchange_nft_id;

            // gets the exchange_nft_id from the receipt NFT (hopefully, may be wrong)

            let exchange_nft_bucket = self.exchange_nfts.take_non_fungible(&exchange_nft_id);

            // takes the appropriate exchange NFT from the exchange_nfts vault
            //
            // note that there's no check on whether it's in the vault because the existence of
            // the receipt means that it is

            let exchange_nft_non_fungible_data: ExchangeNFT = exchange_nft_bucket.non_fungible().data();

            assert!(
                exchange_nft_non_fungible_data.complete == true,
                "There was no response to your prompt, please claim a refund."
            );

            // checks that the exchange NFT has been completed

            self.internal_nft_mint_badge.authorize(|| {
                receipt.burn();
            });

            // burns the receipt

            exchange_nft_bucket

            // returns the completed exchange NFT
            
        }

        pub fn claim_refund(&mut self, receipt: Bucket) -> Bucket {

        // Accepts 'receipt' NFT and refunds the payment if the 'exchange' NFT has
        // not been completed in the required timeframe

            assert!(
                receipt.resource_address() == self.receipt_nft_resource_address,
                "Please submit a vaild receipt."
            );

            // checks that a vaild receipt has been submitted
            
            let time: u64 = Runtime::current_epoch();

            let non_fungible_data: ReceiptNFT = receipt.non_fungible().data();

            let due_time: u64 = non_fungible_data.due_time;

            assert!(
                time > due_time,
                "A refund is not available yet."
            );
            
            let exchange_nft_id: NonFungibleLocalId = non_fungible_data.exchange_nft_id;

            // gets the exchange_nft_id from the receipt NFT (hopefully, may be wrong)

            let exchange_nft_bucket = self.exchange_nfts.take_non_fungible(&exchange_nft_id);

            // takes the appropriate exchange NFT from the exchange_nfts vault

            let exchange_nft_non_fungible_data: ExchangeNFT = exchange_nft_bucket.non_fungible().data();
            
            assert!(
                exchange_nft_non_fungible_data.complete == false,
                "There was a response to your prompt so a refund is unavailable. Please claim your completed NFT."
            );

            // checks that the exchange NFT is incomplete

            let refund_amount: Decimal = exchange_nft_non_fungible_data.price_paid;
            
            let refund_bucket: Bucket = self.xrd_payments.take(refund_amount);
            
            self.internal_nft_mint_badge.authorize(|| {
                receipt.burn();
            });

            self.internal_nft_mint_badge.authorize(|| {
                exchange_nft_bucket.burn();
            });

            // burns the receipt and the incomplete NFT

            refund_bucket

            // returns the completed exchange NFT

        }

        pub fn remove_commission() {
            // todo
        }
    }
}

