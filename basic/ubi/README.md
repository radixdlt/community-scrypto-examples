
# UBI Token Example

UBI is minted at a rate of 1000 UBI every 30 days. If there are multiple identity providers, each provider mints its own fraction of the total UBI for a person.

A UBI token implementation. The token is designed to dynamically allow changing between multiple identity providers. Initially a low sybil resistance provider (e.g. Google) could be used and as the value of the token increases, a higher sybil resistance provider (e.g. BrightID or Instapass) could be used. Perhaps a Radix native identity provider could be used in the future if a native self sovereign proof of person protocol is ever built on the network.

## Design considerations

* Need to explore the api for `collect_ubi` which modifies the badge to record when UBI was last collected. (Should/could a proof be passed instead of the badge itself?)
* If someone's badge is compromised, the `register` function could recall the person's badge, but would it be worth it since the vault id is needed to find the badge? Alternately, after a badge is expired, a new badge could be issued so the badge wouldn't be recallable but then if a badge is lost it would remain compromised till it expires.
* Would like to implement a demmurage to encourage people to spend their UBI, but couldn't find a good way to do that with the current Radix security model.
