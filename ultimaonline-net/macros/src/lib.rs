use darling::FromMeta;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, *};

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

    let from_value = packet_from_content(&parse_quote! {#main_ident}, packet_id, args.var_size);
    let from_ref = packet_from_content(&parse_quote! {&'a #main_ident}, packet_id, args.var_size);

    let fromdata_impl = content_from_packet(main_ident, packet_id, args.var_size);

    quote! {
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        #main_struct

        impl #main_ident {
            pub const PACKET_ID: u8 = #packet_id;
        }

        #from_value

        #from_ref

        #fromdata_impl
    }
    .into()
}

fn packet_from_content(content_type: &Type, id: u8, var_size: bool) -> proc_macro2::TokenStream {
    let (impl_param, to_size_param) = match content_type {
        Type::Reference(r) => (
            match &r.lifetime {
                Some(l) => quote! {<#l>},
                None => quote! {},
            },
            quote! {val},
        ),
        _ => (quote! {}, quote! {&val}),
    };

    // If this packet has a variable size, generate code to
    // calculate the size and include it when serializing
    let (size_calc, size_field) = if var_size {
        (
            quote! {
                let size = crate::ser::to_size(#to_size_param).expect("Could not serialize packet for size");
                let size = ::core::mem::size_of::<u8>() + // packet id
                            ::core::mem::size_of::<u16>() + // packet size
                            size;
            },
            quote! {
               size: Some(size as u16)
            },
        )
    } else {
        (quote! {}, quote! {size: None})
    };

    quote! {
        impl#impl_param ::std::convert::From<#content_type> for crate::packets::Packet<#content_type> {
            fn from(val: #content_type) -> Self {
                #size_calc

                crate::packets::Packet {
                    id: #id,
                    #size_field,
                    contents: val,
                }
            }
        }
    }
}

fn content_from_packet(name: &syn::Ident, id: u8, var_size: bool) -> proc_macro2::TokenStream {
    let size_check = if var_size {
        quote! {
            // TODO: Actually check this length value
            let _ = reader.read_u16::<BigEndian>().map_err(Error::io)?;
        }
    } else {
        quote! {}
    };

    quote! {
        impl crate::packets::FromPacketData for #name {
            fn from_packet_data<R: ::std::io::Read>(reader: &mut R) -> crate::error::Result<Self> {
                use ::byteorder::{ReadBytesExt, BigEndian};
                use crate::error::Error;

                // Parse out the packet header
                let packet_id = reader.read_u8().map_err(Error::io)?;
                if(packet_id != #id) {
                    return Err(Error::data(format!("Packet ID {:#0X} did not match expected {:#0X}", packet_id, #id)));
                }

                #size_check

                crate::de::from_reader(reader)
            }
        }
    }
}
