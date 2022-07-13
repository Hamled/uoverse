use bytes::BytesMut;
use futures::sink::SinkExt;
use std::marker::PhantomData;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::StreamExt;
use tokio_util::codec::{Encoder, FramedRead, FramedWrite};
use ultimaonline_net::{error::Result, packets::ToPacket};

pub trait AsyncIo = AsyncRead + AsyncWrite + Unpin + Send + Sync;

// Ensures that the FSM must start with the initial state (Connected)
struct GameSequencer;

pub struct Connected<Io: AsyncIo> {
    io: Io,
    sequencer: GameSequencer,
}

impl<Io: AsyncIo> Connected<Io> {
    pub async fn recv(&mut self) -> Result<Option<codecs::ConnectedFrame>> {
        let mut reader = FramedRead::new(&mut self.io, codecs::Connected {});
        reader.try_next().await
    }

    pub fn new(io: Io) -> Self {
        Self {
            io,
            sequencer: GameSequencer {},
        }
    }
}

pub struct CharList<Io: AsyncIo> {
    io: Io,
    sequencer: GameSequencer,
}

impl<Io: AsyncIo> CharList<Io> {
    pub async fn send<'a, P>(&mut self, pkt: &'a P) -> Result<()>
    where
        P: codecs::CharListEncode + ToPacket<'a> + ::serde::ser::Serialize,
    {
        let mut writer = FramedWrite::new(&mut self.io, CompressionCodec::new(codecs::CharList {}));
        writer.send(pkt).await
    }
}

impl<Io: AsyncIo> From<Connected<Io>> for CharList<Io> {
    fn from(val: Connected<Io>) -> Self {
        Self {
            io: val.io,
            sequencer: val.sequencer,
        }
    }
}

pub struct ClientVersion<Io: AsyncIo> {
    io: Io,
    sequencer: GameSequencer,
}

impl<Io: AsyncIo> ClientVersion<Io> {
    pub async fn recv(&mut self) -> Result<Option<codecs::ClientVersionFrame>> {
        let mut reader = FramedRead::new(&mut self.io, codecs::ClientVersion {});
        reader.try_next().await
    }

    pub fn new(io: Io) -> Self {
        Self {
            io,
            sequencer: GameSequencer {},
        }
    }
}

impl<Io: AsyncIo> From<CharList<Io>> for ClientVersion<Io> {
    fn from(val: CharList<Io>) -> Self {
        Self {
            io: val.io,
            sequencer: val.sequencer,
        }
    }
}

pub struct CharSelect<Io: AsyncIo> {
    io: Io,
    sequencer: GameSequencer,
}

impl<Io: AsyncIo> CharSelect<Io> {
    pub async fn recv(&mut self) -> Result<Option<codecs::CharSelectFrame>> {
        let mut reader = FramedRead::new(&mut self.io, codecs::CharSelect {});
        reader.try_next().await
    }

    pub fn new(io: Io) -> Self {
        Self {
            io,
            sequencer: GameSequencer {},
        }
    }
}

impl<Io: AsyncIo> From<ClientVersion<Io>> for CharSelect<Io> {
    fn from(val: ClientVersion<Io>) -> Self {
        Self {
            io: val.io,
            sequencer: val.sequencer,
        }
    }
}

pub struct CharLogin<Io: AsyncIo> {
    io: Io,
    sequencer: GameSequencer,
}

impl<Io: AsyncIo> CharLogin<Io> {
    pub async fn send<'a, P>(&mut self, pkt: &'a P) -> Result<()>
    where
        P: codecs::CharLoginEncode + ToPacket<'a> + ::serde::ser::Serialize,
    {
        let mut writer =
            FramedWrite::new(&mut self.io, CompressionCodec::new(codecs::CharLogin {}));
        writer.send(pkt).await
    }
}

impl<Io: AsyncIo> From<CharSelect<Io>> for CharLogin<Io> {
    fn from(val: CharSelect<Io>) -> Self {
        Self {
            io: val.io,
            sequencer: val.sequencer,
        }
    }
}

pub struct InWorld<Io: AsyncIo> {
    io: Io,
    #[allow(dead_code)]
    sequencer: GameSequencer,
}

impl<Io: AsyncIo> InWorld<Io> {
    pub async fn send<'a, P>(&mut self, pkt: &'a P) -> Result<()>
    where
        P: codecs::InWorldEncode + ToPacket<'a> + ::serde::ser::Serialize,
    {
        let mut writer = FramedWrite::new(&mut self.io, CompressionCodec::new(codecs::InWorld {}));
        writer.send(pkt).await
    }
}

impl<Io: AsyncIo> From<CharLogin<Io>> for InWorld<Io> {
    fn from(val: CharLogin<Io>) -> Self {
        Self {
            io: val.io,
            sequencer: val.sequencer,
        }
    }
}

pub mod codecs {
    use crate::macros::define_codec;
    use ultimaonline_net::packets::*;

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
        ],
        recv []
    }

    define_codec! {
        pub InWorld,
        send [
            char_login::LoginComplete,
        ],
        recv []
    }
}

pub struct CompressionCodec<'a, I, C: Encoder<&'a I>> {
    codec: C,
    item_type: PhantomData<&'a I>,
}

impl<'a, I, C: Encoder<&'a I>> CompressionCodec<'a, I, C> {
    fn new(codec: C) -> Self {
        Self {
            codec,
            item_type: PhantomData {},
        }
    }
}

impl<'a, I, C: Encoder<&'a I>> Encoder<&'a I> for CompressionCodec<'a, I, C> {
    type Error = C::Error;

    fn encode(&mut self, pkt: &'a I, dst: &mut BytesMut) -> std::result::Result<(), Self::Error> {
        use ::bytes::BufMut;
        use ultimaonline_net::compression::huffman;

        let mut tmp = BytesMut::with_capacity(64);
        self.codec.encode(&pkt, &mut tmp)?;
        let compressed = huffman::compress(&*tmp);

        dst.put(compressed.as_slice());

        Ok(())
    }
}

// Enable all flags except:
// Third Dawn =                                                0100
// 6th Char Slot =                                        0010 0000
// 8th Age =                                         0001 0000 0000
// 10th Age =                                        0100 0000 0000
// Increased Storage =                               1000 0000 0000
// Roleplay Faces =                             0010 0000 0000 0000
// Trial Account =                              0100 0000 0000 0000
pub const FEATURES: u32 = 0b0000_0000_1111_1111_1001_0010_1101_1011;

// Disable all flags except:
// Context Menus =                                          1000
// AOS Expansion =                                     0010 0000
// SE Expansion =                                      1000 0000
// ML Expansion =                                 0001 0000 0000
// Seventh Char Slot =                       0001 0000 0000 0000
pub const FLAGS: u32 = 0b0000_0000_0000_0000_0001_0001_1010_1000;
