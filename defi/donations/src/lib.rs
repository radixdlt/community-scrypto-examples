use scrypto::prelude::*;

blueprint! {
    struct Donations {
        admin_vault: Vault,
        admin_badge: ResourceDef,
        fee: Decimal,
        collected_fees: Vault,
        badges: HashMap<Address, Vec<Vault>>
    }

    impl Donations {
        
        pub fn new(fee_percent: Decimal) -> (Component, Bucket) {
            let admin_bucket = ResourceBuilder::new()
                .metadata("name", "Donations Badge Mint Auth")
                .new_badge_fixed(2);

            let admin_resource_def = admin_bucket.resource_def();
            let admin_return_bucket: Bucket = admin_bucket.take(1); // Return this badge to the caller

            let component = Self {
                admin_vault: Vault::with_bucket(admin_bucket),
                admin_badge: admin_resource_def,
                collected_fees: Vault::new(RADIX_TOKEN),
                fee: fee_percent,
                badges: HashMap:: new()
            }
            .instantiate();

            (component, admin_return_bucket)
        }

        // make a new badge with specific meta 
        pub fn make_badge(&mut self, owner: Address, identifier: String, title: String, description: String, url: String, price: Decimal, supply: Decimal) {
            scrypto_assert!(supply > Decimal::zero(), "Supply cannot be zero");
            scrypto_assert!(price > Decimal::zero(), "Price cannot be zero");

            // get existing badges for owner
            let badges = self.badges.entry(owner).or_insert(Vec::new());

            let badge_resource_def = ResourceBuilder::new()
                .metadata("name", "Donations Badge")
                .metadata("identifier", identifier)
                .metadata("title", title)
                .metadata("description", description)
                .metadata("url", url)
                .metadata("price", price.to_string())
                .new_badge_mutable(self.admin_vault.resource_def());

            let badge = self.admin_vault.authorize(|badge| {
                badge_resource_def
                    .mint(supply, badge)
            });

            badges.push(Vault::with_bucket(badge));
        }

        // get available badges of an owner
        pub fn get_badges(&mut self, owner: Address) -> Vec<Address> {
            scrypto_assert!(self.badges.contains_key(&owner), "No badges found for this owner");

            let badges = self.badges.get(&owner).unwrap();

            let mut resource_defs = Vec::new();
            for b in &badges[..] {
                if b.amount() > 0.into() {
                    resource_defs.push(b.resource_address())
                }
            }
            return resource_defs
        }

        pub fn donate(&mut self, owner: Address, badge_address: Address, payment: Bucket) -> (Bucket, Bucket){
            scrypto_assert!(self.badges.contains_key(&owner), "No badges found for this owner");
            scrypto_assert!(payment.resource_def() == RADIX_TOKEN.into(), "You must use Radix (XRD).");

            let badges = self.badges.get(&owner).unwrap();

            let mut iter = badges.iter();

            let badge = match iter.find(|&b| b.resource_address() == badge_address) {
                Some(value) => value,
                None => scrypto_abort("No such badge found")
            };

            scrypto_assert!(!badge.is_empty(), "No badge available");

            let metadata = badge.resource_def().metadata();
            let price:Decimal = metadata["price"].parse().unwrap();

            scrypto_assert!(payment.amount() >= price, "Not enough amount");

            // Take fee
            let price_bucket = payment.take(price);
            let fee = price * self.fee / 100;
            self.collected_fees.put(price_bucket.take(fee));
            Account::from(owner).deposit(price_bucket);

            (badge.take(1), payment)
        }
        
        // #[auth(admin_badge)]
        pub fn withdraw(&mut self, amount: Decimal) -> Bucket {
            scrypto_assert!(self.collected_fees.amount() >= amount, "Withdraw amount is bigger than available assets");

            self.collected_fees.take(amount)
        }
    }
}
