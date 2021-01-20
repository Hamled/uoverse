use darling::FromMeta;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemStruct};

#[derive(Debug, FromMeta)]
struct PacketArgs {
    id: u8,
    #[darling(default)]
    var_size: bool,
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
    let (size_field, size_calc) = match args.var_size {
        true => (
            quote! {size: Some(size as u16),},
            quote! {
                let size = crate::ser::to_size(self).expect("Could not serialize packet for size");
                let size = ::core::mem::size_of::<u8>() + // packet id
                           ::core::mem::size_of::<u16>() + // packet size
                           size;
            },
        ),
        false => (quote! {size: None,}, quote! {}),
    };

    let output = quote! {
        #[derive(::serde::Serialize)]
        #main_struct

        impl<'a> ToPacket<'a> for #main_ident {
            fn to_packet(&'a self) -> Packet<'a, Self> {
                #size_calc

                Packet {
                    id: #packet_id,
                    #size_field
                    contents: self,
                }
            }
        }
    };

    output.into()
}
