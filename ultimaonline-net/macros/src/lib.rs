use darling::FromMeta;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, *};

#[derive(Debug, FromMeta)]
enum PacketArgs {
    Fixed { id: u8, size: usize },
    Var { id: u8 },
    Extended { id: u16 },
}

#[proc_macro_attribute]
pub fn packet(args: TokenStream, item: TokenStream) -> TokenStream {
    use PacketArgs::*;

    let parsed_args = parse_macro_input!(args as AttributeArgs);

    let args = match PacketArgs::from_list(&parsed_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let main_struct = parse_macro_input!(item as ItemStruct);
    let main_ident = &main_struct.ident;

    let from_value = packet_from_content(&parse_quote! {#main_ident}, &args);
    let from_ref = packet_from_content(&parse_quote! {&'a #main_ident}, &args);

    let fromdata_impl = content_from_packet(main_ident, &args);

    let (packet_id, extended_id) = match args {
        Fixed { id, .. } | Var { id } => (quote! {#id}, quote! {None}),
        Extended { id } => (
            quote! {crate::packets::EXTENDED_PACKET_ID},
            quote! {
                Some(#id)
            },
        ),
    };

    let packet_size = match args {
        Fixed { size, .. } => quote!(Some(#size)),
        _ => quote!(None),
    };

    quote! {
        #[derive(Clone, Debug, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
        #main_struct

        impl #main_ident {
            pub const PACKET_ID: u8 = #packet_id;
            pub const EXTENDED_ID: Option<u16> = #extended_id;
            pub const SIZE: Option<usize> = #packet_size;
        }

        #from_value

        #from_ref

        #fromdata_impl
    }
    .into()
}

fn packet_from_content(content_type: &Type, args: &PacketArgs) -> proc_macro2::TokenStream {
    use PacketArgs::*;

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
    let (size_calc, size_field) = match args {
        Fixed { .. } => (quote! {}, quote! {size: None}),
        _ => (
            quote! {
                let size = crate::ser::to_size(#to_size_param).expect("Could not serialize packet for size");
                let size = ::core::mem::size_of::<u8>() + // packet id
                           ::core::mem::size_of::<u16>() + // packet size
                           size;
            },
            quote! {
               size: Some(size as u16)
            },
        ),
    };

    let from_type = content_type;
    let (size_calc, content_type, content_val) = match args {
        Extended { id } => (
            quote! {
                #size_calc
                let size = ::core::mem::size_of::<u16>() + // extended id
                           size;
            },
            quote! {(u16, #content_type)},
            quote! {(#id, val)},
        ),
        _ => (size_calc, quote! {#content_type}, quote! {val}),
    };

    let id = match args {
        Fixed { id, .. } | Var { id } => quote! {#id},
        Extended { .. } => quote! {crate::packets::EXTENDED_PACKET_ID},
    };

    quote! {
        impl#impl_param crate::packets::IntoPacket for #from_type {
            type Content = #content_type;
        }

        impl#impl_param ::std::convert::From<#from_type> for crate::packets::Packet<#content_type> {
            fn from(val: #from_type) -> Self {
                #size_calc

                crate::packets::Packet {
                    id: #id,
                    #size_field,
                    contents: #content_val,
                }
            }
        }
    }
}

fn content_from_packet(name: &syn::Ident, args: &PacketArgs) -> proc_macro2::TokenStream {
    use PacketArgs::*;

    let size_check = match args {
        Fixed { .. } => quote! {
            let size = #name::SIZE.unwrap();
        },
        _ => quote! {
            let size = reader.read_u16::<BigEndian>()? as usize
                - 1  // Packet ID
                - 2; // Size field
        },
    };

    let read_extended_id = match args {
        Extended { id } => quote! {
            // Parse out the extended id
            let extended_id = reader.read_u16::<BigEndian>()?;
            if(extended_id != #id) {
                return Err(Error::data(format!("packet extended id {:#0X} did not match expected {:#0X}", extended_id, #id)));
            }

            let size = size - 2; // Extended ID
        },
        _ => quote! {},
    };

    let id = match args {
        Fixed { id, .. } | Var { id } => quote! {#id},
        Extended { .. } => quote! {crate::packets::EXTENDED_PACKET_ID},
    };

    quote! {
        impl crate::packets::FromPacketData for #name {
            fn from_packet_data<R: ::std::io::BufRead>(reader: &mut R) -> crate::error::Result<Self> {
                use ::byteorder::{ReadBytesExt, BigEndian};
                use crate::error::Error;

                // Parse out the packet header
                let packet_id = reader.read_u8()?;
                if(packet_id != #id) {
                    return Err(Error::data(format!("packet id {:#0X} did not match expected {:#0X}", packet_id, #id)));
                }

                #size_check

                #read_extended_id

                crate::de::from_reader(reader, size)
            }
        }
    }
}
