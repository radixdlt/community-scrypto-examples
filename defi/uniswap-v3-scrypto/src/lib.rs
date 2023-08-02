use scrypto::prelude::*;

// @to-do - by commenting out bitmap (below line) the code will build perfectly but using it giving error
mod bitmap;

#[derive(ScryptoSbor)]
struct Slot0 {
    // the current price
    sqrt_price_x96: u128,
    // the current tick
    tick: i32,
}

#[derive(ScryptoSbor)]
struct TicksInfo {
    initialized: bool,
    liquidity: u128,
}

#[derive(ScryptoSbor)]
struct PositionsInfo {
    liquidity: u128,
}

#[derive(ScryptoSbor)]
struct Tick {
    min_tick: i32,
    max_tick: i32,
}

pub const MIN_TICK: i32 = -887272;
pub const MAX_TICK: i32 = -MIN_TICK;

#[blueprint]
mod radiswapv3 {

    struct RadiswapV3 {
        // Pool Tokens
        token0: Vault,
        token1: Vault,
        // LP Token
        pool_units_resource_address: ResourceAddress,
        /// A vault containing a badge which has the authority to mint `pool_units`
        /// tokens.
        pool_units_minter_badge: Vault,
        /// Liquidity
        liquidity: u128,
        /// The amount of fees imposed by the pool on swaps where 0 <= fee <= 1.
        fee: Decimal,
        // Ticks info
        ticks: KeyValueStore<i32, TicksInfo>,
        // positions info
        positions: HashMap<(ResourceAddress, i32, i32), PositionsInfo>,
        // slot0
        slot0: Slot0,
    }

    impl RadiswapV3 {
        // Implement the functions and methods which will manage those resources and data

        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate_radiswapv3(
            bucket_a: Bucket,
            bucket_b: Bucket,
            fee: Decimal,
        ) -> (ComponentAddress, Bucket) {
            // Ensure that none of the buckets are empty and that an appropriate
            // fee is set.
            assert!(
                !bucket_a.is_empty() && !bucket_b.is_empty(),
                "You must pass in an initial supply of each token"
            );
            assert!(
                fee >= dec!("0") && fee <= dec!("1"),
                "Invalid fee in thousandths"
            );

            // Create a badge which will be given the authority to mint the pool
            // unit tokens.
            let pool_units_minter_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "LP Token Mint Auth")
                .mint_initial_supply(1);

            // Create the pool units token along with the initial supply specified
            // by the user.
            let pool_units: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "Pool Unit")
                .metadata("symbol", "UNIT")
                .mintable(
                    rule!(require(pool_units_minter_badge.resource_address())),
                    LOCKED,
                )
                .burnable(
                    rule!(require(pool_units_minter_badge.resource_address())),
                    LOCKED,
                )
                .mint_initial_supply(100);

            // Create the Radiswap component and globalize it
            let radiswap: ComponentAddress = Self {
                token0: Vault::with_bucket(bucket_a),
                token1: Vault::with_bucket(bucket_b),
                pool_units_resource_address: pool_units.resource_address(),
                pool_units_minter_badge: Vault::with_bucket(pool_units_minter_badge),
                liquidity: 0,
                fee: fee,
                ticks: KeyValueStore::new(),
                positions: HashMap::new(),
                slot0: Slot0 {
                    sqrt_price_x96: 0,
                    tick: 0,
                },
            }
            .instantiate()
            .globalize();

            // Return the component address as well as the pool units tokens
            (radiswap, pool_units)
        }

        pub fn mint(
            &mut self,
            owner: ResourceAddress,
            tick_lower: i32,
            tick_upper: i32,
            amount: u128,
            bucket_a: Bucket,
            bucket_b: Bucket,
        ) -> (Bucket, Bucket) {
            let (mut bucket_a, mut bucket_b): (Bucket, Bucket) = if bucket_a.resource_address()
                == self.token0.resource_address()
                && bucket_b.resource_address() == self.token1.resource_address()
            {
                (bucket_a, bucket_b)
            } else if bucket_a.resource_address() == self.token1.resource_address()
                && bucket_b.resource_address() == self.token0.resource_address()
            {
                (bucket_b, bucket_a)
            } else {
                panic!("One of the tokens does not belong to the pool!")
            };

            if tick_lower >= tick_upper || tick_lower < MIN_TICK || tick_upper > MAX_TICK {
                panic!("Invalid Ticks")
            }

            if amount == 0 {
                panic!("Zero Liquidity")
            }

            self.update_ticks(tick_lower, amount);
            self.update_ticks(tick_upper, amount);

            self.update_position(owner, tick_lower, tick_upper, amount);

            let amount0: Decimal = dec!("0.998976618347425280"); // TODO: replace with calculation
            let amount1: Decimal = dec!(5000); // TODO: replace with calculation

            self.liquidity += amount;

            let balance0_before: Decimal;
            let balance1_before: Decimal;

            if amount0 > dec!(0) {
                balance0_before = self.token0.amount();
            } else {
                balance0_before = dec!(0);
            }

            if amount1 > dec!(0) {
                balance1_before = self.token1.amount();
            } else {
                balance1_before = dec!(0);
            }

            self.token0.put(bucket_a.take(amount0));
            self.token1.put(bucket_b.take(amount1));

            if amount0 > dec!(0) && balance0_before + amount0 > self.token0.amount() {
                panic!("Insufficent Input Amount");
            }

            if amount1 > dec!(0) && balance1_before + amount1 > self.token1.amount() {
                panic!("Insufficent Input Amount");
            }

            (bucket_a, bucket_b)
        }

        // Internal functions
        fn update_ticks(&mut self, tick: i32, liquity_delta: u128) {
            let liquity_before: u128;

            match self.ticks.get(&tick) {
                Some(tick_info) => {
                    liquity_before = tick_info.liquidity;
                }
                None => {
                    // Returning liquidity as 0
                    liquity_before = 0;

                    // Initializing HashMap for the tick.
                    // As in Rust variables do not have default values unless explicitly assigned.
                    self.ticks.insert(
                        tick,
                        TicksInfo {
                            initialized: false,
                            liquidity: 0,
                        },
                    )
                }
            };

            let liquity_after: u128 = liquity_before + liquity_delta;

            if liquity_before == 0 {
                // let info_struct = Info { initialized: true };
                // self.ticks.insert(tick, info_struct);
                // self.ticks.entry(1).and_modify(|entry| {
                //     entry.flag = false;
                // });
                match self.ticks.get_mut(&tick) {
                    Some(mut tick_info) => {
                        tick_info.initialized = true;
                    }
                    None => {
                        panic!("Not able to fetch Ticks");
                    }
                }
            }

            // let info_struct = Info {
            //     initialized: true,
            //     liquidity: liquity_before,
            // };
            // self.ticks.insert(tick, info_struct);
            match self.ticks.get_mut(&tick) {
                Some(mut tick_info) => {
                    tick_info.liquidity = liquity_after;
                }
                None => {
                    panic!("Unable to fetch Ticks");
                }
            }
            // if let Some(tick_info) = self.ticks.get_mut(&tick) {
            //     tick_info.liquidity = liquity_after;
            // }
        }

        fn update_position(
            &mut self,
            owner: ResourceAddress,
            tick_lower: i32,
            tick_upper: i32,
            liquity_delta: u128,
        ) {
            // let packed_bytes = ethabi.encode(owner, tickLower, tickUpper);
            let position = (owner, tick_lower, tick_upper);

            let position_info;

            match self.positions.get(&position) {
                Some(position_info_1) => {
                    position_info = position_info_1;
                }
                None => {
                    position_info = &PositionsInfo { liquidity: 0 };
                    self.positions
                        .insert(position, PositionsInfo { liquidity: 0 });
                }
            };

            let liquity_before: u128 = position_info.liquidity;

            let liquity_after: u128 = liquity_before + liquity_delta;

            match self.positions.get_mut(&position) {
                Some(mut position_info_2) => {
                    position_info_2.liquidity = liquity_after;
                }
                None => {
                    panic!("Unable to fetch Positions");
                }
            };
        }

        // This is a method, because it needs a reference to self.  Methods can only be called on components
        // pub fn free_token(&mut self) -> Bucket {
        //     info!(
        //         "My balance is: {} HelloToken. Now giving away a token!",
        //         self.sample_vault.amount()
        //     );
        //     // If the semi-colon is omitted on the last line, the last value seen is automatically returned
        //     // In this case, a bucket containing 1 HelloToken is returned
        //     self.sample_vault.take(1)
        // }

        pub fn swap(&mut self, mut input_tokens: Bucket) -> (Bucket, Bucket) {
            let (input_tokens_vault, output_tokens_vault): (&mut Vault, &mut Vault) =
                if input_tokens.resource_address() == self.token0.resource_address() {
                    (&mut self.token0, &mut self.token1)
                } else if input_tokens.resource_address() == self.token1.resource_address() {
                    (&mut self.token1, &mut self.token0)
                } else {
                    panic!("The given input tokens do not belong to this liquidity pool")
                };

            let next_tick: i32 = 85184; // TODO: replace with calculation
            let next_price: u128 = 5604469350942327889444743441197; // TODO: replace with calculation

            let amount0: Decimal = -dec!("0.008396714242162444"); // TODO: replace with calculation
            let amount1: Decimal = dec!(42); // TODO: replace with calculation

            self.slot0.sqrt_price_x96 = next_price;
            self.slot0.tick = next_tick;

            input_tokens_vault.put(input_tokens.take(amount1));
            (output_tokens_vault.take(-amount0), input_tokens)
        }
    }
}
