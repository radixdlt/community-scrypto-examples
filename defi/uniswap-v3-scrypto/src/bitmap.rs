use ethers::types::U256;
use scrypto::prelude::*;

#[blueprint]
mod bitmap {
    struct TickBitmap {
        tick_bitmap: HashMap<i16, u128>,
    }

    impl TickBitmap {
        pub fn instantiate_bitmap() -> ComponentAddress {
            Self {
                tick_bitmap: HashMap::new(),
            }
            .instantiate()
            .globalize()
        }

        fn position(tick: i32) -> (i16, u8) {
            let word_pos = (tick >> 8) as i16;
            let bit_pos = (tick % 256) as u8;
            (word_pos, bit_pos)
        }

        fn flipTick(&mut self, tick: i32, tick_spacing: i32) {
            assert!((tick % tick_spacing) == 0);
            let (word_pos, bit_pos) = Self::position(tick / tick_spacing);
            let mask: u128 = 1 << bit_pos;

            match self.tick_bitmap.get_mut(&word_pos) {
                Some(mut tick_bitmap) => {
                    let mut bitmap_value = *tick_bitmap ^ mask;
                    tick_bitmap = &mut bitmap_value;
                }

                None => {
                    self.tick_bitmap.insert(word_pos, mask);
                }
            }
        }

        fn nextInitializedTickWithinOneWord(
            &mut self,
            tick: i32,
            tick_spacing: i32,
            lte: bool,
        ) -> (i32, bool) {
            let mut compressed = tick / tick_spacing;
            if tick < 0 && tick % tick_spacing != 0 {
                compressed -= 1;
            }

            let initialized;
            let next;

            if lte {
                let (word_pos, bit_pos) = Self::position(compressed);
                let mask: U256 = U256::from((1 << bit_pos) - 1 + (1 << bit_pos));

                let masked: U256;

                match self.tick_bitmap.get(&word_pos) {
                    Some(tick_bitmap) => {
                        masked = U256::from(*tick_bitmap) & mask;
                    }

                    None => {
                        panic!("Not able to retrieve tick Bitmap");
                    }
                }

                initialized = masked != U256::from(0);

                next = if initialized {
                    (compressed - (bit_pos - Self::most_significant_bit(masked)) as i32)
                        * tick_spacing
                } else {
                    (compressed - (bit_pos as i32)) * tick_spacing
                };
            } else {
                let (word_pos, bit_pos) = Self::position(compressed + 1);

                let mask: U256 = U256::from(!((1 << bit_pos) - 1));
                // uint256 masked = self[wordPos] & mask;
                let masked: U256;

                match self.tick_bitmap.get(&word_pos) {
                    Some(tick_bitmap) => {
                        masked = U256::from(*tick_bitmap) & mask;
                    }

                    None => {
                        panic!("Not able to retrieve tick Bitmap");
                    }
                }

                // if there are no initialized ticks to the left of the current tick, return leftmost in the word
                initialized = masked != U256::from(0);

                next = if initialized {
                    (compressed + 1 + (Self::least_significant_bit(masked) - bit_pos) as i32)
                        * tick_spacing
                } else {
                    (compressed + 1 + (std::u8::MAX - bit_pos) as i32) * tick_spacing
                };

                // initialized
                //     ? (compressed + 1 + int24(uint24((BitMath.leastSignificantBit(masked) - bitPos)))) * tickSpacing
                //     : (compressed + 1 + int24(uint24((type(uint8).max - bitPos)))) * tickSpacing;
            }

            (next, initialized)
        }

        fn most_significant_bit(mut x: U256) -> u8 {
            assert!(x > U256::from(0));

            let mut r: u8 = 0;

            if x >= U256::from("0x100000000000000000000000000000000") {
                x >>= 128;
                r += 128;
            }
            if x >= U256::from("0x10000000000000000") {
                x >>= 64;
                r += 64;
            }
            if x >= U256::from("0x100000000") {
                x >>= 32;
                r += 32;
            }
            if x >= U256::from("0x10000") {
                x >>= 16;
                r += 16;
            }
            if x >= U256::from("0x100") {
                x >>= 8;
                r += 8;
            }
            if x >= U256::from("0x10") {
                x >>= 4;
                r += 4;
            }
            if x >= U256::from("0x4") {
                x >>= 2;
                r += 2;
            }
            if x >= U256::from("0x2") {
                r += 1;
            }

            return r;
        }

        fn least_significant_bit(mut x: U256) -> u8 {
            assert!(x > U256::from(0));

            let mut r: u8 = 255;

            if x & U256::from(std::u128::MAX) > U256::from(0) {
                r -= 128;
            } else {
                x >>= 128;
            }

            if x & U256::from(std::u64::MAX) > U256::from(0) {
                r -= 64;
            } else {
                x >>= 64;
            }

            if x & U256::from(std::u32::MAX) > U256::from(0) {
                r -= 32;
            } else {
                x >>= 32;
            }

            if x & U256::from(std::u16::MAX) > U256::from(0) {
                r -= 16;
            } else {
                x >>= 16;
            }

            if x & U256::from(std::u8::MAX) > U256::from(0) {
                r -= 8;
            } else {
                x >>= 8;
            }

            if x & U256::from(0xf) > U256::from(0) {
                r -= 4;
            } else {
                x >>= 4;
            }

            if x & U256::from(0x3) > U256::from(0) {
                r -= 2;
            } else {
                x >>= 2;
            }

            if x & U256::from(0x1) > U256::from(0) {
                r -= 1;
            }

            return r;
        }
    }
}
