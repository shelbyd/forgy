use darling::{ast, util, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[derive(FromDeriveInput)]
#[darling(attributes(forgy))]
struct BuildArgs {
    ident: syn::Ident,

    data: ast::Data<util::Ignored, BuildField>,

    input: Option<syn::Path>,
}

#[derive(FromField)]
#[darling(attributes(forgy))]
struct BuildField {
    ident: Option<syn::Ident>,

    value: Option<syn::Expr>,
}

impl BuildArgs {
    fn main(input: DeriveInput) -> darling::Result<TokenStream> {
        let args = BuildArgs::from_derive_input(&input)?;

        let constructor = quote!(constructor);

        let struct_name = args.ident;
        let (input_generic, input_ty) = match args.input {
            Some(i) => (quote!(), quote!(#i)),
            None => (quote!(<I>), quote!(I)),
        };

        let fields = args.data.take_struct().unwrap();
        let initializer = if fields.is_unit() {
            quote!()
        } else if fields.is_tuple() {
            let fields = fields.into_iter().map(|f| f.construct_expr(&constructor));
            quote!( (#(#fields),*) )
        } else {
            let fields = fields.into_iter().map(|field| {
                let expr = field.construct_expr(&constructor);
                let ident = field.ident.unwrap();
                quote!(#ident: #expr)
            });
            quote!( { #(#fields),* })
        };

        Ok(TokenStream::from(quote::quote! {
            impl #input_generic ::forgy::Build<#input_ty> for #struct_name {
                fn build(#constructor: &mut ::forgy::Container<#input_ty>) -> Self {
                    Self #initializer
                }
            }
        }))
    }
}

impl BuildField {
    fn construct_expr(&self, constructor: &TokenStream) -> TokenStream {
        if let Some(expr) = &self.value {
            return quote!({
                #[allow(unused)]
                let input = #constructor.input();
                #expr
            });
        }

        quote!(#constructor.get())
    }
}

#[proc_macro_derive(Build, attributes(forgy))]
pub fn derive_build(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = syn::parse_macro_input!(input as DeriveInput);
    match BuildArgs::main(derive_input) {
        Ok(t) => proc_macro::TokenStream::from(t),
        Err(e) => proc_macro::TokenStream::from(e.write_errors()),
    }
}
