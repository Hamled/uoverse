use darling::FromMeta;
use proc_macro::{self, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, AttributeArgs, ItemStruct};

#[derive(Debug, FromMeta)]
struct PacketArgs {
    id: u8,
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
            contents: &'a #impl_ident,
        }

        impl ::serde::ser::Serialize for #main_ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::ser::Serializer,
            {
                let impl_ptr = self as *const #main_ident as *const #impl_ident;
                let packet = #packet_ident {
                    id: #packet_id,
                    contents: unsafe { &*impl_ptr },
                };

                packet.serialize(serializer)
            }
        }
    };

    output.into()
}
