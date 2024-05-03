# Guestbook - a simple guest book

The guestbook blueprint will allow you to create a new guestbook
instance that you can then open and close for people to enter
themselves into.

Visitors can, while the book is open, register themselves with a
message in the guestbook and this event will then be stored on the
ledger until the heat death of the universe (terms and conditions may
apply).

There are also methods for reading the contents of the guestbook so
that the people of the future can suitable celebrate these brave
pioneers in guestbookery on the ledger.

## How to build the blueprint
Make sure you have the necessary toolchain installed, see
[here](https://docs-babylon.radixdlt.com/main/getting-started-developers/getting-started-developers.html)
for details. You will need Scrypto 0.8.0.
- From the command line, in the `guestbook` directory, run `scrypto build`

### How to generate the documentation
- From the command line, in the `guestbook` directory, run `cargo doc`

### How to give the guestbook a spin

These examples use a Unix-like command line. Things may look slightly
different on other platforms. In particular note that I set
environment variables by using `export variable=value`, you may need
to substitute your platform's particular convention for this.

Start by going to the `guestbook` directory. Then run these commands,
and expect output similar to what you see here.

        $ resim reset
        Data directory cleared.
        
        $ resim new-account
        A new account has been created!
        Account component address: account_sim1qwlyc3lymsrpzj8737w8tu5wvqtfa49pxlvzx0c8a69qtduca3
        Public key: 02979388c49526a0b4b0890c5dd0964c02d610ee9f09e83393b292cb1dc5f9e8fd
        Private key: 19776701b3814210559ad70653dbe9b8d9bba24390aea8da5ddcae282fc1f1ec
        NonFungibleGlobalId: resource_sim1qz2rl5shctgafnpm0s44fxfk29n39nl4fe42sh06vccqxg7jw6:#1#
        Account configuration in complete. Will use the above account as default.

Now store that account address, nonfungible resource and nonfungible
local id in environment variables:

        $ export account=account_sim1qwlyc3lymsrpzj8737w8tu5wvqtfa49pxlvzx0c8a69qtduca3
        $ export nfres=resource_sim1qz2rl5shctgafnpm0s44fxfk29n39nl4fe42sh06vccqxg7jw6
        $ export nflid=#1#

Notice that we're splitting up the NonFungibleGlobalId there into its two components.

Then publish the component:

        $ resim publish .
        Success! New Package: package_sim1q92ysw2q52qt0h2p3cnvsm4pr3ruh3lyn49fm76v28fqrd9fk6

And store that package address:

        $ export package=package_sim1q92ysw2q52qt0h2p3cnvsm4pr3ruh3lyn49fm76v28fqrd9fk6

Now create a new Guestbook component. Notice we're using the
`$package` environment variable:

        $ resim call-function $package Guestbook instantiate_book

Or you can use a transaction manifest which does the same ting (and
which uses environmental variables internally):

        $ resim run rtm/instantiate_book.rtm

In either case the output is something like this:

        Transaction Status: COMMITTED SUCCESS
        Transaction Fee: 0.2810085 XRD used for execution, 0 XRD used for royalty, 0 XRD in bad debt
        Cost Units: 100000000 limit, 2810085 consumed, 0.0000001 XRD per cost unit, 0% tip
        Logs: 0
        Outputs:
        ├─ Tuple()
        ├─ Tuple(ComponentAddress("component_sim1qfsu5vcd5tgp6dgznsz7zddeu9jew8t2cn7mzsvcdgcqnzu0hl"), Own("Bucket(2)"))
        └─ Tuple()
        New Entities: 2
        └─ Component: component_sim1qfsu5vcd5tgp6dgznsz7zddeu9jew8t2cn7mzsvcdgcqnzu0hl
        └─ Resource: resource_sim1qpsu5vcd5tgp6dgznsz7zddeu9jew8t2cn7mzsvcdgcqkuy5gh

Store away the component address and admin badge address:

        $ export book=component_sim1qfsu5vcd5tgp6dgznsz7zddeu9jew8t2cn7mzsvcdgcqnzu0hl
        $ export badge=resource_sim1qpsu5vcd5tgp6dgznsz7zddeu9jew8t2cn7mzsvcdgcqkuy5gh

The book starts closed so you can't use it yet. The admin user has to
first open it. Luckily that admin user is you and you have the badge
for it:

        $ resim run rtm/open_book.rtm 

Most of the output now doesn't matter except that it should say this:

        Transaction Status: COMMITTED SUCCESS

The guest book is open. If you want some variety you can make
additional users with `resim new-account` and put multiple different
NFTs in there, but we will keep it simple here and use the same
account as before:

        $ resim call-method $book "i_was_here" -- $nfres:$nflid "First post!"

And we can also use the transaction manifest here so we get two
entries:

        $ resim run rtm/i_was_here.rtm 

Again this response means it worked:

        Transaction Status: COMMITTED SUCCESS

We can now list the book to see if we made it there:

        $ resim call-method $book "who_was_there" -- 0

or with the transaction manifest:

        $ resim run rtm/who_was_there.rtm 

If we did both of the `i_was_there` calls above then we get something
like this:

        Transaction Status: COMMITTED SUCCESS
	    (...)
        Outputs:
        ├─ Tuple()
        ├─ Array<Tuple>(Tuple(0i64, NonFungibleGlobalId("resource_sim1qz2rl5shctgafnpm0s44fxfk29n39nl4fe42sh06vccqxg7jw6:#1#"), "First post!"), Tuple(0i64, NonFungibleGlobalId("resource_sim1qz2rl5shctgafnpm0s44fxfk29n39nl4fe42sh06vccqxg7jw6:#1#"), "Witness me!"))
        └─ Tuple()
        New Entities: 0

We notice two entries in the returned array: the "First post!" from
the manual invocation we did, and the "Witness me!" that comes from
the transaction manifest.

After all this, running the `close_book` and `visitor_count` methods
is left as an exercise for the reader. Have fun!


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