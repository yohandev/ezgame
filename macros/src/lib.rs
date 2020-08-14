use syn::{ DeriveInput, parse_macro_input };
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Component)]
pub fn derive_cmp(input: TokenStream) -> TokenStream
{
    /// next component identifier
    static mut NEXT_ID: u64 = 0;
    
    // parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // type info
    let (impl_gen, ty_gen, where_clause) = input.generics.split_for_impl();
    let name = input.ident;

    // increment type ID
    unsafe { NEXT_ID += 1 };
    
    // get the current type ID
    let id = unsafe { NEXT_ID };

    // impl trait
    TokenStream::from(quote!
    {
        impl #impl_gen ezgame::Component for #name #ty_gen #where_clause
        {
            const ID: ezgame::CmpId = unsafe { ezgame::CmpId::from_u64(#id) };
        }
    })
}