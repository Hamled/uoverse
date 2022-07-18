use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};
use ultimaonline_net::packets::*;

use crate::macros::define_codec;

define_codec! {
    pub Connected,
    send [],
    recv [
        char_select::GameLogin,
    ]
}

define_codec! {
    pub CharList,
    send [
        char_select::Features,
        char_select::CharList,
        char_select::VersionReq,
    ],
    recv []
}

define_codec! {
    pub ClientVersion,
    send [],
    recv [
        char_select::VersionResp,
    ]
}

define_codec! {
    pub CharSelect,
    send [],
    recv [
        char_select::CreateCharacter,
    ]
}

define_codec! {
    pub CharLogin,
    send [
        char_login::LoginConfirmation,
        char_login::CharStatus,
        char_login::LoginComplete,
    ],
    recv []
}

define_codec! {
    pub InWorld,
    send [
        mobile::Appearance,
        mobile::MobLightLevel,
        mobile::State,
        movement::Success,
        movement::Reject,
        network::PingAck,
        world::WorldLightLevel,
    ],
    recv [
        action::ClickUse,
        action::ClickLook,
        char_select::VersionResp,
        chat::OpenWindow,
        client_info::Flags,
        client_info::Language,
        client_info::WindowSize,
        client_info::ViewRange,
        housing::ShowPublicContent,
        mobile::Query,
        movement::Request,
        network::PingReq
    ]
}

pub struct CompressionCodec<C> {
    codec: C,
}

impl<C> CompressionCodec<C> {
    pub fn new(codec: C) -> Self {
        Self { codec }
    }
}

impl<'a, I, C: Encoder<&'a I>> Encoder<&'a I> for CompressionCodec<C> {
    type Error = C::Error;

    fn encode(&mut self, pkt: &'a I, dst: &mut BytesMut) -> std::result::Result<(), Self::Error> {
        use bytes::BufMut;
        use ultimaonline_net::compression::huffman;

        let mut tmp = BytesMut::with_capacity(64);
        self.codec.encode(&pkt, &mut tmp)?;
        let compressed = huffman::compress(&*tmp);

        dst.put(compressed.as_slice());

        Ok(())
    }
}

impl<C: Decoder> Decoder for CompressionCodec<C> {
    type Error = C::Error;
    type Item = C::Item;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> std::result::Result<Option<Self::Item>, Self::Error> {
        self.codec.decode(src)
    }
}
