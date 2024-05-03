# token-sale

An example of a component that implements a simple token sale mechanism. A component like this could be usable on launch
platforms that sell new project tokens to a select group of whitelisted users.

# Resources and data

```rust
struct TokenSale {
    admin_badge: ResourceAddress,
    tokens_for_sale: Vault,
    payment_vault: Vault,
    sale_ticket_minter: Vault,
    sale_tickets: ResourceAddress,
    price_per_token: Decimal,
    max_personal_allocation: Decimal,
    sale_started: bool,
}
```

Each token sale component maintains a vault of the `tokens_for_sale` and a `payment_vault` into which the customer's
payments will be deposited. It uses the `price_per_token` to calculate how many tokens will be returned to the customer
when a payment is received. The payment amount may not exceed the `max_personal_allocation` (same for every user).
Finally, customers can only start buying tokens once `sale_started` has been set to `true`. The component
uses `sale_tickets` to implement a simple form of whitelisting, where a user is only allowed to buy tokens if she is in
possession of a ticket. An `admin_badge` is required to administer the component, i.e. to mint sale tickets, to start
the token sale and to withdraw the payment tokens.

# Facilitating a token sale

Note: This section explains the token sale from the perspective of the programmer.  
See the next section below for a typical test scenario.

## Step 1 - Instantiating a component for our SHINY token sale:

```rust
pub fn new(
    tokens_for_sale: Bucket,
    payment_token: ResourceAddress,
    price_per_token: Decimal,
    max_personal_allocation: Decimal,
) -> (ComponentAddress, Bucket) {
    let admin_badge = ResourceBuilder::new_fungible()
        .divisibility(DIVISIBILITY_NONE)
        .metadata("name", "admin_badge")
        .initial_supply(1);

    let sale_ticket_minter = ResourceBuilder::new_fungible()
        .divisibility(DIVISIBILITY_NONE)
        .metadata("name", "sale_ticket_minter")
        .initial_supply(1);

    let sale_tickets = ResourceBuilder::new_fungible()
        .divisibility(DIVISIBILITY_NONE)
        .metadata("name", "Sale Ticket Token")
        .metadata("symbol", "STT")
        .mintable(rule!(require(sale_ticket_minter.resource_address())), LOCKED)
        .burnable(rule!(require(sale_ticket_minter.resource_address())), LOCKED)
        .no_initial_supply();

    let component = Self {
        admin_badge: admin_badge.resource_address(),
        tokens_for_sale: Vault::with_bucket(tokens_for_sale),
        payment_vault: Vault::new(payment_token),
        sale_ticket_minter: Vault::with_bucket(sale_ticket_minter),
        sale_tickets,
        price_per_token,
        max_personal_allocation,
        sale_started: false,
    }
    .instantiate();

    let access_rules = AccessRules::new()
        .method("create_tickets", rule!(require(admin_badge.resource_address())))
        .method("start_sale", rule!(require(admin_badge.resource_address())))
        .method("withdraw_payments", rule!(require(admin_badge.resource_address())))
        .default(rule!(allow_all));

    (component.add_access_check(access_rules).globalize(), admin_badge)
}
```

Our component's `new` function excepts a few arguments:

1. `tokens_for_sale`: A bucket with tokens that customers should be able to buy. These tokens must be minted before
   instantiating the TokenSale component. For this example let's say that we pass in 10,000 newly minted Shiny Tokens
   (SHINY).
2. `payment_token`: The address of the token we will accept as payment. XRD might be a good choice.
3. `price_per_token`: The price in XRD per SHINY token. This means for 1 XRD a customer gets back 10 SHINY.
4. `max_personal_allocation`: The maximum allocation for each customer. Setting it to a value of `500` would mean that a
   customer may at max give us 500 XRD and will receive 5,000 SHINY in return.

## Step 2 - Whitelisting some customers:

Our component has been instantiated and owns the tokens to be sold. Now it is time to whitelist some customers. This
process is external to our component and may contain steps such as KYC etc. Given that we have whitelisted 10 customers,
we now need to mint 10 corresponding token sale tickets that we can issue to them:

```rust
pub fn create_tickets(&mut self, amount: i32) -> Bucket {
    self.sale_ticket_minter
        .authorize(|| borrow_resource_manager!(self.sale_tickets).mint(amount))
}
```

The `create_tickets` method must be called with the amount of tickets we want to mint. It then returns a bucket with
exactly that number of tickets. The method may only be called by someone in possession of the admin_badge.  
After having received the 10 tickets, we need to send them to our 10 whitelisted customers. These transactions are
external to our component.

## Step 3 - Starting the sale:

There are many examples of launch platforms that employ a first come, first served approach and so do we. We simply
communicate a sale date and time to our customers at which we start the sale.

```rust
pub fn start_sale(&mut self) {
    self.sale_started = true
}
```

The `start_sale` method may again only be called by an admin of the component.

## Step 4 - Customers buying tokens:

```rust
pub fn buy_tokens(&mut self, mut payment: Bucket, ticket: Bucket) -> (Bucket, Bucket) {
    // Check the sale has already started and is not over yet
    assert!(self.sale_started, "The sale has not started yet");
    assert!(self.has_tokens_left(), "The sale has ended already");

    // Check the user's ticket and burn it
    assert!(
        ticket.amount() == Decimal::one(),
        "You need to send exactly one ticket in order to participate in the sale"
    );
    self.sale_ticket_minter
        .authorize(|| ticket.burn());

    // Calculate the actual amount of tokens that the user can buy
    let payment_amount = min(payment.amount(), self.max_personal_allocation);
    let buy_ammount = payment_amount / self.price_per_token;
    let actual_buy_amount = min(self.tokens_for_sale.amount(), buy_ammount);
    let actual_payment_amount = actual_buy_amount * self.price_per_token;

    // Perform the token buy operation
    self.payment_vault.put(payment.take(actual_payment_amount));
    let bought_tokens = self.tokens_for_sale.take(actual_buy_amount);

    // Return the bought tokens and the amount the user might have overpaid
    (bought_tokens, payment)
}
```

When a customer calls the `buy_tokens` method, she must supply two arguments:

1. `payment`: A bucket that contains the payment tokens (XRD in our example).
2. `ticket`: A sale ticket that grants her access to the sale.

The method will first check that the sale has started and that it has not ended yet (i.e. there are still SHINY tokens
available). Next, the method will check that the customer has brought a ticket to the sale. If she did not bring a
ticket, the method will exit with an error. If she brought a ticket it will be burned, so she cannot buy tokens multiple
times. (Notice that we are using tokens instead of badges to implement our tickets as we can easily burn them.)  
After having handled all initial checks and access control, the component will calculate the exact amount of SHINY and
XRD tokens to be exchanged. These amounts depend on the payment amount, the `max_personal_allocation` and the amount
of `tokens_for_sale` that are left. Finally, the component puts the payment into the `payment_vault` and returns the
bought SHINY tokens to the customer. It also returns any amount the customer might have overpaid.

## Step 5 - Profit:

After the sale is over, all that is left to do, is to withdraw the payment tokens.

```rust
pub fn withdraw_payments(&mut self) -> Bucket {
    self.payment_vault.take_all()
}
```

To invoke this method, of course, one must be an administrator.

# Testing the component

The following instructions can be used to test a typical sequence of events that might happen in the lifetime of our
TokenSale component. For the sake of brevity only valid method calls are included. If you wish, you can of course also
try buying tokens before the sale has started or try buying tokens without a ticket. In those cases you should get an
error message.

```shell
# Set up
resim reset
# Save the XRD token address into env variable $xrd
export xrd=030000000000000000000000000000000000000000000000000004

resim new-account # Save the private key and account address into $admin_private_key and $admin_account resp.
resim new-account # Save the private key and account address into $customer_private_key and $customer_account resp.


# Create some tokens that we can put up for sale
resim new-token-fixed --symbol SHINY 10000 # Save the resource address into $shiny

# Publish our package
resim publish . # Save the package address into $package

# Instantiate the TokenSale component
# We pass as arguments:
# - a bucket with our 10,000 SHINY tokens
# - the address of the XRD token that we will accept as the payment token
# - the price per token: 0.1 XRD per SHINY
# - the maximum personal allocation: 500 XRD
# 
# The new call results in the creating of 4 new components.
# The first new ResourceAddress is the admin_badge address. Save that into $admin_badge
# Ignore the second new ResourceAddress.
# The third new ResourceAddress is the address of the sale ticket. Save that into $ticket
# Also, save the component address into $component
resim call-function $package TokenSale new 10000,$shiny $xrd 0.1 500

# Let's check our component. It should contain 10,000 SHINY tokens and no XRD.
resim show $component

# Mint some sale tickets. We need to specify the number of tickets we wish to create. 
resim run create_tickets.rtm

# Let's check our account. We should have 10 "Sale Ticket Token"s in there.
resim show $admin_account

# Next, let's transfer a ticket to a whitelisted user:
resim transfer 1 $ticket $customer_account

# Finally, let's start the sale
resim run start_sale.rtm

# Now we step into the shoes of our customer:
resim set-default-account $customer_account $customer_private_key

# Because we are a sneaky customer, we will try to get a few more tokens than we have been allocated. 
# We specify a bucket with 600 XRD as payment and we also pass our sale ticket.
resim call-method $component buy_tokens 600,$xrd 1,$ticket

# Let's check how many tokens we received.
# We should see 5000 SHINY tokens in our wallet. Even though we tried sending 600 XRD, the component only took 500 XRD 
# and returned the other 100 XRD to us.
resim show $customer_account

# Let's switch back to our admin user.
resim set-default-account $admin_account $admin_private_key

# Checking on our component, we see that some SHINY tokens have been sold and some XRD tokens have been deposited.
resim show $component

# Even if the token sale has not yet ended, we may withdraw payments received up to this point.
resim run withdraw_payments.rtm
```



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