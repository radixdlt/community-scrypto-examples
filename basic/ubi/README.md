
# UBI Token Example

UBI is minted at a rate of 1000 UBI every 30 days. If there are multiple identity providers, each provider mints its own fraction of the total UBI for a person.

A UBI token implementation. The token is designed to dynamically allow changing between multiple identity providers. Initially a low sybil resistance provider (e.g. Google) could be used and as the value of the token increases, a higher sybil resistance provider (e.g. BrightID or Instapass) could be used. Perhaps a Radix native identity provider could be used in the future if a native self sovereign proof of person protocol is ever built on the network.

## Design considerations

* Need to explore the api for `collect_ubi` which modifies the badge to record when UBI was last collected. (Should/could a proof be passed instead of the badge itself?)
* If someone's badge is compromised, the `register` function could recall the person's badge, but would it be worth it since the vault id is needed to find the badge? Alternately, after a badge is expired, a new badge could be issued so the badge wouldn't be recallable but then if a badge is lost it would remain compromised till it expires.
* Would like to implement a demmurage to encourage people to spend their UBI, but couldn't find a good way to do that with the current Radix security model.


# License

The Radix Community Scrypto Examples code is released under Radix Modified MIT License.

    Copyright 2024 Radix Publishing Ltd

    Permission is hereby granted, free of charge, to any person obtaining a copy of
    this software and associated documentation files (the "Software"), to deal in
    the Software for non-production informational and educational purposes without
    restriction, including without limitation the rights to use, copy, modify,
    merge, publish, distribute, sublicense, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    This notice shall be included in all copies or substantial portions of the
    Software.

    THE SOFTWARE HAS BEEN CREATED AND IS PROVIDED FOR NON-PRODUCTION, INFORMATIONAL
    AND EDUCATIONAL PURPOSES ONLY.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
    FOR A PARTICULAR PURPOSE, ERROR-FREE PERFORMANCE AND NONINFRINGEMENT. IN NO
    EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES,
    COSTS OR OTHER LIABILITY OF ANY NATURE WHATSOEVER, WHETHER IN AN ACTION OF
    CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
    SOFTWARE OR THE USE, MISUSE OR OTHER DEALINGS IN THE SOFTWARE. THE AUTHORS SHALL
    OWE NO DUTY OF CARE OR FIDUCIARY DUTIES TO USERS OF THE SOFTWARE.