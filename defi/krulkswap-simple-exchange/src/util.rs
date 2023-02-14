use scrypto::prelude::*;

// Sort two ResourceAddresses.
pub fn address_sort(a: ResourceAddress, b: ResourceAddress) -> (ResourceAddress, ResourceAddress) {
    if a.to_vec() > b.to_vec() {
        return (a, b)
    } else {
        return (b, a)
    }
}

// Sort two Buckets.
pub fn bucket_sort(a: Bucket, b: Bucket) -> (Bucket, Bucket) {
    if a.resource_address().to_vec() > b.resource_address().to_vec() {
        return (a, b)
    } else {
        return (b, a)
    }
}
