pub fn swap_tokens_for_exact_tokens(
    &mut self,
    tokens: Bucket,
    output_resource_address: ResourceAddress,
    output_amount: Decimal
) -> (Bucket, Bucket) {
    // Checking if there does exist a liquidity pool for the given pair of tokens
    self.assert_pool_exists(tokens.resource_address(), output_resource_address, String::from("DEX Swap For Exact"));

    // Sorting the two addresses passed, getting the associated liquidity pool and then performing the swap.
    let sorted_addresses: (ResourceAddress, ResourceAddress) = sort_addresses(
        tokens.resource_address(), 
        output_resource_address
    );
    return self.liquidity_pools[&sorted_addresses].swap_tokens_for_exact_tokens(tokens, output_amount);
}

pub fn swap_exact_tokens_for_tokens(
    &mut self,
    tokens: Bucket,
    output_resource_address: ResourceAddress,
    min_amount_out: Decimal
) -> Bucket {
    // Checking if there does exist a liquidity pool for the given pair of tokens
    self.assert_pool_exists(tokens.resource_address(), output_resource_address, String::from("DEX Swap Exact"));

    // Sorting the two addresses passed, getting the associated liquidity pool and then performing the swap.
    let sorted_addresses: (ResourceAddress, ResourceAddress) = sort_addresses(
        tokens.resource_address(), 
        output_resource_address
    );
    return self.liquidity_pools[&sorted_addresses].swap_exact_tokens_for_tokens(tokens, min_amount_out);
}

pub fn swap(
    &mut self,
    tokens: Bucket,
    output_resource_address: ResourceAddress
) -> Bucket {
    // Checking if there does exist a liquidity pool for the given pair of tokens
    self.assert_pool_exists(tokens.resource_address(), output_resource_address, String::from("DEX Swap"));

    // Sorting the two addresses passed, getting the associated liquidity pool and then performing the swap.
    let sorted_addresses: (ResourceAddress, ResourceAddress) = sort_addresses(
        tokens.resource_address(), 
        output_resource_address
    );
    return self.liquidity_pools[&sorted_addresses].swap(tokens);
}

pub fn remove_liquidity(
    &mut self,
    tracking_tokens: Bucket
) -> (Bucket, Bucket) {
    // Check to make sure that the tracking tokens provided are indeed valid tracking tokens that belong to this
    // DEX.
    assert!(
        self.tracking_token_address_pair_mapping.contains_key(&tracking_tokens.resource_address()),
        "[DEX Remove Liquidity]: The tracking tokens given do not belong to this exchange."
    );

    // Getting the address pair associated with the resource address of the tracking tokens and then requesting
    // the removal of liquidity from the liquidity pool
    let addresses: (ResourceAddress, ResourceAddress) = self.tracking_token_address_pair_mapping[&tracking_tokens.resource_address()];
    return self.liquidity_pools[&addresses].remove_liquidity(tracking_tokens);
}






