use quote::{ToTokens, quote};
use std::borrow::Cow;

pub mod prelude {
    pub use crate::{CatCow, CatCowMap};
}

pub type CatCowMap = io_cat::CatMap<Cow<'static, str>>;
pub struct CatCow(pub CatCowMap);

impl ToTokens for CatCow {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let entries = self.0.iter().map(|(key, cat_kind)| {
            let key_str = key.as_ref();
            quote! {
                (::std::borrow::Cow::from(#key_str), #cat_kind)
            }
        });
        let expanded = quote! {
            Some(::std::collections::HashMap::from([#(#entries),*]))
        };
        tokens.extend(expanded);
    }
}
