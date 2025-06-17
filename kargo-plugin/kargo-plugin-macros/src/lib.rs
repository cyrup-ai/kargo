use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn plugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let ident = &func.sig.ident;
    let wrapper = format_ident!("__{}_wrapper", ident);
    let vis = &func.vis;

    TokenStream::from(quote! {
        #func

        #vis fn #wrapper() -> Box<dyn ::kargo_plugin_api::PluginCommand> {
            #ident().build_or_panic()
        }

        #[no_mangle]
        pub extern "C" fn _kargo_plugin_create() -> Box<dyn ::kargo_plugin_api::PluginCommand> {
            #wrapper()
        }
    })
}
