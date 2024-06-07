extern crate proc_macro;
use proc_macro::TokenStream;


#[proc_macro_attribute]
pub fn show(attr: TokenStream, item: TokenStream) -> TokenStream {
    attr.into_iter().for_each(|x|println!("{x:?}"));
    item.clone().into_iter().for_each(|x|println!("{:?}",x.span()));
    println!("{}",item.to_string());
    item
}
