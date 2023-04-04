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
    due_time: u64,

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
            .method("respond", rule!(require(account.resource_address())), LOCKED)
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

                // ***better to put the account address in the account NFT???***

                
        }

        pub fn prompt(&mut self, payment: Bucket, prompt: String) -> (Bucket, Bucket) {

            // Accepts a 'prompt' string together with a payment in XRD (from anyone)
            // Mints a partial 'exchange' NFT and 
            // Returns a bucket containing a 'receipt' NFT and a bucket with any change.

            let time: u64 = Runtime::current_epoch();

            // gets the current time

            if time >= self.thread_time {
                let start_time: u64 = time;
            } else {
                let start_time: u64 = self.thread_time;
            };

            // sets start_time at current time if it's later than the thread_time or
            // sets start_time as the thread_time
            
            let end_time: u64 = start_time + 24u64;

            // sets end_time to ~24h after start_time

            let required_payment_exponent = ((end_time - time) / 24 ).ceil() -1;
            
            // pseudocode

            // calcutates the exponent for the required payment. Rounding it up ensures that
            // the required_payment will depend on the number of pending 'exchange' NFTs

            // ***will need to change its type to use in required_payment calc?***

            let required_payment: Decimal = 20 * (2^required_payment_exponent); // pseudocode

            // calculates the required_payment by multiplying 20 by the required payment
            // multiplier which is calculated by 2^required_payment_exponent

            self.xrd_payments.put(payment.take(required_payment));

            // OR PANIC WITH MESSAGE INDICATING REQUIRED PAYMENT
 
            let new_pending_prompt = ExchangeNFT {
                prompt: prompt,
                price_paid: required_payment,
                start_time: start_time,
                end_time: end_time,
                response: "",// Empty String or omit field ??!?
                complete: false,
            };

            let pending_prompt_bucket = self.internal_nft_mint_badge.authorize(|| {
                borrow_resource_manager!(self.exchange_nfts.resource_address).mint_uuid_non_fungible(
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

            let mut thread_time = end_time; // pseudocode

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

            // takes the appropriate exchange NFT from the exchange_nfts vault
            //
            // note: should first assert that the NFT is actually there. Find out how to compare the
            // nft_id argument to a BTreeSet of the NonFungibleIds contained in the vault.

            let mut non_fungible_data: ExchangeNFT = exchange_nft_bucket.non_fungible().data();
            non_fungible_data.response = response;
            non_fungible_data.complete = true;
            
            self.internal_nft_mint_badge
            .authorize(|| exchange_nft_bucket.non_fungible().update_data(non_fungible_data));

            // appends the response to the 'exchange NFT' and marks it as complete

            let payment_amount: Decimal = exchange_nft_bucket.ExchangeNFT.price_paid * 0.9;

            let payment_bucket: Bucket = self.xrd_payments.take(payment_amount);

            // calculates the payment to return, minus 10% commission

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

            let due_time: u64 = receipt.ReceiptNFT.due_time;

            assert!(
                time > due_time,
                "This NFT is not claimable yet."
            );
            
            let exchange_nft_id: NonFungibleLocalId = receipt.ReceiptNFT.exchange_nft_id;

            // gets the exchange_nft_id from the receipt NFT (hopefully, may be wrong)

            let exchange_nft_bucket = self.exchange_nfts.take_non_fungible(&exchange_nft_id);

            // takes the appropriate exchange NFT from the exchange_nfts vault
            //
            // note that there's no check on whether it's in the vault because the existence of
            // the receipt means that it is

            assert!(
                exchange_nft_bucket.ExchangeNFT.complete == true,
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

            let due_time: u64 = receipt.ReceiptNFT.due_time;

            assert!(
                time > due_time,
                "A refund is not available yet."
            );
            
            let exchange_nft_id: NonFungibleLocalId = receipt.ReceiptNFT.exchange_nft_id;

            // gets the exchange_nft_id from the receipt NFT (hopefully, may be wrong)

            let exchange_nft_bucket = self.exchange_nfts.take_non_fungible(&exchange_nft_id);

            // takes the appropriate exchange NFT from the exchange_nfts vault
            
            assert!(
                exchange_nft_bucket.ExchangeNFT.complete == false,
                "There was a response to your prompt so a refund is unavailable. Please claim your completed NFT."
            );

            // checks that the exchange NFT is incomplete

            let refund_amount: Decimal = exchange_nft_bucket.ExchangeNFT.price_paid;
            
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

