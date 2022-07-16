use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{
    self, bracketed,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, Path, Token, Visibility,
};

mod kw {
    syn::custom_keyword!(send);
    syn::custom_keyword!(recv);
}

struct CodecDef {
    visibility: Visibility,
    name: Ident,
    send_pkts: Vec<Path>,
    recv_pkts: Vec<Path>,
}

impl Parse for CodecDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let visibility: Visibility = input.parse()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        input.parse::<kw::send>()?;
        let send_pkts = {
            let contents;
            bracketed!(contents in input);
            let paths: Punctuated<Path, Token![,]> = contents.parse_terminated(Path::parse)?;
            let mut pkts: Vec<Path> = Vec::new();
            for path in paths {
                pkts.push(path.to_owned());
            }
            pkts
        };
        input.parse::<Token![,]>()?;

        input.parse::<kw::recv>()?;
        let recv_pkts = {
            let contents;
            bracketed!(contents in input);
            let paths: Punctuated<Path, Token![,]> = contents.parse_terminated(Path::parse)?;
            let mut pkts: Vec<Path> = Vec::new();
            for path in paths {
                pkts.push(path.to_owned());
            }
            pkts
        };

        Ok(CodecDef {
            visibility,
            name,
            send_pkts,
            recv_pkts,
        })
    }
}

#[proc_macro]
pub fn define_codec(item: TokenStream) -> TokenStream {
    let codec_def = parse_macro_input!(item as CodecDef);

    let vis = codec_def.visibility;
    let codec_name = codec_def.name;
    let frame_name = Ident::new(&format!("{}FrameRecv", codec_name), codec_name.span());

    let decoder = {
        let frame = {
            let pkts = codec_def.recv_pkts.iter();
            let variants = codec_def
                .recv_pkts
                .iter()
                .map(|p| &p.segments.last().unwrap().ident);
            quote! {
                #vis enum #frame_name {
                    #( #variants(#pkts) ),*
                }
            }
        };

        let id_match_arms = if !codec_def.recv_pkts.is_empty() {
            let pkts = codec_def.recv_pkts.iter();
            let names = codec_def
                .recv_pkts
                .iter()
                .map(|p| &p.segments.last().unwrap().ident);
            quote! {
               #( #pkts::PACKET_ID => Ok(Some(#names(#pkts::from_packet_data(&mut src.reader())?))) ),*,
            }
        } else {
            quote! {}
        };

        quote! {
            #frame

            impl ::tokio_util::codec::Decoder for #codec_name {
                type Item = #frame_name;
                type Error = ::ultimaonline_net::error::Error;

                fn decode(&mut self, src: &mut ::bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
                    use ::bytes::Buf;
                    use ::ultimaonline_net::packets::FromPacketData;
                    use #frame_name::*;

                    // Peek at the first byte
                    if(src.len() < 1) { return Ok(None); }
                    let packet_id = src[0];

                    // match that to the appropriate packet, or error if none matches
                    match packet_id {
                        #id_match_arms
                        _ => Err(::ultimaonline_net::error::Error::data(format!("Unexpected packet ID: {}", packet_id))),
                    }
                }
            }
        }
    };

    let encoder = {
        let trait_name = Ident::new(&format!("{}PacketSend", codec_name), codec_name.span());
        let frame_name = Ident::new(&format!("{}FrameSend", codec_name), codec_name.span());

        let frame = {
            let pkts = codec_def.send_pkts.iter();
            let variants = codec_def
                .send_pkts
                .iter()
                .map(|p| &p.segments.last().unwrap().ident);
            quote! {
                #vis enum #frame_name {
                    #( #variants(#pkts) ),*
                }
            }
        };

        let frame_match_arms = if !codec_def.send_pkts.is_empty() {
            let names = codec_def
                .send_pkts
                .iter()
                .map(|p| &p.segments.last().unwrap().ident);
            quote! {
               #( #names(content) => ::ultimaonline_net::packets::write_packet(content, &mut dst.writer()) ),*,
            }
        } else {
            quote! {}
        };

        let pkts = codec_def.send_pkts.iter();
        quote! {
            #vis trait #trait_name {}

            #( impl #trait_name for #pkts {} )*

            impl<'a, P> ::tokio_util::codec::Encoder<&'a P> for #codec_name
            where
                P: #trait_name + ::serde::ser::Serialize,
                ::ultimaonline_net::packets::Packet<&'a P>: ::std::convert::From<&'a P>,
            {
                type Error = ::ultimaonline_net::error::Error;

                fn encode(&mut self, pkt: &'a P, dst: &mut ::bytes::BytesMut) -> Result<(), Self::Error> {
                    use ::bytes::BufMut;

                    ::ultimaonline_net::packets::write_packet(pkt, &mut dst.writer())
                }
            }

            #frame

            impl<'a> ::tokio_util::codec::Encoder<&'a #frame_name> for #codec_name {
                type Error = ::ultimaonline_net::error::Error;

                fn encode(&mut self, pkt: &'a #frame_name, dst: &mut ::bytes::BytesMut) -> Result<(), Self::Error> {
                    use ::bytes::BufMut;
                    use #frame_name::*;

                    match pkt {
                        #frame_match_arms
                        _ => unreachable!(),
                    }
                }
            }

        }
    };

    let output = quote! {
        #vis struct #codec_name;
        #decoder
        #encoder
    };

    output.into()
}
