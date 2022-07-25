use scrypto::prelude::*;
use crate::liquidity_pool::*;


/// Design question: Not sure if should provide 1 token per LP or have the LP token itself be used to vote.
/// If use LP token to vote, then they can vote different selection i.e allocate portions between Yes and No... but why would they wanna do this?
/// Same problem will occur if provide voting tokens for LP. And if each LP only get 1, then it would be unfair for LP that has most ownership in the pool?

#[derive(TypeId, Encode, Decode, Describe, Debug, PartialEq)]
pub enum Vote {
    Yes,
    No,
    NoWithVeto,
}

#[derive(TypeId, Encode, Decode, Describe, Debug, PartialEq)]
pub enum Resolution {
    Passed,
    Failed,
    InProcess,
}

#[derive(TypeId, Encode, Decode, Describe, Debug, PartialEq)]
pub enum Stage {
    DepositPhase,
    VotingPeriod,
}

#[derive(NonFungibleData, Debug)]
pub struct Proposal {
    pub token_1_weight: Decimal,
    pub token_2_weight: Decimal,
    pub fee_to_pool: Decimal,
    #[scrypto(mutable)]
    pub amount_deposited: Decimal,
    #[scrypto(mutable)]
    pub stage: Stage,
    #[scrypto(mutable)]
    pub yes_counter: Decimal,
    #[scrypto(mutable)]
    pub no_counter: Decimal,
    #[scrypto(mutable)]
    pub no_with_veto_counter: Decimal,
    pub vote_ends_in_epoch: u64,
    pub proposal_end_epoch: u64,
    pub current_epoch: u64,
    #[scrypto(mutable)]
    pub resolution: Resolution,
}

#[derive(NonFungibleData, Debug)]
pub struct VoteBadge {
    proposal: NonFungibleId,
    #[scrypto(mutable)]
    vote: Vote,
    #[scrypto(mutable)]
    vote_weight: Decimal,
    #[scrypto(mutable)]
    lp_tokens_allocated: Decimal,
}

blueprint!{
    /// The `LiquidityPoolDAO` blueprint defines the state and logic involved in the DAO governance process of liquidity pools. This blueprint is 
    /// automatically instantiated when a liquidity pool is created. It has a couple NFTs that faciliates the governance proposal and its voting. 
    /// There are two phases to the governance process. One is the "Deposit Phase" to which LP's will decide which proposal is worth voting for
    /// by depositing XRD to a vault contained within the `LiquidityPoolDAO` component. It is still undetermined how the fees will be used for.
    /// Once a minimum of 500 XRD has been deposited, the proposal can advance to the next stage; the Voting Process.  Proposal NFT's are minted 
    /// and are contained within the `proposal_vault`. Multiple proposals can be created. However, only one can be advanced for the Voting Process
    /// stage. There is a time limit for each stage. Currently, it is set to 5 epochs for the Deposit Phase and 10 epochs for the Voting Process
    /// stage. This is for demonstration purposes only. At least 30% of the LPs will need to vote in order for there to be a resolution. There is also
    /// a `No with veto` threshold of 33.4% that must be maintained or else the proposal will not pass. While `No` votes signlas a disagreement to the 
    /// proposal, a `No with veto` signals a significantly stronger disagreement, where you may feel the proposal is actively harming the governance 
    /// system by abusing or spamming pointless proposals just to send a message. Other than that, a simple majority will suffice to resolve the proposal. 
    struct LiquidityPoolDAO {
        /// This contains the admin badge to which Proposal NFTs and Vote Badge NFTs can be minted, burnt, or updated.
        nft_proposal_admin: Vault,
        /// This is the resource address of the Proposal NFT used primarily to view the NFT data or prove that it belongs to this protocol.
        nft_proposal_address: ResourceAddress,
        /// Likewise, this is the resource address of the Vote Badge NFT for similar purposes.
        vote_badge_address: ResourceAddress,
        /// This is the vault where all the proposals are contained.
        proposal_vault: Vault,
        /// This is the vault where LP Tokens are allocated towards a `Yes` vault.
        voting_yes_vault: Vault,
        /// This is the vault where LP Tokens are allocated towards a `No` vault.
        voting_no_vault: Vault,
        /// This is the vault where LP Tokens are allocated towards a `No with veto` vault.
        voting_no_with_veto_vault: Vault,
        /// This is the time constraint for the Voting Process.
        voting_end_epoch: u64,
        /// This is the minimum amount of votes that need to resolve the proposal.
        vote_quorom: Decimal,
        /// This is the maximum amount of `No with veto` votes that a proposal can have.
        no_with_veto_threshold: Decimal,
        /// This is the minimum XRD required to advance the proposal from the Deposit Phase to the Voting Process stage.
        minimum_xrd: Decimal,
        /// This is where the XRD deposits will be contained. Undetermined yet how this will be used.
        xrd_vault: Vault,
        /// This is the time constraint for the Deposit Phase.
        proposal_end_epoch: u64,
        /// This is where proposal(s) who have advanced to the Voting Process will be contained. Only one can advance to the Voting Process at a time. 
        proposal_in_voting_period: Vec<NonFungibleId>,
        /// This is the component address of the liquidity pool to change the pool parameter if the proposal succeeds.
        liquidity_pool: Option<ComponentAddress>,
    }

    impl LiquidityPoolDAO {
        /// The `LiquidityPoolDAO` component will instantiate along with the liquidity pool. The liquidity pool will provide the LP token resource address
        /// along with the Proposal NFT admin badge in order to mint, burn, or update the NFT data. The liquidity pool will require this badge as well
        /// since if the proposal passes or fails, the Proposal NFT is sent to the liquidity pool to be enacted and burnt, or simply burnt if the proposal
        /// failed.
        /// 
        /// # Arguments:
        /// 
        /// * `tracking_token_address` (ResourceAddress) - The resource address of the LP tokens to check whether the LP tokens belongs to this pool or not.
        /// * `nft_proposal_admin` (Bucket) - The bucket that contains the admin badge to mint, burn, or update the Proposal NFT or Vote Badge NFT.
        /// 
        /// # Returns:
        /// 
        /// * `ComponentAddress` - The component address of the LiquidityPoolDAO.
        pub fn new(
            tracking_token_address: ResourceAddress,
            nft_proposal_admin: Bucket,
        ) -> ComponentAddress
        {
            // The Proposal NFT definition
            let nft_proposal = ResourceBuilder::new_non_fungible()
                .metadata("name", "NFT Proposal")
                .metadata("symbol", "NP")
                .metadata("description", "An NFT that documents governance proposal to the pool")
                .mintable(rule!(require(nft_proposal_admin.resource_address())), LOCKED)
                .burnable(rule!(require(nft_proposal_admin.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(nft_proposal_admin.resource_address())), LOCKED)
                .no_initial_supply();

            // The Vote Badge NFT definition
            let vote_badge = ResourceBuilder::new_non_fungible()
                .metadata("name", "Vote Badge")
                .metadata("symbol", "VB")
                .metadata("description", "An NFT that documents your vote to a governance proposal.")
                .mintable(rule!(require(nft_proposal_admin.resource_address())), LOCKED)
                .burnable(rule!(require(nft_proposal_admin.resource_address())), LOCKED)
                .updateable_non_fungible_data(rule!(require(nft_proposal_admin.resource_address())), LOCKED)
                .no_initial_supply();

            // Creating the liquidity pool DAO component and instantiating it
            let liquidity_pool_dao: ComponentAddress = Self { 
                nft_proposal_admin: Vault::with_bucket(nft_proposal_admin),
                nft_proposal_address: nft_proposal,
                vote_badge_address: vote_badge,
                proposal_vault: Vault::new(nft_proposal),
                voting_yes_vault: Vault::new(tracking_token_address),
                voting_no_vault: Vault::new(tracking_token_address),
                voting_no_with_veto_vault: Vault::new(tracking_token_address),
                voting_end_epoch: 10,
                vote_quorom: dec!("0.3"),
                no_with_veto_threshold: dec!("0.334"),
                minimum_xrd: dec!("500"),
                xrd_vault: Vault::new(RADIX_TOKEN),
                proposal_end_epoch: 5,
                proposal_in_voting_period: Vec::new(),
                liquidity_pool: None,
            }
            .instantiate()
            .globalize();

            return liquidity_pool_dao
        }

        /// Sets the liquidity pool address so that the LiquidityPoolDAO component can call methods from the liquidity pool
        /// 
        /// This method does not perform any checks.
        /// 
        /// This method does not return any asset.
        pub fn set_address(
            &mut self,
            liquidity_pool_address: ComponentAddress,
        )
        {
            self.liquidity_pool.get_or_insert(liquidity_pool_address);
        }

        /// Creates a governance proposal
        /// 
        /// For now the only pool parameter to change are the token weights and the fee to the pool.
        /// 
        /// This method performs a few checks:
        /// 
        /// * **Check 1:** Checks that the weight of both tokens cannot exceed 100%.
        /// * **Check 2:** Checks that the fee must be between 0% and 100%.
        /// 
        /// # Arguments:
        /// 
        /// * `token_1_weight` (Decimal) - The weight desired to change for Token 1.
        /// * `token_2_weight` (Decimal) - The weight desired to change for Token 2.
        /// * `fee_to_pool` (Decimal) - The swap fee desired to change for the pool.
        /// 
        /// # Returns:
        /// 
        /// `NonFungible` - The NonFungibleId of the Proposal NFT used for identification. 
        pub fn propose(
            &mut self,
            token_1_weight: Decimal,
            token_2_weight: Decimal,
            fee_to_pool: Decimal,
        ) -> NonFungibleId
        {
            assert!((token_1_weight + token_2_weight) <= dec!("1.0"), 
                "[Governance Proposal]: The weight of both tokens cannot exceed {:?}.", dec!("1.0")
            );

            assert!(
                (fee_to_pool >= Decimal::zero()) & (fee_to_pool <= dec!("100")), 
                "[Pool Creation]: Fee must be between 0 and 100"
            );

            // Mints the Proposal NFT based on the arguments passed.
            let nft_proposal = self.nft_proposal_admin.authorize(|| {
                borrow_resource_manager!(self.nft_proposal_address)
                .mint_non_fungible(
                    &NonFungibleId::random(),
                    Proposal {
                        token_1_weight: token_1_weight,
                        token_2_weight: token_2_weight,
                        fee_to_pool: fee_to_pool,
                        amount_deposited: Decimal::zero(),
                        yes_counter: Decimal::zero(),
                        no_counter: Decimal::zero(),
                        no_with_veto_counter: Decimal::zero(),
                        stage: Stage::DepositPhase,
                        vote_ends_in_epoch: self.voting_end_epoch,
                        proposal_end_epoch: self.proposal_end_epoch,
                        current_epoch: Runtime::current_epoch(),
                        resolution: Resolution::InProcess,
                    },
                )
            });

            // Calls the liquidity pool component to allow liquidity pool visibility that the Proposal NFT has been created.
            self.push_nft_proposal_address();

            // Retrieves the LiquidityPool component.
            let liquidity_pool: LiquidityPool = self.liquidity_pool.unwrap().into();

            // Retrieves LiquidityPool component data.
            let liquidity_pool_token_1_weight = liquidity_pool.token_1_weight();
            let liquidity_pool_token_2_weight = liquidity_pool.token_2_weight();
            let liquidity_pool_fee_to_pool = liquidity_pool.fee_to_pool();

            // Retrieves the Proposal NFT ID.
            let proposal_id = nft_proposal.non_fungible::<Proposal>().id();

            info!("[Governance Proposal]: Proposal has has been created!");
            info!("[Governance Proposal]: The proposal ID is {:?}", proposal_id);
            info!("[Governance Proposal]: Token 1 weight adjustment to {:?} from {:?}",
                token_1_weight, liquidity_pool_token_1_weight);
            info!("[Governance Proposal]: Token 2 weight adjustment to {:?} from {:?}",
                token_2_weight, liquidity_pool_token_2_weight);
            info!("[Governance Proposal]: Pool fee adjustment to {:?} from {:?}",
                fee_to_pool, liquidity_pool_fee_to_pool);   
            info!("[Governance Proposal]: The conditions to advance this proposal to the Voting Period are as follows:"); 
            info!("[Governance Proposal]: Condition 1 - A minimum of {:?} in XRD must be deposited.", 
                self.minimum_xrd);
            info!("[Governance Proposal]: The voting process will end in {:?} epoch.", self.proposal_end_epoch);

            // Puts the Proposal NFT ID in its respective vault.
            self.proposal_vault.put(nft_proposal);

            return proposal_id
        }

        /// This method is used to deposit XRD towards the Proposal NFT in the Deposit Phase.
        /// 
        /// This method performs a few checks:
        /// 
        /// * **Check 1:** Checks that the proposal exists.
        /// * **Check 2:** Checks that the bucket passed contains XRD.
        /// * **Check 3:** Checks that the user has not deposited more than what is required to advance the proposal.
        /// * **Check 4:** Checks that no more than one Proposal NFT can be advanced to the Voting Process.
        /// 
        /// # Arguments:
        /// 
        /// * `fee` (Bucket) - The bucket that contains the XRD to be deposited towards the Proposal NFT.
        /// * `proposal_id` (NonFungibleId) - The NonFungibleId of the proposal selected.
        /// 
        /// This method does not return any assets. 
        pub fn deposit_proposal(
            &mut self,
            fee: Bucket,
            proposal_id: NonFungibleId,
        )
        {   
            assert_eq!(self.proposal_vault.non_fungible_ids().contains(&proposal_id), true, 
                "The proposal you are voting for does not exist."
            );

            assert_eq!(fee.resource_address(), self.xrd_vault.resource_address(), 
                "Incorrect asset deposited."
            );

            let amount = fee.amount();

            // Retrieves resource manager.
            // We use the resource manager to update the Proposal data NFT because it's a common
            // occurence to forget to put the NFT back into the vault after it's been updated.
            let resource_manager = borrow_resource_manager!(self.nft_proposal_address);
            let mut proposal_data: Proposal = resource_manager.get_non_fungible_data(&proposal_id);

            proposal_data.amount_deposited += amount;

            // Since proposal_data.amount_deposited value has been used, we shall create another one.
            // We also want to have this after the amount has been changed so we can retrieve the updated data.
            let amount_deposited = proposal_data.amount_deposited;

            assert!(amount_deposited <= self.minimum_xrd, 
                "You have deposited {:?} more XRD than what is required for this proposal.",
                (amount_deposited - self.minimum_xrd)
            );

            // Authorize update of the Proposal NFT.
            self.nft_proposal_admin.authorize(|| 
                resource_manager.update_non_fungible_data(&proposal_id, proposal_data)
            );

            self.xrd_vault.put(fee);

            // Retrieves the Proposal NFT data again since the previous value has been consumed.
            let proposal_data: Proposal = resource_manager.get_non_fungible_data(&proposal_id);

            if proposal_data.amount_deposited >= self.minimum_xrd {

                // Retrieves the Proposal NFT data again since the previous value has been consumed.
                let mut proposal_data: Proposal = resource_manager.get_non_fungible_data(&proposal_id);

                proposal_data.stage = Stage::VotingPeriod;

                // Authorize update of the Proposal NFT.
                self.nft_proposal_admin.authorize(|| 
                    resource_manager.update_non_fungible_data(&proposal_id, proposal_data)
                );

                // This helper function is used to ensure that no more than one Proposal NFT has advanced to the Voting Process.
                self.check_voting_stage();

                // Pushes the Proposal NFT that has advanced to the Voting Prorcess.
                self.proposal_in_voting_period.push(proposal_id);

                assert_eq!(self.proposal_in_voting_period.len(), 1, "Cannot have more than one proposal in the voting period.");

                info!("[Depositing Proposal]: This proposal has now advanced to the Voting Period phase!"); 
                info!("[Depositing Proposal]: The conditions to pass this proposal are as follows:"); 
                info!("[Depositing Proposal]: Condition 1 - Simple majority will need to vote 'Yes' in order to pass this proposal");
                info!("[Depositing Proposal]: Condition 2 - A minimum of {:?} of the protocol's voting power (quorum) is required.",
                    self.vote_quorom
                );
                info!("[Depositing Proposal]: Condition 3 - Less than {:?} of participating voting power votes 'no-with-veto'.",
                    self.no_with_veto_threshold
                );
            
            } else if Runtime::current_epoch() > self.proposal_end_epoch {

                // Here we take the NFT out of the vault because there is no other option other than
                // to burn the NFT.
                let proposal = self.proposal_vault.take_non_fungible(&proposal_id);

                info!("[Depositing Proposal]: The Deposit Phase has elapsed and this proposal is rejected.");
                info!("[Depositing Proposal]: Create a new proposal to start another process."); 

                self.nft_proposal_admin.authorize(|| proposal.burn());
                
            } else {

                let amount_deposited = proposal_data.amount_deposited;

                info!("[Depositing Proposal]: You have deposited {:?} XRD to this proposal.",
                    amount
                );

                info!("[Depositing Proposal]: This proposal has {:?} XRD deposited.", 
                    amount_deposited
                ); 

                info!("[Depositing Proposal]: This proposal require(s) additional {:?} XRD to advance this proposal to the Voting Period.", 
                    (self.minimum_xrd - amount_deposited)
                ); 
            };
        }

        fn check_voting_stage(
            &self,
        )
        {
            let proposals = self.proposal_vault.non_fungible_ids();
            let all_proposals = proposals.iter();
            for proposal in all_proposals {
                let resource_manager = borrow_resource_manager!(self.nft_proposal_address);
                let proposal_data: Proposal = resource_manager.get_non_fungible_data(&proposal);
                let proposal_stage = proposal_data.stage;
                if proposal_stage == Stage::VotingPeriod {
                    let proposals_in_voting_period = vec![proposal];
                    assert_eq!(proposals_in_voting_period.len(), 1, "Cannot have more than one proposal in the voting period.");
                }
            }
        }

        pub fn check_proposal(
            &self,
            proposal_id: NonFungibleId
        )
        {
            let resource_manager = borrow_resource_manager!(self.nft_proposal_address);
            let proposal_data: Proposal = resource_manager.get_non_fungible_data(&proposal_id);
            info!("[Proposal Info]: Token 1 weight: {:?}", proposal_data.token_1_weight);
            info!("[Proposal Info]: Token 2 weight: {:?}", proposal_data.token_2_weight);
            info!("[Proposal Info]: Fee to Pool: {:?}", proposal_data.fee_to_pool);
            info!("[Proposal Info]: Amount deposited to this proposal: {:?}", proposal_data.amount_deposited);
            info!("[Proposal Info]: Yes: {:?}", proposal_data.yes_counter);
            info!("[Proposal Info]: No: {:?}", proposal_data.no_counter);
            info!("[Proposal Info]: No with veto: {:?}", proposal_data.no_with_veto_counter);
            info!("[Proposal Info]: Current stage of the proposal: {:?}", proposal_data.stage);
            info!("[Proposal Info]: Voting period ends in: {:?} epoch", proposal_data.vote_ends_in_epoch);
            info!("[Proposal Info]: Deposit Phase period ends in: {:?} epoch", proposal_data.proposal_end_epoch);
            info!("[Proposal Info]: Current epoch: {:?}", proposal_data.current_epoch);
            info!("[Proposal Info]: Resolution: {:?}", proposal_data.resolution);
        }
        // Test function
        pub fn see_voting(
            &self,
        ) -> Vec<NonFungibleId>
        {
            return self.proposal_in_voting_period.clone()
        }

        pub fn vote_proposal(
            &mut self,
            lp_tokens: Bucket,
            vote_submission: Vote,
            proposal_id: NonFungibleId,
        ) -> Bucket
        {
            let liquidity_pool: LiquidityPool = self.liquidity_pool.unwrap().into();
            let tracking_token_address = liquidity_pool.tracking_token_address();

            assert_eq!(self.proposal_vault.non_fungible_ids().contains(&proposal_id), true, 
                "The proposal you are voting for does not exist."
            );

            assert_eq!(lp_tokens.resource_address(), tracking_token_address,
                "Wrong LP tokens passed."
            );

            let resource_manager = borrow_resource_manager!(self.nft_proposal_address);
            let mut proposal_data: Proposal = resource_manager.get_non_fungible_data(&proposal_id);

            assert_eq!(proposal_data.stage, Stage::VotingPeriod, 
                "[Voting]: The proposal has not advanced to the Voting Period, yet.
                This proposal requires an additional deposit of {:?} in order to advance.",
                (self.minimum_xrd - proposal_data.amount_deposited)
            );

            assert_eq!(self.proposal_in_voting_period.contains(&proposal_id), true,
                "[Voting]: The proposal has not advanced to the Voting Period, yet."
            );
            
            // Get amount of voting tokens
            let amount = lp_tokens.amount();

            // Calculate vote weight.
            let tracking_tokens_manager: &ResourceManager = borrow_resource_manager!(tracking_token_address);
            let vote_weight: Decimal = amount / tracking_tokens_manager.total_supply();

            match vote_submission {
                Vote::Yes => {
                    self.voting_yes_vault.put(lp_tokens);

                    proposal_data.yes_counter += vote_weight;

                    self.nft_proposal_admin.authorize(|| 
                        resource_manager.update_non_fungible_data(&proposal_id, proposal_data)
                    );

                    let proposal_data: Proposal = resource_manager.get_non_fungible_data(&proposal_id);

                    let yes_counter = proposal_data.yes_counter;
                    let no_counter = proposal_data.no_counter;
                    let no_with_veto_counter = proposal_data.no_with_veto_counter;

                    let abstain_counter = dec!("1.0") - (yes_counter + no_counter);
                    info!("[Voting]: The current count for the vote is: Yes - {:?} | No - {:?} | No with veto - {:?} | Abstain - {:?}",
                    yes_counter, no_counter, no_with_veto_counter, abstain_counter);

                    info!("[Voting]: You have voted 'Yes' for the proposal.");
                    info!("[Voting]: The weight of your vote is {:?}.", vote_weight);
                    let vote_badge = self.nft_proposal_admin.authorize(|| {
                        borrow_resource_manager!(self.vote_badge_address)
                        .mint_non_fungible(
                            &NonFungibleId::random(),
                            VoteBadge {
                                proposal: proposal_id,
                                vote: Vote::Yes,
                                vote_weight: vote_weight,
                                lp_tokens_allocated: amount,
                            },
                        )
                    });

                    info!("[Voting]: You've received a badge for your vote!");
                    info!("[Voting]: Your badge proof is {:?}", vote_badge.resource_address());
                    return vote_badge
                }
                Vote::No => {
                    self.voting_no_vault.put(lp_tokens);

                    proposal_data.no_counter += vote_weight;

                    self.nft_proposal_admin.authorize(|| 
                        resource_manager.update_non_fungible_data(&proposal_id, proposal_data)
                    );

                    let proposal_data: Proposal = resource_manager.get_non_fungible_data(&proposal_id);

                    let yes_counter = proposal_data.yes_counter;
                    let no_counter = proposal_data.no_counter;
                    let no_with_veto_counter = proposal_data.no_with_veto_counter;

                    let abstain_counter = dec!("1.0") - (yes_counter + no_counter);
                    info!("[Voting]: The current count for the vote is: Yes - {:?} | No - {:?} | No with veto - {:?} | Abstain - {:?}",
                    yes_counter, no_counter, no_with_veto_counter, abstain_counter);

                    info!("[Voting]: You have voted 'No' for the proposal.");
                    info!("[Voting]: The weight of your vote is {:?}.", vote_weight);
                    let vote_badge = self.nft_proposal_admin.authorize(|| {
                        borrow_resource_manager!(self.vote_badge_address)
                        .mint_non_fungible(
                            &NonFungibleId::random(),
                            VoteBadge {
                                proposal: proposal_id,
                                vote: Vote::No,
                                vote_weight: vote_weight,
                                lp_tokens_allocated: amount,
                            },
                        )
                    });

                    info!("[Voting]: You've received a badge for your vote!");
                    info!("[Voting]: Your badge proof is {:?}", vote_badge.resource_address());
                    return vote_badge
                }
                Vote::NoWithVeto => {
                    self.voting_no_with_veto_vault.put(lp_tokens);

                    proposal_data.no_with_veto_counter += vote_weight;

                    self.nft_proposal_admin.authorize(|| 
                        resource_manager.update_non_fungible_data(&proposal_id, proposal_data)
                    );

                    let proposal_data: Proposal = resource_manager.get_non_fungible_data(&proposal_id);
                    
                    let yes_counter = proposal_data.yes_counter;
                    let no_counter = proposal_data.no_counter;
                    let no_with_veto_counter = proposal_data.no_with_veto_counter;
    
                    let abstain_counter = dec!("1.0") - (yes_counter + no_counter);
                    info!("[Voting]: The current count for the vote is: Yes - {:?} | No - {:?} | No with veto - {:?} | Abstain - {:?}",
                    yes_counter, no_counter, no_with_veto_counter, abstain_counter);

                    info!("[Voting]: You have voted 'No' for the proposal.");
                    info!("[Voting]: The weight of your vote is {:?}.", vote_weight);
                    let vote_badge = self.nft_proposal_admin.authorize(|| {
                        borrow_resource_manager!(self.vote_badge_address)
                        .mint_non_fungible(
                            &NonFungibleId::random(),
                            VoteBadge {
                                proposal: proposal_id,
                                vote: Vote::NoWithVeto,
                                vote_weight: vote_weight,
                                lp_tokens_allocated: amount,
                            },
                        )
                    });

                    info!("[Voting]: You've received a badge for your vote!");
                    info!("[Voting]: Your badge proof is {:?}", vote_badge.resource_address());
                    return vote_badge
                }
            };
        }

        pub fn check_vote(
            &self,
            vote_badge: Proof,
        )
        {
            assert_eq!(vote_badge.resource_address(), self.vote_badge_address, 
                "Incorrect proof provided."
            );
            
            let vote_badge_data = vote_badge.non_fungible::<VoteBadge>().data();
            let proposal = vote_badge_data.proposal;
            let vote = vote_badge_data.vote;
            let vote_weight = vote_badge_data.vote_weight;
            let lp_tokens_allocated = vote_badge_data.lp_tokens_allocated;
            info!("[Vote Info]: Proposal: {:?}", proposal);
            info!("[Vote Info]: Vote: {:?}", vote);
            info!("[Vote Info]: Vote weight: {:?}", vote_weight);
            info!("[Vote Info]: LP tokens allocated: {:?}", lp_tokens_allocated);
        }

        pub fn recast_vote(
            &mut self,
            vote_badge: Proof,
            vote_submission: Vote,
        )
        {
            assert_eq!(vote_badge.resource_address(), self.vote_badge_address, 
                "Incorrect proof provided."
            );

            // Retreives Vote Badge NFT data.
            let vote_badge_data = vote_badge.non_fungible::<VoteBadge>().data();

            let proposal_id = vote_badge_data.proposal;
            let vote = vote_badge_data.vote;
            let vote_weight = vote_badge_data.vote_weight;

            assert_ne!(vote, vote_submission, "You have already voted {:?}.", 
                vote_submission
            );

            // Retrieves Proposal NFT Data again due to value being consumed.
            let resource_manager = borrow_resource_manager!(self.nft_proposal_address);

            let lp_tokens = self.reallocate_lp_token(&vote_badge);

            // Evaluates the new vote and adds allocation based on matched criteria.
            // Data manipulation occurs for both Proposal NFT data and Vote Badge data.
            match vote_submission {
                Vote::Yes => {
                    self.voting_yes_vault.put(lp_tokens);

                    let mut proposal_data: Proposal = resource_manager.get_non_fungible_data(&proposal_id);

                    proposal_data.yes_counter += vote_weight;

                    info!("[Vote Recast]: You have voted 'Yes' for the proposal.");
                    info!("[Vote Recast]: The weight of your vote is {:?}.", vote_weight);

                    // Logic for changing VoteBadge
                    let mut vote_badge_data = vote_badge.non_fungible::<VoteBadge>().data();
                    vote_badge_data.vote = Vote::Yes;

                    self.nft_proposal_admin.authorize(||
                        vote_badge.non_fungible().update_data(vote_badge_data)
                    );
                    self.nft_proposal_admin.authorize(|| 
                        resource_manager.update_non_fungible_data(&proposal_id, proposal_data)
                    );

                    let proposal_data: Proposal = resource_manager.get_non_fungible_data(&proposal_id);

                    let yes_counter = proposal_data.yes_counter;
                    let no_counter = proposal_data.no_counter;
                    let no_with_veto_counter = proposal_data.no_with_veto_counter;
    
                    let abstain_counter = dec!("1.0") - (yes_counter + no_counter);
                    info!("[Vote Recast]: The current count for the vote is: Yes - {:?} | No - {:?} | No with veto - {:?} | Abstain - {:?}",
                    yes_counter, no_counter, no_with_veto_counter, abstain_counter);
                }
                Vote::No => {
                    self.voting_no_vault.put(lp_tokens);

                    let mut proposal_data: Proposal = resource_manager.get_non_fungible_data(&proposal_id);

                    proposal_data.no_counter += vote_weight;

                    info!("[Vote Recast]: You have voted 'No' for the proposal.");
                    info!("[Vote Recast]: The weight of your vote is {:?}.", vote_weight);

                    // Logic for changing VoteBadge
                    let mut vote_badge_data = vote_badge.non_fungible::<VoteBadge>().data();
                    vote_badge_data.vote = Vote::No;

                    self.nft_proposal_admin.authorize(||
                        vote_badge.non_fungible().update_data(vote_badge_data)
                    );
                    self.nft_proposal_admin.authorize(|| 
                        resource_manager.update_non_fungible_data(&proposal_id, proposal_data)
                    );

                    let proposal_data: Proposal = resource_manager.get_non_fungible_data(&proposal_id);

                    let yes_counter = proposal_data.yes_counter;
                    let no_counter = proposal_data.no_counter;
                    let no_with_veto_counter = proposal_data.no_with_veto_counter;
    
                    let abstain_counter = dec!("1.0") - (yes_counter + no_counter);
                    info!("[Vote Recast]: The current count for the vote is: Yes - {:?} | No - {:?} | No with veto - {:?} | Abstain - {:?}",
                    yes_counter, no_counter, no_with_veto_counter, abstain_counter);
                }
                Vote::NoWithVeto => {
                    self.voting_no_with_veto_vault.put(lp_tokens);

                    let mut proposal_data: Proposal = resource_manager.get_non_fungible_data(&proposal_id);

                    proposal_data.no_with_veto_counter += vote_weight;

                    info!("[Vote Recast]: You have voted 'No' for the proposal.");
                    info!("[Vote Recast]: The weight of your vote is {:?}.", vote_weight);
      
                    // Logic for changing VoteBadge
                    let mut vote_badge_data = vote_badge.non_fungible::<VoteBadge>().data();
                    vote_badge_data.vote = Vote::NoWithVeto;

                    self.nft_proposal_admin.authorize(||
                        vote_badge.non_fungible().update_data(vote_badge_data)
                    );
                    self.nft_proposal_admin.authorize(|| 
                        resource_manager.update_non_fungible_data(&proposal_id, proposal_data)
                    );

                    let proposal_data: Proposal = resource_manager.get_non_fungible_data(&proposal_id);

                    let yes_counter = proposal_data.yes_counter;
                    let no_counter = proposal_data.no_counter;
                    let no_with_veto_counter = proposal_data.no_with_veto_counter;
    
                    let abstain_counter = dec!("1.0") - (yes_counter + no_counter);
                    info!("[Vote Recast]: The current count for the vote is: Yes - {:?} | No - {:?} | No with veto - {:?} | Abstain - {:?}",
                    yes_counter, no_counter, no_with_veto_counter, abstain_counter);
    
                    let abstain_counter = dec!("1.0") - (yes_counter + no_counter);
                    info!("[Vote Recast]: The current count for the vote is: Yes - {:?} | No - {:?} | No with veto - {:?} | Abstain - {:?}",
                    yes_counter, no_counter, no_with_veto_counter, abstain_counter);
                }
            }
        }

        fn reallocate_lp_token(
            &mut self,
            vote_badge: &Proof
        ) -> Bucket
        {
            // Retreives Vote Badge NFT data.
            let vote_badge_data = vote_badge.non_fungible::<VoteBadge>().data();

            let vote = vote_badge_data.vote;
            let proposal_id = vote_badge_data.proposal;
            let vote_weight = vote_badge_data.vote_weight;
            let lp_tokens_allocated = vote_badge_data.lp_tokens_allocated;

            // Retrieves Proposal NFT data.
            let resource_manager = borrow_resource_manager!(self.nft_proposal_address);
            let mut proposal_data: Proposal = resource_manager.get_non_fungible_data(&proposal_id);

            // Evaluates the current vote from Vote Badge NFT data and removes allocation based on matched criteria.
            // Data manipulation only happens with Proposal NFT data. 
            let lp_tokens: Bucket = match vote {
                Vote::Yes => {
                    assert!(
                        self.voting_yes_vault.amount() >= lp_tokens_allocated,
                        "[Withdraw]: Vault only has {:?} to withdraw.",
                        self.voting_yes_vault.amount()
                    );

                    let lp_tokens: Bucket = self.voting_yes_vault.take(lp_tokens_allocated);

                    proposal_data.yes_counter -= vote_weight;

                    self.nft_proposal_admin.authorize(|| 
                        resource_manager.update_non_fungible_data(&proposal_id, proposal_data)
                    );

                    info!("[Vote Recast]: Your allocations towards your 'Yes' cast has been decreased by {:?}.", 
                        lp_tokens_allocated);
                    info!("[Vote Recast]: Your weighted vote towards 'Yes' cast has been decreased by {:?}",
                        vote_weight);

                    lp_tokens
                }
                Vote::No => {
                    assert!(
                        self.voting_no_vault.amount() >= lp_tokens_allocated,
                        "[Withdraw]: Vault only has {:?} to withdraw.",
                        self.voting_no_vault.amount()
                    );

                    let lp_tokens: Bucket = self.voting_no_vault.take(lp_tokens_allocated);

                    proposal_data.no_counter -= vote_weight;

                    self.nft_proposal_admin.authorize(|| 
                        resource_manager.update_non_fungible_data(&proposal_id, proposal_data)
                    );

                    info!("[Vote Recast]: Your allocations towards your 'No' cast has been decreased by {:?}.", 
                        lp_tokens_allocated);
                    info!("[Vote Recast]: Your weighted vote towards 'No' cast has been decreased by {:?}",
                        vote_weight);

                    lp_tokens
                }
                Vote::NoWithVeto => {
                    assert!(
                        self.voting_no_with_veto_vault.amount() >= lp_tokens_allocated,
                        "[Withdraw]: Vault only has {:?} to withdraw.",
                        self.voting_no_with_veto_vault.amount()
                    );

                    let lp_tokens: Bucket = self.voting_no_with_veto_vault.take(lp_tokens_allocated);

                    proposal_data.no_with_veto_counter -= vote_weight;

                    self.nft_proposal_admin.authorize(|| 
                        resource_manager.update_non_fungible_data(&proposal_id, proposal_data)
                    );

                    info!("[Vote Recast]: Your allocations towards your 'No with veto' cast has been decreased by {:?}.", 
                        lp_tokens_allocated);
                    info!("[Vote Recast]: Your weighted vote towards 'No with' cast has been decreased by {:?}",
                        vote_weight);

                    lp_tokens
                }
            };

            return lp_tokens
        }

        pub fn vault_amounts(
            &self,
        )
        {
            info!("Yes: {:?}", self.voting_yes_vault.amount());
            info!("No: {:?}", self.voting_no_vault.amount());
            info!("No with veto: {:?}", self.voting_no_with_veto_vault.amount());
        }

        pub fn get_tracking_tokens_supply(
            &self,
        ) -> Decimal
        {
            let liquidity_pool: LiquidityPool = self.liquidity_pool.unwrap().into();
            let tracking_token_address = liquidity_pool.tracking_token_address();
            let tracking_tokens_manager: &ResourceManager = borrow_resource_manager!(tracking_token_address);
            let tracking_amount: Decimal = tracking_tokens_manager.total_supply(); 
            return tracking_amount
        }

        /// Checks if a finish condition is reached. Sends funds if such a condition is reached
        pub fn resolve_proposal(
            &mut self
        )
        {
            let yes_vote = self.voting_yes_vault.amount();
            let no_vote = self.voting_no_vault.amount();
            let no_with_veto_vote = self.voting_no_with_veto_vault.amount();
            let participating_vote =  yes_vote + no_vote + no_with_veto_vote;
            let yes_counter = yes_vote / participating_vote;
            let no_counter = no_vote / participating_vote;
            let no_with_veto_counter = no_with_veto_vote / participating_vote;
            let liquidity_pool: LiquidityPool = self.liquidity_pool.unwrap().into();
            let tracking_token_address = liquidity_pool.tracking_token_address();
            let tracking_tokens_manager: &ResourceManager = borrow_resource_manager!(tracking_token_address);
            let tracking_amount: Decimal = tracking_tokens_manager.total_supply();
            let vote_threshold = participating_vote / tracking_amount;

            if yes_counter > no_counter && vote_threshold >= self.vote_quorom && no_with_veto_counter < self.no_with_veto_threshold {
                
                let proposal_id = self.proposal_in_voting_period.pop().unwrap();
                let proposal = self.proposal_vault.take_non_fungible(&proposal_id);
                let mut proposal_data = proposal.non_fungible::<Proposal>().data();
                
                proposal_data.resolution = Resolution::Passed;
                
                self.nft_proposal_admin.authorize(|| 
                    proposal.non_fungible().update_data(proposal_data)
                );

                let proposal_data = proposal.non_fungible::<Proposal>().data();
                
                info!("[Proposal Resolution]: The proposal {:?} has passed!", proposal_id);
                info!("[Proposal Resolution]: 'No' count ({:?}) has the simple majority against the 'Yes' count ({:?}).",
                    yes_counter, no_counter
                );
                info!("[Proposal Resolution]: The pool paramaters has now changed to:");
                info!("[Proposal Resolution]: Token 1 weight: {:?} | Token 2 weight: {:?} | Fee to pool: {:?}",
                    proposal_data.token_1_weight, proposal_data.token_2_weight, proposal_data.fee_to_pool
                );
                
                liquidity_pool.resolve_proposal(proposal);

            } else if no_counter > yes_counter && vote_threshold >= self.vote_quorom {
                let proposal_id = self.proposal_in_voting_period.pop().unwrap();
                let proposal = self.proposal_vault.take_non_fungible(&proposal_id);
                let mut proposal_data = proposal.non_fungible::<Proposal>().data();
                
                proposal_data.resolution = Resolution::Failed;
                
                self.nft_proposal_admin.authorize(|| 
                    proposal.non_fungible().update_data(proposal_data)
                );
                
                info!("[Proposal Resolution]: The proposal {:?} has failed!", proposal_id);
                info!("[Proposal Resolution]: 'No' count ({:?}) has the simple majority against the 'Yes' count ({:?}).",
                    no_counter, yes_counter
                );

                liquidity_pool.resolve_proposal(proposal);

            } else if no_with_veto_counter >= self.no_with_veto_threshold {

                let proposal_id = self.proposal_in_voting_period.pop().unwrap();
                let proposal = self.proposal_vault.take_non_fungible(&proposal_id);
                let mut proposal_data = proposal.non_fungible::<Proposal>().data();
                
                proposal_data.resolution = Resolution::Failed;
                
                self.nft_proposal_admin.authorize(|| 
                    proposal.non_fungible().update_data(proposal_data)
                );
                
                info!("[Proposal Resolution]: The proposal {:?} has failed!", proposal_id);
                info!("[Proposal Resolution]: 'No with veto' count ({:?}) has surpassed the threshold ({:?}).",
                    no_with_veto_counter, self.no_with_veto_threshold
                );

                liquidity_pool.resolve_proposal(proposal);

            } else if Runtime::current_epoch() > self.voting_end_epoch {

                let proposal_id = self.proposal_in_voting_period.pop().unwrap();
                let proposal = self.proposal_vault.take_non_fungible(&proposal_id);
                let mut proposal_data = proposal.non_fungible::<Proposal>().data();
                
                proposal_data.resolution = Resolution::Failed;
                
                self.nft_proposal_admin.authorize(|| 
                    proposal.non_fungible().update_data(proposal_data)
                );
                
                liquidity_pool.resolve_proposal(proposal);

                info!("[Proposal Resolution]: The proposal {:?} has run out of time and failed!", proposal_id);
            }

            info!("[Proposal Resolution]: No end condition reached");
        }

        pub fn retrieve_lp_tokens(
            &mut self,
            vote_badge: Bucket
        ) -> Bucket
        {
            assert_eq!(vote_badge.resource_address(), self.vote_badge_address, "Incorrect badge");

            let vote_badge_data = vote_badge.non_fungible::<VoteBadge>().data();
            let amount = vote_badge_data.lp_tokens_allocated;
            let vote = vote_badge_data.vote;

            match vote {
                Vote::Yes => {
                    let return_lp: Bucket = self.voting_yes_vault.take(amount);
                    self.nft_proposal_admin.authorize(|| vote_badge.burn());
                    return_lp
                }
                Vote::No => {
                    let return_lp: Bucket = self.voting_no_vault.take(amount);
                    self.nft_proposal_admin.authorize(|| vote_badge.burn());
                    return_lp
                }
                Vote::NoWithVeto => {
                    let return_lp: Bucket = self.voting_no_with_veto_vault.take(amount);
                    self.nft_proposal_admin.authorize(|| vote_badge.burn());
                    return_lp
                }
            }
        }

        pub fn push_nft_proposal_address(
            &self
        )
        {
            let liquidity_pool: LiquidityPool = self.liquidity_pool.unwrap().into();
            liquidity_pool.push_nft_proposal_address(self.nft_proposal_address);
        }
    }
}