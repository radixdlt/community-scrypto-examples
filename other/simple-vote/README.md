* Simple Vote

A blueprint which can be instantiated to represent a voting process
where each member gets a single vote of equal weight. This is
simplified to try to keep it in a single blueprint, but some of the
limitations could be overcome by adding another "vote manager"
blueprint.

- Votes are simple "yes" or "no" tallies, with the assumption that the
  subject being voted on is communicated off-chain
- We only allow one vote at a time
- The component will have data containing an optional "CurrentVote",
  where CurrentVote will contain the vote identifier, end time, and
  yes/no tallies. The CurrentVote data is only present when a vote is
  in progress
- Upon instantiation of the component, an admin badge will be returned
- A method can be used to obtain a "member" badge, required to
  vote. This part would be more sophisticated in a real dApp, possibly
  requiring the member to have some proof of stake, or only allowing a
  predetermined set of accounts to vote. For now, a proof of the admin
  badge will be required (admin must manually authorize new members)
  - The member badge will be locked with the user's account (unless
    admin permits otherwise) using ~restrict_deposit~
  - The member badge will be recallable by the admin, to remove a
    member from the organization
- Another method is used to start a vote, specifying an end time and
  vote identifier. This method requires that a vote is not already in
  progress, and requires the admin badge.
- Another method is used to vote, which:
  - Requires that a vote has been started and the end time has not
    passed
  - Requires the voter has a member badge and the badge does not
    contain this vote's identifier in its data
  - Updates the vote state, and updates the member's badge to mark
    that they have voted
- Another method is used to end the vote, which checks that a vote is
  in progress, and that the end time has passed. It mints a vote
  badge with the CurrentVote as its data, and removes the data from
  the component state. The vote badges are stored with the component
  - The vote badge is only mintable by the component to ensure the
    necessary checks have been passed and the data is valid

** References
- [[https://github.com/rynoV/community-scrypto-examples/blob/main/basic/auction/src/lib.rs][This example]] for how to deal with time

** Test cases

- Check default states of resource behaviours, e.g. can the vote badge
  be burned or its data updated? Can the component badge be minted?
- Check configured resource behaviours
- Requirements described above


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