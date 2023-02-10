use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct LoanDue {
    pub amount_due: Decimal,
    pub loan_amount: Decimal,
    pub fees: Decimal,
}

blueprint! {
    struct FlashLoanPool {
        loan_fee_rate: Decimal,
        loan_vault: Vault,
        auth_vault: Vault,
        transient_resource_address: ResourceAddress,
    }

    impl FlashLoanPool {
        /// The most elementary possible flash loan.  Creates a loan pool from whatever is initially supplied,
        /// provides loans with a .1% fee, and lets anyone freely add liquidity.
        ///
        /// Does NOT reward liquidity providers in any way or provide a way to remove liquidity from the pool.
        /// Minting LP tokens for rewards, and removing liquidity, is covered in other examples, such as:
        /// https://github.com/radixdlt/scrypto-examples/tree/main/defi/radiswap
        pub fn instantiate_default(pool_resource_addresse: ResourceAddress,loan_fee_rate:Decimal) -> (ComponentAddress,Bucket) {

            assert!(
                loan_fee_rate > dec!(0),
                "Loan interest can't be negative"
            );

            let auth_token = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Admin authority for FlashLoanPool")
                .initial_supply(1);

            // Define a "transient" resource which can never be deposited once created, only burned
            let address = ResourceBuilder::new_non_fungible(NonFungibleIdType::UUID)
                .metadata(
                    "name",
                    "Promise token for BasicFlashLoan - must be returned to be burned!",
                )
                .mintable(rule!(require(auth_token.resource_address())), AccessRule::DenyAll)
                .burnable(rule!(require(auth_token.resource_address())), AccessRule::DenyAll)
                .restrict_deposit(AccessRule::DenyAll, AccessRule::DenyAll)
                .no_initial_supply();


            //  Lock components methods to admin only

            let admin_badge = ResourceBuilder::new_fungible()
            .divisibility(DIVISIBILITY_NONE)
            .metadata("name", "FlashLoanPool admin bage")
            .initial_supply(1);

            let mut  flash_loan_component = Self {
                loan_vault: Vault::new(pool_resource_addresse),
                auth_vault: Vault::with_bucket(auth_token),
                transient_resource_address: address,
                loan_fee_rate,
            }
            .instantiate();

            let access_rules = AccessRules::new()
                .method("add_liquidity", rule!(require(admin_badge.resource_address())), AccessRule::DenyAll)
                .method("remove_liquidity", rule!(require(admin_badge.resource_address())), AccessRule::DenyAll)
                .default(AccessRule::AllowAll, AccessRule::DenyAll);

            flash_loan_component.add_access_check(access_rules);

            (flash_loan_component.globalize(),admin_badge)

        }

        pub fn available_liquidity(&self) -> Decimal {
            self.loan_vault.amount()
        }

        pub fn add_liquidity(&mut self, tokens: Bucket) {
            self.loan_vault.put(tokens);
        }

        pub fn remove_liquidity(&mut self, tokens: Decimal) -> Bucket{
            self.loan_vault.take(tokens)
        }

        pub fn take_loan(&mut self, loan_amount: Decimal) -> (Bucket, Bucket) {
            assert!(
                loan_amount <= self.loan_vault.amount(),
                "Not enough liquidity to supply this loan!"
            );

            // Calculate how much we must be repaid

            let fees = loan_amount * (self.loan_fee_rate);

            // Mint an NFT with the loan terms.  Remember that this resource previously had rules defined which
            // forbid it from ever being deposited in any vault.  Thus, once it is present in the transaction
            // the only way for the TX to complete is to remove this "dangling" resource by burning it.
            //
            // Our component will control the only badge with the authority to burn the resource, so anyone taking
            // a loan must call our repay_loan() method with an appropriate reimbursement, at which point we will
            // burn the NFT and allow the TX to complete.
            let loan_terms = self.auth_vault.authorize(|| {
                borrow_resource_manager!(self.transient_resource_address).mint_non_fungible(
                    &NonFungibleId::random(),
                    LoanDue {
                        amount_due: fees + loan_amount,
                        fees,
                        loan_amount,
                    },
                )
            });
            (self.loan_vault.take(loan_amount), loan_terms)
        }

        pub fn repay_loan(&mut self, mut loan_repayment: Bucket, loan_terms: Bucket) -> Bucket{
            assert!(
                loan_terms.resource_address() == self.transient_resource_address,
                "Incorrect resource passed in for loan terms"
            );

            // Verify we are being sent at least the amount due
            let terms: LoanDue = loan_terms.non_fungible().data();
            assert!(
                loan_repayment.amount() >= terms.amount_due,
                "Insufficient repayment given for your loan!"
            );

            // We could also verify that the resource being repaid is of the correct kind, and give a friendly
            // error message if not. For this example we'll just let the engine handle that when we try to deposit
            self.loan_vault.put(loan_repayment.take( terms.amount_due));

            // We have our payment; we can now burn the transient token
            self.auth_vault.authorize(|| loan_terms.burn());


            // Return the change to the work top
            loan_repayment
        }
    }
}
