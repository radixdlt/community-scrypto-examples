use scrypto::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Decode, Encode, Describe)]
struct Reference {
    id: u32,
    sequence: u32,
    name: String,

    // Price
    unit_price: Decimal,
}

impl Reference {
    pub fn next(&mut self) -> u32 {
        self.sequence += 1;
        self.sequence
    }
}

#[derive(NonFungibleData)]
struct Article {
    name: String,
    unit_price: Decimal,
}

blueprint! {
    struct Catalog {
        sequence: u32,
        references: HashMap<u32, Reference>,
        stock: HashMap<u32, Vault>,

        cashier: Vault,

        reference_minter: Vault,
        // Owner
        owner_badge_ref: ResourceAddress,
    }

    impl Catalog {
        pub fn new() -> (ComponentAddress, Bucket) {

            // Owner relative
            let owner_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", format!("Owner of catalog"))
                .initial_supply(1);

            // Articles NFT vault
            let reference_minter: Bucket = ResourceBuilder::new_fungible()
            .metadata("name", format!("Reference minter"))
            .mintable(rule!(require(owner_badge.resource_address())), LOCKED)
            .burnable(rule!(require(owner_badge.resource_address())), LOCKED)
            .divisibility(DIVISIBILITY_NONE)
            .initial_supply(1);

            let reference_minter_address = reference_minter.resource_address();

            let component = Self{
                sequence: 0u32,
                references: HashMap::new(),
                stock: HashMap::new(),

                cashier: Vault::new(RADIX_TOKEN),
                reference_minter: Vault::with_bucket(reference_minter),

                owner_badge_ref: owner_badge.resource_address(),
            }.instantiate();

            let access_rules = AccessRules::new()
            .method("withdraw", rule!(require(owner_badge.resource_address())))
            .method("become_minter", rule!(require(owner_badge.resource_address())))
            .method("register_reference", rule!(require(reference_minter_address)))
            .method("add_stock_to_reference", rule!(require(reference_minter_address)))
            .default(AccessRule::AllowAll);

            (component.add_access_check(access_rules).globalize(), owner_badge)
        }

        pub fn become_minter(&mut self) -> Bucket{

            self.reference_minter.authorize(|| {

                let reference_minter_manager: &ResourceManager = borrow_resource_manager!(self.reference_minter.resource_address());

                reference_minter_manager.mint(1)
            })
        }

        pub fn register_reference(&mut self, name: String, unit_price: Decimal) -> u32 {

            self.sequence += 1;

            let article_stock = ResourceBuilder::new_non_fungible()
            .metadata("name", format!("Reference {}", name))
            .mintable(rule!(require(self.reference_minter.resource_address())), LOCKED)
            // We could imagine perishable article which would be burned upon reaching their expiry date
            .burnable(rule!(require(self.reference_minter.resource_address())), LOCKED)
            .no_initial_supply();

            self.references.insert(self.sequence, Reference{id: self.sequence, sequence: 0, name, unit_price});
            self.stock.insert(self.sequence, Vault::new(article_stock));

            self.sequence
        }

        pub fn add_stock_to_reference(&mut self, id: u32, amount: u64) -> Decimal {

            self.reference_minter.authorize(|| {
                let manager: &ResourceManager = borrow_resource_manager!(self.stock.get_mut(&id).unwrap().resource_address());

                let reference = self.references.get_mut(&id).unwrap();

                for _counter in 0..amount {
                    let next = reference.next();
                    let new_article = manager.mint_non_fungible(&NonFungibleId::from_u32(next), Article{name: String::from(reference.name.to_string()), unit_price: reference.unit_price});
                    self.stock.get_mut(&id).unwrap().put(new_article);
                }

            });

            self.stock.get(&id).unwrap().amount()
        }

        pub fn purchase_article(&mut self, id: u32, quantity: u64, mut amount: Bucket) -> (Bucket, Bucket) {
            assert!(amount.resource_address() == RADIX_TOKEN, "You need to pay in XRD.");

            let unit_price = self.references.get(&id).unwrap().unit_price;
            let total_price = unit_price * quantity;

            assert!(amount.amount() >= total_price, "Total price for {} of article {} is {} XRD. You only provided {} XRD.", quantity, id, total_price, amount.amount());

            self.cashier.put(amount.take(total_price));

            (self.stock.get_mut(&id).unwrap().take(quantity), amount)
        }

        pub fn withdraw(&mut self) -> Bucket {
            self.cashier.take_all()
        }
    }
}
