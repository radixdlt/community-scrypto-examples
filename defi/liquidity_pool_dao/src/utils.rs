use scrypto::prelude::*;

/// Sorts the two addresses passed to it and returns them.
/// 
/// # Arguments:
/// 
/// * `address1` (ResourceAddress) - The first resource address
/// * `address2` (ResourceAddress) - The second resource address
/// 
/// # Returns:
/// 
/// * (ResourceAddress, ResourceAddress) - A tuple containing the two addresses passed after they had been sorted.
/// 
/// # Notes:
///    
/// This function performs what is called "Lexicographical comparison" on the vector representation of the two addresses
/// passed to this function. The order in which these two addresses are sorted is not important. The only thing that is
/// important is that this function must be deterministic such that on all runs on the same two addresses it sorts them
/// in the same way. The reason as to why the sorting is not as important as determinism is because this function will 
/// mainly be used for the keys and values of the hashmap storing the liquidity pools. As long as this function output
/// is deterministic, the way of sorting the addresses is irrelevant. 
pub fn sort_addresses(address1: ResourceAddress, address2: ResourceAddress) -> (ResourceAddress, ResourceAddress) {
    return if address1.to_vec() > address2.to_vec() {
        (address1, address2)
    } else {
        (address2, address1)
    };
}

/// Sorts the two buckets passed depending on their resource addresses and returns them.
/// 
/// # Arguments:
/// 
/// * `bucket1` (Bucket) - The first bucket
/// * `bucket2` (Bucket) - The second bucket
/// 
/// # Returns:
/// 
/// * (Bucket, Bucket) - A tuple of the two buckets sorted according to their resource addresses.
pub fn sort_buckets(bucket1: Bucket, bucket2: Bucket) -> (Bucket, Bucket) {
    // Getting the sorted addresses of the two buckets given
    let sorted_addresses: (ResourceAddress, ResourceAddress) = sort_addresses(
        bucket1.resource_address(), 
        bucket2.resource_address()
    );

    // Sorting the buckets and returning them back
    return if bucket1.resource_address() == sorted_addresses.0 {
        (bucket1, bucket2)
    } else {
        (bucket2, bucket1)
    }
}

/// Creates a symbol for the given address pair.
/// 
/// This function is used to create a symbol for a given address pair. Tokens in Radix typically have metadata associated
/// with them. One of the things that tokens typically have is a symbol. Therefore, this function loads the resource
/// manager of the two addresses passed and tried to find if there is a `symbol` key in the token metadata. If there
/// is then that's good and that will be used. However, if no symbol is found then the resource address is used as the
/// symbol of the token.
/// 
/// # Arguments:
/// 
/// * `address1` (ResourceAddress) - The resource address of the first token.
/// * `address2` (ResourceAddress) - The resource address of the second token.
/// 
/// # Returns:
/// 
/// `String` - A string of the pair symbol
pub fn address_pair_symbol(address1: ResourceAddress, address2: ResourceAddress) -> String {
    // Sorting the two addresses passed to the function
    let addresses: (ResourceAddress, ResourceAddress) = sort_addresses(address1, address2);
    
    // If the resource manager of the given address has a symbol, then we attempt to get this symbol. If not, then we 
    // simply put in the address of the resource. 
    //
    // This following block of code is implemented in a less than ideal way as tuples do not support mapping a function
    // to them, so we need to work on each of the elements separately. 
    let names: (String, String) = (
        match borrow_resource_manager!(addresses.0).metadata().get("symbol") {
            Some(s) => format!("{}", s),
            None => format!("{}", addresses.0)
        },
        match borrow_resource_manager!(addresses.1).metadata().get("symbol") {
            Some(s) => format!("{}", s),
            None => format!("{}", addresses.1)
        }
    );

    // Format the names and return them.
    return format!("{}-{}", names.0, names.1);
}