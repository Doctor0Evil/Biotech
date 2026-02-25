#[proc_macro_attribute]
pub fn bioscale_upgrade(args: TokenStream, input: TokenStream) -> TokenStream {
    use quote::quote;
    use syn::{parse_macro_input, DeriveInput};

    let _args = parse_macro_input!(args as syn::AttributeArgs);
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = ast.ident.clone();

    let expanded = quote! {
        #ast

        impl #ident {
            pub fn descriptor(&self) -> bioscale_upgrade_store::UpgradeDescriptor {
                use bioscale_upgrade_store::{UpgradeDescriptor};
                let mut desc: UpgradeDescriptor = self.into();
                // Hard evidence invariant – prevents "lightweight" upgrades.
                desc.evidence.require_10(desc.id.as_str());
                desc
            }
        }
    };
    expanded.into()
}
