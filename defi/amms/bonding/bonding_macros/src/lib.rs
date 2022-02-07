extern crate proc_macro;
use proc_macro::{TokenStream};
use proc_macro2::{Span};
use quote::{quote};
use syn::{parse_macro_input, ItemImpl, ImplItem, ImplItemMethod, ItemTrait, TraitItem, Ident, Block, Visibility, VisPublic, token::Pub};
//use syn::{ReturnType};

/// a macro to generate an empty blueprint just to get the stub functions for inter-blueprint calls
/// avoids manually specifying an ABI, and allows a concise definition using trait syntax
#[proc_macro_attribute]
pub fn blueprint_stub(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    // parse the trait
    let mut input = parse_macro_input!(input as ItemTrait);
    // grap the identifier
    let ident = input.ident.clone();
    // create the skeleton for impl to be generated
    let the_impl = TokenStream::from(quote! {
        #[allow(unused)]
        impl #ident {

        }
    });
    let mut the_impl = parse_macro_input!(the_impl as ItemImpl);
    // iterate over the methods creating the implementation and filling the impl items
    // let decimal_ident = Ident::new(&format!("Decimal"), Span::call_site()); // used to find Decimal type
    let items = &mut input.items;
    for i in 0..items.len() {
        match items[i] {
            TraitItem::Method(ref mut method) => {
                //method.semi_token = None;
                //method.sig.ident =  Ident::new(&format!("{}_{}", ident, i), Span::call_site());
                let empty_block = TokenStream::from(quote! {
                    {
                        panic!("only exists for stub, do not call");
                    }
                });
                /*
                let block = match method.sig.output {
                    ReturnType::Default => { empty_block },
                    ReturnType::Type(_, ref box_type) => {
                        let is_decimal: bool = match **box_type {
                            syn::Type::Path(ref type_path) => {
                                type_path.path.is_ident(&decimal_ident)
                            },
                            _ => false
                        };
                        if is_decimal {
                            TokenStream::from(quote! {
                                { // block returning the default for Decimal
                                    0.into()
                                    //let result: #box_type = Default::default();
                                    //result
                                }
                            })
                        } else {
                            TokenStream::from(quote! {
                                { // block returning the default for the type
                                    let result: #box_type = Default::default();
                                    result
                                }
                            })
                        }
                    }
                };
                let block = parse_macro_input!(block as Block);
                */
                let block = parse_macro_input!(empty_block as Block);
                method.default = Some(block.clone());
                // now update the vec of ImplItem::Method from the TraitItem::Method 
                the_impl.items.push(
                    ImplItem::Method(ImplItemMethod {
                        attrs: the_impl.attrs.clone(), // copy the #[allow(unused)] to each method
                        vis: Visibility::Public(
                            VisPublic{
                                pub_token: Pub{
                                    span: Span::call_site()
                                }
                            }
                        ),
                        defaultness: None,
                        sig: method.sig.clone(),
                        block: block
                    }
                )

                );
            },
            _ => {}
        }
    }

    // create the final output
    // a blueprint! with the geenerated impl, but all inside a module so the code is not callable
    // then reexport only the stubs
    let mod_name = Ident::new(&format!("internal_{}", ident), Span::call_site());
    TokenStream::from(quote!{
        mod #mod_name {
        use super::*;
        blueprint! {
            struct #ident {}
            #the_impl
        }
        }
        // reexport the stub only
        pub use #mod_name::#ident;
    })
}