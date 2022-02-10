use scrypto::prelude::*;

#[derive(Debug, sbor::Decode, sbor::Encode, sbor::Describe, sbor::TypeId)]
pub struct Product {
    id : u128,
    name : String,
    price : Decimal
}

impl ToString  for Product {
    fn to_string(&self)  -> String
    { 
        return format!("{}|{}|{}",self.id, self.name,self.price);
    }
}

#[derive(Default ,Debug, sbor::Decode, sbor::Encode, sbor::Describe, sbor::TypeId)]
pub struct PostalAddress {
    street : String, 
    zip_code : String, 
    city : String
}

#[derive(NftData)]
pub struct PermanentSellerNftData {
 
}

#[derive(NftData)]
pub struct SellerNftData {
    product_id : u128,
    #[scrypto(mutable)]
    has_been_sent : bool,
    #[scrypto(mutable)]
    postal_stamp_collected : bool,
    #[scrypto(mutable)]
    was_received_by_buyer : bool,
    #[scrypto(mutable)]
    buyer_address : PostalAddress, 
    #[scrypto(mutable)]
    product_has_been_purchased : bool
} 


#[derive(NftData)]
pub struct BuyerNftData {
    product_id : u128,
    fees : Decimal,
    #[scrypto(mutable)]
    has_been_sent_by_seller : bool
} 

blueprint! {
    struct ProductMarketPlace {
        // Vault thats containe the fees
        fees_vault: Vault,
        permanent_seller_nft_id_by_products_id : HashMap<u128,u128>,
        products_for_sale : HashMap<u128, Product>,
        seller_nft_id_by_product_id : HashMap<u128,u128>,
        buyer_nft_id_by_product_id : HashMap<u128, u128>,
        admin_badge: Address,
        seller_buyer_product_minter_badge_vault : Vault,
        seller_permanent_badge_vault : Vault,
        seller_buyer_product_badge_def : ResourceDef,
        seller_permanent_badge_def : ResourceDef,
        vault_by_seller : HashMap<u128,Vault>,
        sell_fees : Decimal,
        buy_fees : Decimal,
        token_type : Address,
        payment_by_buyer_nft_id : HashMap<u128,Vault>, 
    }

    impl ProductMarketPlace {
        // Implement the functions and methods which will manage those resources and data
        
        // This is a function, and can be called directly on the blueprint once deployed
        pub fn new(token_type : Address, 
                   sell_fees : Decimal, 
                   buy_fees : Decimal) -> (Component, Bucket) {
            
                    
            let admin_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
            .initial_supply_fungible(1);

            let seller_buyer_product_minter_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
            .metadata("name", "seller buyer minter product badge")
            .initial_supply_fungible(1);

            let seller_permanent_minter_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
            .metadata("name", "seller minter permanent badge")
            .initial_supply_fungible(1);

            let seller_buyer_product_badge_def = ResourceBuilder::new_non_fungible()
            .metadata("name", "seller or buyer badge ")
            .metadata("description", "this badge give to seller rigth to get postal stamp to send product to buyer. The buyer use this badge for confirming product reception")
            .flags(MINTABLE | INDIVIDUAL_METADATA_MUTABLE | BURNABLE)
            .badge(seller_buyer_product_minter_badge.resource_address(), MAY_MINT | MAY_CHANGE_INDIVIDUAL_METADATA | MAY_BURN)
            .no_initial_supply(); 
            
            let seller_permanent_badge_def = ResourceBuilder::new_non_fungible()
            .metadata("name", "permanent seller badge")
            .metadata("description", "this badge give to seller rigth to collect money")
            .flags(MINTABLE | INDIVIDUAL_METADATA_MUTABLE)
            .badge(seller_permanent_minter_badge.resource_address(), MAY_MINT | MAY_CHANGE_INDIVIDUAL_METADATA)
            .no_initial_supply(); 
            

            let component = Self {
                fees_vault: Vault ::new(token_type),
                permanent_seller_nft_id_by_products_id : HashMap::new(),
                products_for_sale : HashMap::new(),
                seller_nft_id_by_product_id : HashMap::new(),
                buyer_nft_id_by_product_id : HashMap::new(),
                admin_badge: admin_badge.resource_address(),
                seller_buyer_product_minter_badge_vault : Vault::with_bucket(seller_buyer_product_minter_badge),
                seller_permanent_badge_vault : Vault::with_bucket(seller_permanent_minter_badge),
                seller_buyer_product_badge_def : seller_buyer_product_badge_def,
                seller_permanent_badge_def : seller_permanent_badge_def,
                vault_by_seller : HashMap::new(),
                sell_fees : sell_fees,
                buy_fees : buy_fees,
                token_type : token_type,
                payment_by_buyer_nft_id : HashMap::new()
            }
            .instantiate();   

            (component, admin_badge)
        }
        
        pub fn register_as_seller(&mut self) -> Bucket 
        {
            let permanent_nft_id = Uuid::generate();
            let seller_badge = self.seller_permanent_badge_vault.authorize(|auth| {
                return self.seller_permanent_badge_def.mint_nft(permanent_nft_id, PermanentSellerNftData {}, auth);
            }); 
            return seller_badge;
        }

        #[auth(seller_permanent_badge_def)]
        pub fn list_product(&mut self, name : String, 
                                       price : Decimal, 
                                       fees: Bucket) -> (Bucket,Bucket)
        {
            let permanent_seller_nt_id = auth.get_nft_id();
            assert!(fees.resource_address() == self.token_type , "token address must match"); 
            assert!(fees.amount() >= self.sell_fees,"the fees must be >= {}", self.sell_fees); 
            
            let id  = Uuid::generate(); 
            let seller_nft_id = Uuid::generate(); 
            let buyer_nft_id = Uuid::generate(); 
            let product = Product {
                                    id : id,
                                    name : name, 
                                    price : price
                                }; 
           
            self.products_for_sale.insert(id, product);
            self.permanent_seller_nft_id_by_products_id.insert(id , permanent_seller_nt_id); 
        
            let seller_badge = self.seller_buyer_product_minter_badge_vault.authorize(|auth| {
                return self.seller_buyer_product_badge_def.mint_nft(seller_nft_id, SellerNftData {
                     product_id : id,
                     has_been_sent : false,
                     postal_stamp_collected : false,
                     was_received_by_buyer : false,
                     buyer_address : Default :: default(),
                     product_has_been_purchased : false
                    //  products_sales : HashSet::new()
                }, auth)
            }); 

            self.seller_nft_id_by_product_id.insert(id ,seller_nft_id); 
            self.buyer_nft_id_by_product_id.insert(id, buyer_nft_id);

            self.fees_vault.put(fees.take(self.sell_fees)); 

            return (seller_badge, fees);
        }

        pub fn get_available_products(&mut self, page_index : u32) -> Vec<Product>
        {
            const MAX_PRODUCTS_BY_PAGE : u32 = 100; 
            let mut result : Vec<Product> = Vec::new();
            let mut products : Vec<String> = Vec::new();
             
            for key in self.products_for_sale.keys().skip((MAX_PRODUCTS_BY_PAGE * (page_index)) as usize).take(MAX_PRODUCTS_BY_PAGE as usize) {
                let product = self.products_for_sale.get(key).unwrap(); 
                if  self.product_is_available_for_sale(key) {
               
                    result.push( Product {
                                        id : product.id, 
                                        name : product.name.clone(),
                                        price : product.price,
                                }); 
                    products.push(product.to_string()); 
                }
            }
            
            info!("products : {}", products.join(";"));

            return result;
        }

        pub fn buy_product(&mut self, product_id : u128, 
                                      city : String, 
                                      street : String, 
                                      zip_code : String, 
                                      payment : Bucket) -> (Bucket ,Bucket)
        {
            assert!(payment.resource_address() == self.token_type , "token address must match");
            assert!(self.products_for_sale.contains_key(&product_id), "product not found");
            assert!(self.product_is_available_for_sale(&product_id), "product is not available");
            let product = self.products_for_sale.get_mut(&product_id).unwrap(); 
            let total_amount : Decimal = self.buy_fees + (*product).price; 
            assert!(payment.amount() >= total_amount, "payment amount must be greather than or equal {}", total_amount); 

            let buyer_address : PostalAddress = PostalAddress { city : city, 
                                                                street : street, 
                                                                zip_code : zip_code
                                                               } ;   

            let buyer_nft_id = self.buyer_nft_id_by_product_id.get(&product_id).unwrap(); 
            let buyer_badge = self.seller_buyer_product_minter_badge_vault.authorize(|auth| {
                return self.seller_buyer_product_badge_def.mint_nft(*buyer_nft_id, BuyerNftData {
                     product_id : product_id,
                     fees : self.buy_fees,
                     has_been_sent_by_seller : false 
                }, auth)
            }); 

           
            // set buyer address
            let seller_nft_id : u128 = *self.seller_nft_id_by_product_id.get(&product_id).unwrap();
            let mut seller_nft_data  : SellerNftData = self.seller_buyer_product_badge_def.get_nft_data(seller_nft_id);  
            
            seller_nft_data.buyer_address = buyer_address; 
            seller_nft_data.product_has_been_purchased = true; 
            self.seller_buyer_product_minter_badge_vault.authorize(|a|  {
                self.seller_buyer_product_badge_def.update_nft_data(seller_nft_id, seller_nft_data,a); 
            }); 

             // take fees 
            self.fees_vault.put(payment.take(self.buy_fees));
            // hold payments
            self.payment_by_buyer_nft_id.entry(*buyer_nft_id)
                .and_modify(|vault| vault.put(payment.take(product.price)))
                .or_insert(Vault::with_bucket(payment.take(product.price)));

            return (buyer_badge, payment);
        }

        #[auth(seller_buyer_product_badge_def)]
        pub fn collect_postal_stamp(&mut self) -> PostalAddress
        {
            let id = auth.get_nft_id(); 
            let mut seller_nft_data  : SellerNftData = self.seller_buyer_product_badge_def.get_nft_data(id); 
            assert!(!seller_nft_data.postal_stamp_collected, "postale stamp has already collected"); 
            assert!(seller_nft_data.product_has_been_purchased , 
                    "product must be purchased");

            let buyer_postal_address :PostalAddress = PostalAddress {
                 street : seller_nft_data.buyer_address.street.clone(),
                 city : seller_nft_data.buyer_address.city.clone(),
                 zip_code : seller_nft_data.buyer_address.zip_code.clone()
            };

            seller_nft_data.postal_stamp_collected = true; 
            self.seller_buyer_product_minter_badge_vault.authorize(|a|  {
                self.seller_buyer_product_badge_def.update_nft_data(id, seller_nft_data, a); 
            }); 

            return buyer_postal_address;
        }
    
        #[auth(seller_buyer_product_badge_def)]
        pub fn send_product(&mut self)
        {
            let seller_nft_id = auth.get_nft_id(); 
            let mut seller_nft_data  : SellerNftData = self.seller_buyer_product_badge_def.get_nft_data(seller_nft_id); 
            assert!(seller_nft_data.product_has_been_purchased , 
                "product must be purchased");

            let product_id = seller_nft_data.product_id;
            seller_nft_data.has_been_sent = true; 
            self.seller_buyer_product_minter_badge_vault.authorize(|a|  {
                self.seller_buyer_product_badge_def.update_nft_data(seller_nft_id, seller_nft_data,a); 
            }); 

            let buyer_nft_id  : u128 = *self.buyer_nft_id_by_product_id.get(&product_id).unwrap(); 
            let mut buyer_nft_data : BuyerNftData = self.seller_buyer_product_badge_def.get_nft_data(buyer_nft_id);
            buyer_nft_data.has_been_sent_by_seller = true;
            self.seller_buyer_product_minter_badge_vault.authorize(|a| {
                self.seller_buyer_product_badge_def.update_nft_data(buyer_nft_id , buyer_nft_data, a); 
            }); 

        }

        
        pub fn confirm_reception(&mut self , buyer_nft : Bucket)
        {
            assert!(buyer_nft.amount() > Decimal::zero(), "the nft bucket quantity must be greather than or equal 1"); 
            assert!(buyer_nft.resource_address() == self.seller_buyer_product_badge_def.address(), "the nft bucket is not buyer nft");
            let buyer_nft_id = buyer_nft.get_nft_id(); 
            let buyer_nft_data : BuyerNftData = self.seller_buyer_product_badge_def.get_nft_data(buyer_nft_id);
            
            let seller_nft_id = *self.seller_nft_id_by_product_id.get(&buyer_nft_data.product_id).unwrap(); 
            let mut seller_nft_data  : SellerNftData = self.seller_buyer_product_badge_def.get_nft_data(seller_nft_id);  
            seller_nft_data.was_received_by_buyer = true; 
            let product_id = seller_nft_data.product_id; 
            self.seller_buyer_product_minter_badge_vault.authorize(|a|  {
                self.seller_buyer_product_badge_def.update_nft_data(seller_nft_id, seller_nft_data,a); 
            }); 

            let payment = self.payment_by_buyer_nft_id.get_mut(&buyer_nft_id).unwrap();
            let permanent_nft_id = *self.permanent_seller_nft_id_by_products_id.get(&product_id).unwrap(); 
   
            self.vault_by_seller.entry(permanent_nft_id)
                                .and_modify(|vault| vault.put(payment.take_all()))
                                .or_insert(Vault::with_bucket(payment.take_all()));

            self.payment_by_buyer_nft_id.remove(&buyer_nft_id); 

            //burn buyer nft 
            self.seller_buyer_product_minter_badge_vault.authorize(|a|  {
                buyer_nft.burn_with_auth(a);
            }); 
        }
    
        #[auth(seller_permanent_badge_def)]
        pub fn get_available_amount(&self) -> Decimal
        {
           let nft_id = auth.get_nft_id(); 
           if self.vault_by_seller.contains_key(&nft_id)
           {
                let vault = self.vault_by_seller.get(&nft_id).unwrap();
                return vault.amount();   
           } 
           return Decimal::from(0);
        }

        #[auth(seller_permanent_badge_def)]
        pub fn collect_by_seller(&mut self) -> Bucket 
        {
            let nft_id = auth.get_nft_id(); 
            assert!(self.vault_by_seller.contains_key(&nft_id), "Nothing to collect"); 
            let vault = self.vault_by_seller.get(&nft_id).unwrap();
            assert!(vault.amount() > Decimal::from(0) , "Nothing to collect");
            return vault.take_all(); 
        }

        #[auth(admin_badge)]
        pub fn collect_by_admin(&mut self) -> Bucket
        {
            assert!(self.fees_vault.amount() > Decimal::from(0), "Nothing to collect"); 
            return self.fees_vault.take_all();
        }

        pub fn burn_seller_nft(&mut self, seller_nft : Bucket)
        {
            assert!(seller_nft.amount() > Decimal::from(0) , "the nft bucket quantity must be greather than or equal 1");
            assert!(seller_nft.resource_def() == self.seller_buyer_product_badge_def, "bucket must be seller nft"); 
            let nft_id = seller_nft.get_nft_id(); 
            self.seller_nft_id_by_product_id.remove(&nft_id);
            self.seller_buyer_product_minter_badge_vault.authorize(|a| {
                seller_nft.burn_with_auth(a); 
            }); 
        }

        fn product_is_available_for_sale(&self, product_id : &u128) -> bool
        {   
               if self.seller_nft_id_by_product_id.contains_key(product_id) {
                    let seller_nft_id = *self.seller_nft_id_by_product_id.get(product_id).unwrap(); 
                    let  seller_nft_data  : SellerNftData = self.seller_buyer_product_badge_def.get_nft_data(seller_nft_id); 
                    return !seller_nft_data.product_has_been_purchased;
               }           
               return false; 
        }

    }
}
