use darling::FromMeta;
use proc_macro::{self, TokenStream};
use proc_macro2::{self, Span};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, AttributeArgs, Field, Fields, GenericArgument, Ident,
    ItemStruct, Path, PathArguments, PathSegment, Type,
};

#[derive(Debug, FromMeta)]
struct PacketArgs {
    id: u8,
    #[darling(default)]
    var_size: bool,
}

fn packet_size_calc(item: &ItemStruct) -> proc_macro2::TokenStream {
    let fields = match &item.fields {
        Fields::Unnamed(fields) => &fields.unnamed,
        Fields::Named(fields) => &fields.named,
        Fields::Unit => unimplemented!("Cannot create a packet from unit struct"),
    };

    // HACK: All of this is super fragile
    // we're trying to figure out if the type for a struct field
    // is ::std::vec::Vec looking at the last segment of the path,
    // looking for Vec with a single type argument
    fn vec_type(path: &Path) -> Option<&Type> {
        match path.segments.last() {
            Some(PathSegment { ident, arguments }) => {
                if *ident == Ident::new("Vec", Span::call_site()) {
                    match arguments {
                        PathArguments::AngleBracketed(args) => args
                            .args
                            .iter()
                            .filter_map(|a| match a {
                                GenericArgument::Type(ty) => Some(ty),
                                _ => None,
                            })
                            .next(),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            None => None,
        }
    }

    fn field_size_term(field: &Field) -> proc_macro2::TokenStream {
        let id = &field.ident;
        let ty = &field.ty;
        match ty {
            Type::Path(path) => {
                if let Some(inner_type) = vec_type(&path.path) {
                    // Add 2 for the field holding the length of th vector
                    quote! { (2 + ::core::mem::size_of::<#inner_type>() * self.#id.len()) }
                } else {
                    quote! { ::core::mem::size_of::<#ty>() }
                }
            }
            _ => quote! { ::core::mem::size_of::<#ty>() },
        }
    }

    let terms = fields.iter().map(field_size_term);

    let output = quote! {
        let prefix_size = 1 + 2; // packet id and size field
        let size = (prefix_size + #(#terms)+*) as u16;
    };

    output.into()
}

#[proc_macro_attribute]
pub fn packet(args: TokenStream, item: TokenStream) -> TokenStream {
    let parsed_args = parse_macro_input!(args as AttributeArgs);

    let args = match PacketArgs::from_list(&parsed_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };
    let packet_id = args.id;

    let main_struct = parse_macro_input!(item as ItemStruct);
    let main_ident = &main_struct.ident;

    // If this packet has a variable size, generate code to
    // calculate the size and include it when serializing
    let (size_field_def, size_field, size_calc) = match args.var_size {
        true => (
            quote! {size: u16,},
            quote! {size,},
            packet_size_calc(&main_struct),
        ),
        false => (quote! {}, quote! {}, quote! {}),
    };

    let impl_struct = ItemStruct {
        vis: parse_quote! { pub(self) },
        ident: format_ident!("{}PacketImpl", main_ident),
        ..main_struct.clone()
    };
    let impl_ident = &impl_struct.ident;

    let packet_ident = format_ident!("{}Packet", main_ident);

    let output = quote! {
        #[allow(dead_code)]
        #main_struct

        #[derive(::serde::Serialize)]
        #impl_struct

        #[derive(::serde::Serialize)]
        struct #packet_ident<'a> {
            id: u8,
            #size_field_def
            contents: &'a #impl_ident,
        }

        impl ::serde::ser::Serialize for #main_ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::ser::Serializer,
            {
                #size_calc

                let impl_ptr = self as *const #main_ident as *const #impl_ident;
                let packet = #packet_ident {
                    id: #packet_id,
                    #size_field
                    contents: unsafe { &*impl_ptr },
                };

                packet.serialize(serializer)
            }
        }
    };

    output.into()
}
