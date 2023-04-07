use scrypto::prelude::*;

#[derive(NonFungibleData)]
pub struct Mail {
    // Subject of mail.
    subject: String,

    // Main body of email, could just be simple text or perhaps formatted with Markdown/HTML, or even encrypted with PGP.
    message: String,

    // A map of attachments, the idea here is that the mapped values would be addresses to downloads such as documents, high-res images etc...
    attachments: HashMap<String, String>, // Perhaps at the given address a cryptographic proof of the NFT is required to download.
}

#[blueprint]
mod rad_mail {
    struct RadMailManager {
        rmail_addr: ResourceAddress,
        auth_badge: Vault,
    }

    impl RadMailManager {
        // Create an instance of RadMail
        pub fn instantiate_rmail() -> ComponentAddress {
            // Create auth badge.
            let auth_badge: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);
            // Create RadMail Resource.
            let rmail_rsrc: ResourceAddress = ResourceBuilder::new_uuid_non_fungible()
                .metadata("name", "RadMail")
                .metadata("description", "A simple mail NFT on the Radix Ledger.")
                .metadata("symbol", "MAIL")
                .mintable(rule!(require(auth_badge.resource_address())), LOCKED)
                .burnable(rule!(require(auth_badge.resource_address())), LOCKED)
                .create_with_no_initial_supply();
            // Instantiate and globalize component.
            Self {
                // Save RadMail resource address.
                rmail_addr: rmail_rsrc,
                // Instantiate auth badge vault from bucket.
                auth_badge: Vault::with_bucket(auth_badge),
            }.instantiate().globalize()
        }

        // Method to create a new RadMail NFT, returns a bucket.
        pub fn compose_mail(&self, subject: String, message: String, attachments: HashMap<String, String>) -> Bucket {
            // Construct NFData
            let data = Mail {
                subject: subject,
                message: message,
                attachments: attachments,
            };
            // Mint and return new NFT with specified data.
            self.auth_badge.authorize(|| {
                borrow_resource_manager!(self.rmail_addr).mint_uuid_non_fungible(data)
            })
        }
    }
}
