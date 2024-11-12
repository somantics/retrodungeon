use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(EntityTemplateEnum)]
pub fn derive_entity_template(input: TokenStream) -> TokenStream { 
    let ast = syn::parse(input).unwrap();

    impl_derive_template(&ast)
}

fn impl_derive_template(ast: &syn::DeriveInput) -> TokenStream {
    let syn::Data::Enum(value) = &ast.data else {
        return TokenStream::new();
    };

    let variants = value.variants
        .iter()
        .map(|variant| &variant.ident);

    let gen = quote! {
        impl EntityTemplate for EntityTemplateEnum {
            fn add_components(&self, entity: usize, world: &mut World, depth: u32, resources: &ResourceManager) -> crate::spawning::Result<()> {
                match self {
                    #(
                        Self::#variants(template) => template.add_components(entity, world, depth, resources),
                    )*
                }
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(EventResponse)]
pub fn derive_event_response(input: TokenStream) -> TokenStream { 
    let ast = syn::parse(input).unwrap();

    impl_derive_event_response(&ast)
}

fn impl_derive_event_response(ast: &syn::DeriveInput) -> TokenStream {
    let ident = &ast.ident;
    let gen = quote! {
        impl EventResponse for #ident {
            fn respond(
                &self,
                event: &dyn Event<Response = Self>,
                response_data: ResponseArguments,
            ) -> Result<()> {
                let callable = self.response.get_callable()?;
                (callable)(event, response_data, &self.args)
            }
        }
    };
    gen.into()
}


#[proc_macro_derive(TagComponent)]
pub fn derive_tag_component(input: TokenStream) -> TokenStream { 
    let ast = syn::parse(input).unwrap();

    impl_derive_tag_component(&ast)
}

fn impl_derive_tag_component(ast: &syn::DeriveInput) -> TokenStream {
    let ident = &ast.ident;
    let gen = quote! {
        impl Tag for #ident {}
    };
    gen.into()
}