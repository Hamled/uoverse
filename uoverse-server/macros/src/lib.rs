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
            send_pkts: send_pkts,
            recv_pkts: recv_pkts,
        })
    }
}

#[proc_macro]
pub fn define_codec(item: TokenStream) -> TokenStream {
    let codec_def = parse_macro_input!(item as CodecDef);

    let vis = codec_def.visibility;
    let codec_name = codec_def.name;

    fn frame<'a>(vis: &Visibility, name: &Ident, pkts: &Vec<Path>) -> proc_macro2::TokenStream {
        let variants = pkts.iter().map(|p| &p.segments.last().unwrap().ident);
        let pkts = pkts.iter();

        quote! {
            #vis enum #name {
                #( #variants(#pkts) ),*
            }
        }
    }

    let decoder = if !codec_def.recv_pkts.is_empty() {
        let frame_name = Ident::new(&format!("{}FrameRecv", codec_name), codec_name.span());
        let frame = frame(&vis, &frame_name, &codec_def.recv_pkts);
        let decoder_impl = {
            let pkts = codec_def.recv_pkts.iter();
            let names = pkts.clone().map(|p| &p.segments.last().unwrap().ident);
            quote! {
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
                            #( #pkts::PACKET_ID => Ok(Some(#names(#pkts::from_packet_data(&mut src.reader())?))) ),*,
                            _ => Err(::ultimaonline_net::error::Error::Data),
                        }
                    }
                }
            }
        };

        quote! {
            #frame
            #decoder_impl
        }
    } else {
        quote! {}
    };

    let encoder = if !codec_def.send_pkts.is_empty() {
        let frame_name = Ident::new(&format!("{}FrameSend", codec_name), codec_name.span());
        let frame = frame(&vis, &frame_name, &codec_def.send_pkts);
        let encoder_impl = {
            let pkts = codec_def.send_pkts.iter();
            let names = pkts.clone().map(|p| &p.segments.last().unwrap().ident);
            quote! {
                impl ::tokio_util::codec::Encoder<#frame_name> for #codec_name {
                    type Error = ::ultimaonline_net::error::Error;

                    fn encode(&mut self, item: #frame_name, dst: &mut ::bytes::BytesMut) -> Result<(), Self::Error> {
                        use ::bytes::BufMut;
                        use ::ultimaonline_net::packets::ToPacket;
                        use #frame_name::*;

                        match item {
                            #( #names(pkt) => {
                                pkt.to_packet().to_writer(&mut dst.writer())?;
                            }),*,
                        };
                        Ok(())
                    }
                }
            }
        };

        quote! {
            #frame
            #encoder_impl
        }
    } else {
        quote! {}
    };

    let output = quote! {
        #vis struct #codec_name;
        #decoder
        #encoder
    };

    output.into()
}
