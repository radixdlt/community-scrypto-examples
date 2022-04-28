use sbor::*;
use scrypto::prelude::*;

blueprint! {
    struct CandyDex {        
        // XRD vault.
        collected_xrd: Vault,         
        // Candy Hashmap of vaults.                                   
        candy_vaults: HashMap<ResourceAddress, Vault>,      
        
        // Candy Hashmap with name, symbol, price.              
        candy_map: HashMap<ResourceAddress, (String, String, Decimal)>,        
        // metaBadge Hashmap with entry fee level, metaCandy amount & Candy address.  
        badge_map: HashMap<ResourceAddress, (Decimal, Decimal, ResourceAddress)>,         
        // Candy Hashmap with accrued fee, metaCandy amount & address.
        meta_map: HashMap<ResourceAddress, (Decimal, Decimal, ResourceAddress)>,         
        // metaCandy Hashmap with MetaToken resource adresses. 
        meta: HashMap<ResourceAddress, MetaToken>,  
        
        // Badge to mint and burn metaCandies.                      
        minter_badge: Vault,         
        // Owner badge to determine protocol fee and collect accrued XRD fee.                                     
        owner_badge: ResourceAddress,      
        
        // Protocol XRD fee variable.                           
        xrd_fee: Decimal,   
        // Amount of accrued XRD protocol fee withdrawed by protocol owner.                                      
        xrd_claimed: Decimal,                                     
        // Protocol fee variable.
        fee: Decimal                                              
    }

    impl CandyDex {
        pub fn new(fee: Decimal) -> (ComponentAddress,Bucket) {
            let minter_badge : Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", " MinterBadge ")
                .initial_supply(1);

            let badge_bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", " OwnerBadge ")
                .initial_supply(1);

            let component = Self {
                collected_xrd: Vault::new(RADIX_TOKEN),
                candy_vaults: HashMap::new(),
                candy_map: HashMap::new(),
                badge_map: HashMap::new(),
                meta_map: HashMap::new(),
                meta: HashMap::new(),
                minter_badge: Vault::with_bucket(minter_badge),
                owner_badge: badge_bucket.resource_address(),
                xrd_fee: Decimal::zero(),
                xrd_claimed: Decimal::zero(),
                fee
            }
            .instantiate();

            let access_rules = AccessRules::new()
                .method("claim_xrd_fee", rule!(require(badge_bucket.resource_address())))
                .method("set_fee", rule!(require(badge_bucket.resource_address())))
                .default(rule!(allow_all));

            (component.add_access_check(access_rules).globalize(),badge_bucket)
        }

            // Create a metaBadge and populate a hashmap to associate a determinated entry fee level
            // to a candy stoke event.
            fn add_meta_badge(
                &mut self, 
                symbol: String, 
                candy_addr: ResourceAddress, 
                entry_fee: Decimal, 
                meta_amnt: Decimal
            ) -> Bucket {
                let meta_badge_res_def: ResourceAddress = 
                    ResourceBuilder::new_fungible()
                        .metadata("symbol", format!(" mBadge{}", symbol.clone()))
                        .mintable(rule!(require(self.minter_badge.resource_address())), LOCKED)
                        .burnable(rule!(require(self.minter_badge.resource_address())), LOCKED)
                        .no_initial_supply();
                             
                self.badge_map.insert(meta_badge_res_def,(entry_fee, meta_amnt, candy_addr));

                self.minter_badge.authorize(|| { borrow_resource_manager!(meta_badge_res_def).mint(1) })
            }

            // Populate a hashmap relating a determinated metaBadge address to a metaCandy amount.
            fn meta_amounts(
                &mut self, 
                meta_badge: Bucket, 
                meta_amount: Decimal, 
                flag: Decimal
            ) -> Bucket {
                let meta_badge_addr: ResourceAddress = meta_badge.resource_address().clone();
                
                let (a,mut m_candy_amnt,c) = *self.badge_map.get(&meta_badge_addr).unwrap();
                let entry_fee = a;
                let candy_addr = c;               
                
                if flag == Decimal::zero() {
                    m_candy_amnt = m_candy_amnt+meta_amount;
                    self.badge_map.insert(meta_badge_addr,(entry_fee,m_candy_amnt,candy_addr));
                    meta_badge
                } else if flag == Decimal::one() {
                    assert!( meta_amount <= m_candy_amnt," Let's check passed amount ");
                    m_candy_amnt = m_candy_amnt-meta_amount;
                  
                    if m_candy_amnt == Decimal::zero() {
                        self.badge_map.remove(&meta_badge_addr);
                        CandyDex::badge_burn(self, meta_badge);
                        Bucket::new(RADIX_TOKEN)
                    } else { 
                        self.badge_map.insert(meta_badge_addr,(entry_fee,m_candy_amnt,candy_addr));
                        meta_badge 
                    }
                } else { 
                    std::process::abort() 
                }
            }

            // Create a metaCandy resource relative to a kind of candy provided to protocol by end 
            // users.
            fn add_meta_candy(
                &mut self, 
                name: String, 
                symbol: String, 
                address: ResourceAddress
            ) -> ResourceAddress {
                assert!(!self.meta.contains_key(&address)," Candy already exist ");
                
                let  meta_res_def: ResourceAddress 
                    = ResourceBuilder::new_fungible()
                        .metadata("name", format!(" m{}", name.clone()))
                        .metadata("symbol", format!(" m{}", symbol.clone()))
                        .mintable(rule!(require(self.minter_badge.resource_address())), LOCKED)
                        .burnable(rule!(require(self.minter_badge.resource_address())), LOCKED)
                        .no_initial_supply();

                self.meta.insert(address.clone(),MetaToken::new(name, symbol, address, meta_res_def));
                
                match self.meta_map.get_mut(&address.clone()) {
                    Some((_a,_b,c)) => *c = meta_res_def,
                    None => std::process::abort()                  
                };
               
                meta_res_def
            }

            // Mint a metacandy amount relative to an amount of candy provided to protocol and 
            // update total metaCandy minted amount.
            fn meta_mint(&mut self, candy_amnt: Decimal, candy_address: ResourceAddress) -> Bucket {             
                match self.meta_map.get_mut(&candy_address.clone()) {
                    Some((_a,minted_amnt,_c)) => *minted_amnt = *minted_amnt+candy_amnt,
                    None => std::process::abort()                  
                };

                let m_candy = self.meta.get_mut(&candy_address).unwrap();
               
                self.minter_badge.authorize(|| { borrow_resource_manager!(m_candy.meta_res_def).mint(candy_amnt) })
            }

            // Burn a metacandy amount relative to amount of candy claimed by end user 
            // via "unstock_candy" function.
            fn meta_burn(&mut self, meta_candy: Bucket) {                
                self.minter_badge.authorize(|| {meta_candy.burn()});
            }

            // Burn metaBadge once all relative metaCandy has been claimed by end user 
            // via "unstock_candy" function.
            fn badge_burn(&mut self, meta_badge: Bucket) {
                self.minter_badge.authorize(|| {meta_badge.burn()});
            }

            // Retrieve price of specific candy type.
            fn candyprice(&mut self, candy_addr: ResourceAddress) -> Decimal {
                let price: Decimal;
                
                match self.candy_map.get(&candy_addr) {
                    Some((_a,_b,c)) => price = *c,
                    None => { 
                        info!(" Candy not in stock! ");
                        std::process::abort()
                    }
                };
                
                price
            }

            // Put candy to sell in vault whenever end user specify an exact number of candy to sell 
            // using swap functions.
            fn candyput_pri(
                &mut self, 
                prc_in: Decimal, 
                prc_out: Decimal, 
                addr_in: ResourceAddress, 
                candy: Bucket
            ) -> (Decimal,Decimal) {
                let candy_addr = candy.resource_address();
                let mut candy_amnt = candy.amount();   
                           
                let nmbr = candy.amount()*prc_in/prc_out;
                
                if prc_in == Decimal::zero() { 
                    candy_amnt = CandyDex::adjust_fee(self, candy_amnt); 
                }
                
                let amount = CandyDex::candy_sum(self, candy_amnt, candy_addr, addr_in, 1);

                let v = self.candy_vaults.get_mut(&candy_addr).unwrap();

                v.put(candy);

                (nmbr,amount)
            }

            // Put candy to sell in vault whenever the number of candy to sell is determined by 
            // protocol.
            fn candyput_sec(
                &mut self, 
                amnt: Decimal, 
                addr: ResourceAddress, 
                prc_in: Decimal, 
                prc_out: Decimal, 
                mut candy: Bucket
            ) -> Bucket {
                let candy_addr = candy.resource_address();
                let amount = candy.amount();                
                let amnt_in: Decimal;
                
                if prc_in == Decimal::zero() {
                    amnt_in = CandyDex::candy_sum(self, amnt, addr, candy_addr, 1);
                } else {
                    amnt_in = amnt*prc_in/prc_out;
                }           
                assert!( amnt_in <= amount, "Not enough input amount ");

                let v = self.candy_vaults.get_mut(&candy_addr).unwrap();

                v.put(candy.take(amnt_in));

                candy
            }

            // Take buyed candy from candy vault and increment total accrued fee in relative hashmap.
            fn candytake(&mut self, candy_out_nbr: Decimal, candy_out_addr: ResourceAddress) -> Bucket {
                match self.meta_map.get_mut(&candy_out_addr.clone()) {
                    Some((amnt_fee,_b,_c)) => *amnt_fee = *amnt_fee+candy_out_nbr*self.fee/100,
                    None => std::process::abort()                  
                };

                match self.candy_vaults.get_mut(&candy_out_addr) {
                    Some(vault) => vault.take(candy_out_nbr-(candy_out_nbr*self.fee/100)),
                    None => { info!("Candy not in stock! ");
                              std::process::abort()
                    }
                }
            }

            // Calculate new candy price.
            fn price_mod(
                &mut self, 
                amount: Decimal, 
                address: ResourceAddress, 
                price: Decimal, 
                flag: Decimal
            ) -> Decimal {
                let total_candy = self.candy_vaults.get(&address).unwrap().amount();
            
                if flag == Decimal::one() { 
                    total_candy*price/(total_candy-amount)
                } else { 
                    total_candy*price/(total_candy+amount)
                }
            }

            // Calculate candy output amount.
            fn candy_sum(
                &mut self, 
                amnt_pri: Decimal, 
                addr_pri: ResourceAddress, 
                addr_sec: ResourceAddress, 
                flag: i32
            )-> Decimal {
                let mut amount: Decimal = amnt_pri;

                let price_in: Decimal = CandyDex::candyprice(self, addr_pri);
                let price_out : Decimal = CandyDex::candyprice(self, addr_sec);

                let tot_amnt_out = self.candy_vaults.get(&addr_sec).unwrap().amount();
                let candy_out_amnt = amnt_pri*price_in/price_out;

                match flag {
                    1 => {
                        let price_new = price_out*tot_amnt_out/(tot_amnt_out+candy_out_amnt);
                        price_new*candy_out_amnt/price_out
                    },
                    _ => {
                        if flag == 2 {
                            amount = amnt_pri/price_in;
                        }
                        let price_new = 
                            CandyDex::price_mod(self, amount, addr_pri, price_in, dec!(1));
                        let (name,symbol,_price) = self.candy_map.get(&addr_pri).unwrap();
                        let name_str = name.to_string();
                        let symbol_str = symbol.to_string();
                        self.candy_map.insert(addr_pri,(name_str,symbol_str,price_new));
                        amnt_pri/price_new
                    }
                }
            }

            // Adjust buying exact candy amount neutralizing protocol fee incidence on final amount.
            fn adjust_fee(&mut self, amount_in: Decimal ) -> Decimal {
                amount_in*dec!(100)/(dec!(100)-self.fee)
            }

            // Set protocol fee function whom only protocol owner can succesfully call.
        pub fn set_fee(&mut self, prtcl_fee: Decimal) {
            assert!(prtcl_fee >= dec!(0) && prtcl_fee <= dec!(1)," Let's pass a fee in thousandths! ");
            
            self.fee = prtcl_fee;
            info!(" Protocol fee set to {}% ", self.fee);
        }

            // Claim accrued XRD fee function whom only protocol owner can succesfully call.
        pub fn claim_xrd_fee(&mut self) -> Bucket {
            info!(" Fee value {} XRD ", self.xrd_fee);
        
            let xrd_output: Bucket = self.collected_xrd.take(self.xrd_fee);

            self.xrd_fee = dec!(0);

            self.xrd_claimed += xrd_output.amount();
            info!(" Protocol fee claimed {} XRD ", self.xrd_claimed);
            
            xrd_output
        }

            // Stock candy function callable by an end user wishing to supply unpresent liquidity to 
            // protocol.
        pub fn stock_candy(&mut self, candy: Bucket, new_price: Decimal) -> (Bucket,Bucket) {
            let candy_addr = candy.resource_address();
            let candy_amnt = candy.amount();
            let candy_res_def = candy.resource_address();

            let name = borrow_resource_manager!(candy_res_def).metadata()["name"].clone();
            let symbol = borrow_resource_manager!(candy_res_def).metadata()["symbol"].clone();

            assert!( candy_addr != RADIX_TOKEN," Cannot stock XRD as candy ");
            assert!(new_price > Decimal::zero(), "new price must be a positive value");
            
            if self.candy_map.contains_key(&candy_addr) {
                info!(" Candy already in Vault. Please use restock_candy function ");
                std::process::abort()
            }
            info!(" Added {} {} candy, {} symbol @{}XRD price ", candy_amnt, name, symbol ,new_price);
            
            let v = self.candy_vaults.entry(candy_addr).or_insert(Vault::new(candy_addr));
            
            v.put(candy);
            
            let none: Decimal = Decimal::zero();
            
            self.candy_map.insert(candy_addr,(name.clone(),symbol.clone(),new_price));
            self.meta_map.insert(candy_addr,(none,none,candy_addr));
            
            CandyDex::add_meta_candy(self, name.clone(), symbol.clone(), candy_addr);
            
            let meta_candy: Bucket = CandyDex::meta_mint(self, candy_amnt, candy_addr);           
            
            let meta_amount = meta_candy.amount();
            let meta_badge: Bucket = 
                CandyDex::add_meta_badge(self, symbol, candy_addr, none, meta_amount);
            
            self.badge_map.insert(meta_badge.resource_address().clone(),(none,none,candy_addr));
            
            let output_badge: Bucket = CandyDex::meta_amounts(self, meta_badge, meta_amount, none);
            
            (meta_candy,output_badge)
        }

            // Restock candy function callable by an end user wishing to supply present liquidity to 
            // protocol.
        pub fn restock_candy(&mut self, candy: Bucket) -> (Bucket,Bucket) {
            let candy_addr = candy.resource_address();            
            assert!( candy_addr != RADIX_TOKEN," Cannot stock XRD as candy ");
            
            let amnt = candy.amount();

            match self.candy_map.get(&candy_addr) {
                Some((a,b,c)) => 
                    info!(" Adding {} {} candy, {} symbol, @{} $XRD price ", amnt, a.to_string(), b.to_string(), c),
                _ => { 
                    info!(" Found no candy in Vault. Please use stock_candy function ");
                    std::process::abort()
                }
            }
            
            let v = self.candy_vaults.get_mut(&candy_addr).unwrap();

            v.put(candy);
                
            let meta_candy: Bucket = CandyDex::meta_mint(self, amnt, candy_addr);

            let (accrued_fee,_b,_c) = self.meta_map.get(&candy_addr).unwrap();
            info!(" entry_fee {} ",accrued_fee);
            
            let entry_fee = *accrued_fee;            
            let (_name,symbol,_price) = self.candy_map.get(&candy_addr).unwrap();
            let symbol_str = symbol.to_string();
            let meta_amount = meta_candy.amount();
            let meta_badge: Bucket = 
                CandyDex::add_meta_badge(self, symbol_str, candy_addr, entry_fee, meta_amount);
            
            self.badge_map.insert(meta_badge.resource_address()
                .clone(),(entry_fee,dec!(0),candy_addr));
            
            let output_badge: Bucket = 
                CandyDex::meta_amounts(self, meta_badge, meta_amount, dec!(0));
            
            (meta_candy,output_badge)
        }

            // Unstock candy function callable by an end user wishing to withdraw owned candy 
            // liquidity amount from protocol.
        pub fn unstock_candy(
            &mut self, 
            candy_addr: ResourceAddress, 
            meta_candy: Bucket, 
            meta_badge: Bucket
        ) -> (Bucket,Bucket,Bucket) {
            let badge_amnt: Decimal = meta_badge.amount();
            assert!( badge_amnt >= Decimal::one(), " Please supply meta badge ");
            
            let (accrued_fee,total_minted,meta_address) = self.meta_map.get(&candy_addr).unwrap();
            assert!(meta_address == &meta_candy.resource_address()," Mismatch between Candy & metaCandy! ");
            
            let (entry_fee,_b,candy_address) = 
                self.badge_map.get(&meta_badge.resource_address()).unwrap();
            assert!(candy_address == &candy_addr," MetaBadge address unrecognized! ");
            
            let meta_candy_amnt: Decimal = meta_candy.amount();
            let candy_out_nbr: Decimal;
            let candy_bucket: Bucket;
            let xrd_out: Bucket;
            let delta_fee: Decimal = *accrued_fee-(*entry_fee);
            
            candy_out_nbr = meta_candy_amnt+delta_fee*meta_candy_amnt/(*total_minted);
            
            info!(" total_minted {} ",total_minted);
            info!(" accrued_fee {} ",accrued_fee);
            info!(" entry_fee {} ",entry_fee);
            info!(" delta_fee {} ",delta_fee);
            info!(" candy_out_nbr {} ",candy_out_nbr);
            
            let total_candy = self.candy_vaults.get(&candy_addr).unwrap().amount();
            
            if candy_out_nbr <= total_candy {
                    candy_bucket = match self.candy_vaults.get_mut(&candy_addr) {
                        Some(vault) => vault.take(candy_out_nbr),
                        None => {
                            info!("Candy not in stock !");
                            std::process::abort()
                        }
                    };
                    let zero: Decimal = dec!(0);
                    xrd_out = self.collected_xrd.take(zero);
            }else{  let delta_candy = candy_out_nbr-total_candy;
                    candy_bucket = match self.candy_vaults.get_mut(&candy_addr) {
                        Some(vault) => vault.take(total_candy),
                        None => {
                            info!("Candy not in stock !");
                            std::process::abort()
                        }
                    };
                    let price_in: Decimal = CandyDex::candyprice(self, candy_addr);
                    let xrd_amnt = delta_candy*price_in;
                    assert!( xrd_amnt <= self.collected_xrd.amount(), " Not enough XRD in Vault ");
                    xrd_out = self.collected_xrd.take(xrd_amnt);
            }

            let meta_candy_amnt: Decimal = meta_candy.amount().clone();
            
            CandyDex::meta_burn(self, meta_candy);
            
            let output_badge: Bucket = 
                CandyDex::meta_amounts(self, meta_badge, meta_candy_amnt, dec!(1));
            
            (candy_bucket,xrd_out,output_badge)
        }

            // Retrieve liquidity provider position providing a relative metaBadge as reference.
        pub fn stock_position(&mut self, meta_badge: Proof) {
            let badge_amnt: Decimal = meta_badge.amount();
            assert!( badge_amnt >= Decimal::one(), " Please provide your own metaBadge as reference ");
            
            match self.badge_map.get(&meta_badge.resource_address()) {
                Some((a,b,c)) => { 
                    let (accrued_fee,total_minted,_meta_address) = self.meta_map.get(&c).unwrap();
                    let delta_fee: Decimal = *accrued_fee-(*a);
                    let candy_out_nbr = *b+delta_fee*(*b)/(*total_minted);
                    info!(" accrued_fee {} ",accrued_fee);
                    info!(" total_minted {} ",total_minted);
                    info!(" entry_fee {} ",a);
                    info!(" delta_fee {} ",delta_fee);
                    info!(" candy_out_nbr {} ",candy_out_nbr);
                },
                None => {
                    info!(" No badge's correspondence! ");
                    std::process::abort()
                }
            }
        }

            // Get price, name, symbol of a determinated candy giving his resource address.
        pub fn get_price(&self, candy_addr: ResourceAddress) {
            assert!( candy_addr != RADIX_TOKEN, " XRD is priceless ");
           
            match self.candy_map.get(&candy_addr) {
                Some((a,b,c)) => 
                    info!(" Address: {}, Name: {}, Symbol: {}, Price: @{} $XRD ",candy_addr ,a,b,c),
                None => info!("Could not find candy in stock !")
            }
        }

            // Get reserve amount of a determinated candy giving his resource address.
        pub fn get_reserve(&self, candy_addr: ResourceAddress) {
            match self.candy_map.get(&candy_addr) {
                Some((a,_b,_c)) => { 
                    let total_candy = self.candy_vaults.get(&candy_addr).unwrap().amount();
                    info!(" {} candy reserve amount is {} ", a, total_candy);
                },
                None => {
                    info!(" Could not find candy in stock !");
                    std::process::abort()
                }
            }
        }

            // Get protocol's candies menu.
        pub fn menu(&self){
            for (addr_name,(str_name,str_sym,price)) in self.candy_map.iter() {
                info!(" At address {} we've got {} ({}) candy @ {} $XRD each ", addr_name, str_name, str_sym, price);
            }
        }

            // Get candy sell amount. Use with function "buy_exact_xrd_sell_candy" (bexsc)
        pub fn get_candy_sell_amount_bexsc(
            &mut self, 
            candy_addr: ResourceAddress, 
            xrd_amnt: Decimal
        ) -> Decimal {
            let xrd_amount = CandyDex::adjust_fee(self, xrd_amnt);

            let price = CandyDex::candyprice(self, candy_addr);
            
            let new_price: Decimal = 
                CandyDex::price_mod(self, xrd_amount/price, candy_addr, price, Decimal::zero());
            
            xrd_amount/new_price
        }

            // Get XRD buy amount. Use with function "buy_xrd_sell_exact_candy" (bxsec)
        pub fn get_xrd_buy_amount_bxsec(
            &mut self, 
            candy_addr: ResourceAddress, 
            candy_amnt: Decimal
        ) -> Decimal {
            let price = CandyDex::candyprice(self, candy_addr);
            
            let new_price: Decimal = 
                CandyDex::price_mod(self, candy_amnt, candy_addr, price, Decimal::zero());
            
            (candy_amnt*new_price)-(candy_amnt*new_price)*self.fee/100
        }

            // Get candy buy amount. Use with function "buy_candy_sell_exact_xrd" (bcsex)
        pub fn get_candy_buy_amount_bcsex(
            &mut self, 
            candy_addr: ResourceAddress, 
            xrd_amnt: Decimal
        ) -> Decimal {
            let price = CandyDex::candyprice(self, candy_addr);
            
            let new_price: Decimal = 
                CandyDex::price_mod(self, xrd_amnt/price, candy_addr, price, Decimal::one());
            
            (xrd_amnt/new_price)-(xrd_amnt/new_price)*self.fee/100
        }
            
            // Get XRD sell amount. Use with function "buy_exact_candy_sell_xrd" (becsx)
        pub fn get_xrd_sell_amount_becsx(
            &mut self, 
            candy_addr: ResourceAddress, 
            candy_amnt: Decimal
        ) -> Decimal {
            let candy_amount = CandyDex::adjust_fee(self, candy_amnt);
            
            let price = CandyDex::candyprice(self, candy_addr);
            
            let new_price: Decimal = 
                CandyDex::price_mod(self,  candy_amount, candy_addr, price, Decimal::one());
            
            candy_amount*new_price
        }

            // Get candy sell amount. Use with function "buy_exact_candy_sell_candy" (becsc)
        pub fn get_candy_sell_amount_becsc(
            &mut self, 
            amnt_in: Decimal, 
            addr_in: ResourceAddress, 
            addr_out: ResourceAddress
        ) -> Decimal {
            let amount_in = CandyDex::adjust_fee(self, amnt_in);
            
            CandyDex::candy_sum(self, amount_in, addr_in, addr_out, 1)
        }

            // Get candy buy amount. Use with function "buy_candy_sell_exact_candy"(bcsec)
        pub fn get_candy_buy_amount_bcsec(
            &mut self, 
            addr_in: ResourceAddress, 
            amnt_out: Decimal, 
            addr_out: ResourceAddress
        ) -> Decimal {
            let amount_out = CandyDex::adjust_fee(self, amnt_out);
            
            let amount = CandyDex::candy_sum(self, amount_out, addr_out, addr_in, 1);
            
            amount-amount*self.fee/100
        }

            // Obtain a minimum candy amount in exchange of an exact XRD amount. 
            // Function swap exact XRD for candy.
        pub fn buy_candy_sell_exact_xrd(
            &mut self, 
            min_in: Decimal, 
            addr_in: ResourceAddress, 
            xrd_out: Bucket
        ) -> Bucket {
            let xrd_amnt = xrd_out.amount();
            
            self.collected_xrd.put(xrd_out);
            
            let amount_in = CandyDex::candy_sum(self, xrd_amnt, addr_in, addr_in, 2);
            assert!( amount_in >= min_in, "Not enough candies output amount");
            
            CandyDex::candytake(self, amount_in, addr_in)
        }

            // Obtain a minimum candy amount in exchange of an exact candy amount. 
            // Function swap exact candy for candy.
        pub fn buy_candy_sell_exact_candy(
            &mut self, 
            min_in: Decimal, 
            addr_in: ResourceAddress, 
            candy_out: Bucket
        ) -> Bucket {
            let addr_out = candy_out.resource_address();            
            assert!(addr_in != addr_out," Same candy's address detect! ");
            
            let (_nmbr,amount_in) = 
                CandyDex::candyput_pri(self, Decimal::zero(), Decimal::one(), addr_in, candy_out);

            assert!( amount_in >= min_in, "Not enough candies output amount");
            
            CandyDex::candytake(self, amount_in, addr_in)
        }

            // Obtain a minimum XRD amount in exchange of an exact candy amount. 
            // Function swap exact candy for XRD.
        pub fn buy_xrd_sell_exact_candy(&mut self, xrd_min: Decimal, candy_out: Bucket) -> Bucket {
            let addr: ResourceAddress = candy_out.resource_address();
            
            let price_out: Decimal = CandyDex::candyprice(self, candy_out.resource_address());
            assert!( candy_out.amount()*price_out <= self.collected_xrd.amount(), "Not enough XRD in Vault");
            
            let new_price: Decimal = 
                CandyDex::price_mod(self, candy_out.amount(), addr, price_out, Decimal::zero());
            
            let (name,symbol,_price) = self.candy_map.get(&addr).unwrap();
            let name_str = name.to_string();
            let symbol_str = symbol.to_string();
            self.candy_map.insert(addr,(name_str,symbol_str,new_price));
            
            let (nmbr,_amount_in) = 
                CandyDex::candyput_pri(self, new_price*new_price, new_price, addr, candy_out);
            assert!( nmbr >= xrd_min , "Not enough xrd output amount");
             
            self.xrd_fee = self.xrd_fee+nmbr*self.fee/100;

            self.collected_xrd.take(*&(nmbr-nmbr*self.fee/100))
        }

            // Obtain an exact candy amount in exchange of a maximum XRD amount. 
            // Function swap XRD for exact candy.
        pub fn buy_exact_candy_sell_xrd(
            &mut self, 
            nbr_in: Decimal, 
            addr_in: ResourceAddress, 
            mut xrd_out: Bucket
        ) -> (Bucket,Bucket) {
            let amnt_in = CandyDex::adjust_fee(self, nbr_in);
            
            let mut xrd_amnt = CandyDex::candy_sum(self, amnt_in, addr_in, addr_in, 0);
            
            xrd_amnt = amnt_in*amnt_in/xrd_amnt;
            assert!( xrd_amnt <=  xrd_out.amount(), " Not enough XRD input");
            
            self.collected_xrd.put(xrd_out.take(xrd_amnt));
            
            (CandyDex::candytake(self, amnt_in, addr_in),xrd_out)
        }

            // Obtain an exact candy amount in exchange of a maximum candy amount. 
            // Function swap candy for exact candy.
        pub fn buy_exact_candy_sell_candy(
            &mut self,            
            amnt_in: Decimal, 
            addr_in: ResourceAddress, 
            candy_out: Bucket
        ) -> (Bucket,Bucket) {
            let addr_out = candy_out.resource_address();    

            assert!(addr_in != addr_out," Same candy's address detect! ");
            
            let amount_in = CandyDex::adjust_fee(self, amnt_in);
            
            (
                CandyDex::candyput_sec( self, amount_in, addr_in, Decimal::zero(), Decimal::one(), candy_out),
                CandyDex::candytake(self, amount_in, addr_in)
            )
        }
        
            // Obtain an exact XRD amount in exchange of a maximum candy amount. 
            // Function swap candy for exact XRD.
        pub fn buy_exact_xrd_sell_candy(
            &mut self, 
            xrd_in: Decimal, 
            candy_out: Bucket
        ) -> (Bucket,Bucket) {
            let addr = candy_out.resource_address();
            
            let xrd_input = CandyDex::adjust_fee(self, xrd_in);
            
            let price_out: Decimal = CandyDex::candyprice(self, candy_out.resource_address());
            
            assert!( xrd_in <= self.collected_xrd.amount(), "Not enough XRD in Vault");
            
            let new_price = 
                CandyDex::price_mod(self, xrd_input/price_out, addr, price_out, dec!(0));
            
            match self.candy_map.get_mut(&addr) {
                Some((_a,_b,price)) => *price = new_price,
                None => std::process::abort()                  
            };

            self.xrd_fee = self.xrd_fee+xrd_input*self.fee/100;

            (
                CandyDex::candyput_sec(self, xrd_input, addr, dec!(1), new_price, candy_out),
                self.collected_xrd.take(*&(xrd_input-xrd_input*self.fee/100))
            )
        }

            // Request a flashswap performing a call to an external Component address.
        pub fn flashswap(
            &mut self, 
            amnt_in: Decimal, 
            addr_in: ResourceAddress, 
            bckt_addr: ResourceAddress, 
            ext_addr: ComponentAddress, 
            method: String
        ) -> Bucket {                
            let amount_in = CandyDex::adjust_fee(self, amnt_in);
            
            let token_bucket: Bucket;
            let token_output: Bucket;

            let price_in: Decimal;
            let price_out: Decimal;
            
            if  addr_in == bckt_addr {    
                price_in = Decimal::one();
                price_out = Decimal::one();

                if addr_in == RADIX_TOKEN {
                    token_bucket = self.collected_xrd.take(*&(amount_in-amount_in*self.fee/100));
                    self.xrd_fee = self.xrd_fee+amount_in*self.fee/100;
                } else { 
                    token_bucket = CandyDex::candytake(self, amount_in, addr_in); 
                }

            } else if addr_in == RADIX_TOKEN && addr_in != bckt_addr {
                token_bucket = self.collected_xrd.take(*&(amount_in-amount_in*self.fee/100));
                self.xrd_fee = self.xrd_fee+amount_in*self.fee/100;
                price_in = Decimal::one();
                price_out = CandyDex::candyprice(self, bckt_addr);
            } else if addr_in != bckt_addr && bckt_addr == RADIX_TOKEN {
                token_bucket = CandyDex::candytake(self, amount_in, addr_in);
                price_in = CandyDex::candyprice(self, addr_in);
                price_out = Decimal::one();
            } else if bckt_addr != RADIX_TOKEN {
                token_bucket = CandyDex::candytake(self, amount_in, addr_in);
                price_in = CandyDex::candyprice(self, addr_in);
                price_out = CandyDex::candyprice(self, bckt_addr);
            } else { 
                info!(" Check out addresses! "); 
                std::process::abort(); 
            }
            
            let args = vec![scrypto_encode(&token_bucket),scrypto_encode(&bckt_addr)];
            
            let token_return = borrow_component!(ext_addr).call::<Bucket>(&method.to_string(), args);

            let amount = token_return.amount();
            let nmbr = (amnt_in+amnt_in*self.fee/100)*price_in/price_out;
            
            if bckt_addr != RADIX_TOKEN {   
                let v = self.candy_vaults.get_mut(&bckt_addr).unwrap();

                v.put(token_return);

                token_output = v.take(amount-nmbr);    
            } else { 
                self.collected_xrd.put(token_return);
                token_output = self.collected_xrd.take(*&(amount-nmbr)); 
            }

            if token_output.amount() < dec!(0) {
               info!(" Sorry mate, ain't nothin' to scrape! ");
               std::process::abort();
            }

            token_output
        }
    }
}

// Build a structure and implement it to populate a meta hashmap and relate metaCandy resource with 
// respective Candy resource.
#[derive(Debug, Clone, TypeId, Encode, Decode, Describe, PartialEq, Eq)]
pub struct MetaToken {
    candy_name: String,
    candy_symbol: String,
    candy_address: ResourceAddress,
    meta_res_def: ResourceAddress,
}

impl MetaToken {
    pub fn new(
        candy_name: String,
        candy_symbol: String,
        candy_address: ResourceAddress,
        meta_res_def: ResourceAddress,
    ) -> Self {
        Self {
            candy_name,
            candy_symbol,
            candy_address,
            meta_res_def,
        }
    }
}
