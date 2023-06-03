use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData, Debug)]
struct FriendData {
    #[mutable]
    savings: Decimal,
}

#[blueprint]
mod save_with_friends {

    struct SaveWithFriends {
        savings: Vault,
        friends_nft: ResourceAddress,
        desired_amount: Option<Decimal>,
    }

    impl SaveWithFriends {
        pub fn instantiate_amount_bound(desired_amount: Decimal) -> ComponentAddress {
            let friends_nft = ResourceBuilder::new_uuid_non_fungible::<FriendData>()
                .metadata("name", "Save With Friends NFT")
                .metadata("symbol", "SWF")
                .create_with_no_initial_supply();

            let component_address = Self {
                savings: Vault::new(RADIX_TOKEN),
                friends_nft,
                desired_amount: Some(desired_amount),
            }
            .instantiate()
            .globalize();

            component_address
        }

        pub fn register_friend(&mut self) -> Bucket {
            // TODO: auth
            let nft_resource_manager = borrow_resource_manager!(self.friends_nft);
            nft_resource_manager.mint_uuid_non_fungible(FriendData { savings: 0.into() })
        }

        pub fn deposit(&mut self, xrd: Bucket, nft: Bucket) {
            // TODO: auth

            // deposit xrd into savings
            let amount = xrd.amount();
            self.savings.put(xrd);

            // update friend's savings record
            let friend_nft: NonFungible<FriendData> = nft.non_fungible();
            let friend_savings = friend_nft.data().savings + amount;
            borrow_resource_manager!(self.friends_nft).update_non_fungible_data(
                friend_nft.local_id(),
                "savings",
                friend_savings,
            );

            // return all XRD if we've reached the desired amount
            if let Some(desired_amount) = self.desired_amount {
                if self.savings.amount() >= desired_amount {
                    let resource_manager = borrow_resource_manager!(self.friends_nft);
                    let friends = resource_manager.
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scrypto_unit::*;

    #[test]
    fn amount_bound_works() {}
}
