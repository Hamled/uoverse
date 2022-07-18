use futures::sink::SinkExt;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::mpsc,
};
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;
use ultimaonline_net::{
    error::{Error, Result},
    packets::Packet,
};

pub mod codecs;
use codecs::CompressionCodec;

pub trait AsyncIo = AsyncRead + AsyncWrite + Unpin + Send + Sync;

// Ensures that the FSM must start with the initial state (Connected)
struct GameSequencer;

pub struct Connected<Io: AsyncIo> {
    sequencer: GameSequencer,
    framer: Framed<Io, codecs::Connected>,
}

impl<Io: AsyncIo> Connected<Io> {
    pub async fn recv(&mut self) -> Result<Option<codecs::ConnectedFrameRecv>> {
        self.framer.try_next().await
    }

    pub fn new(io: Io) -> Self {
        Self {
            sequencer: GameSequencer {},
            framer: Framed::new(io, codecs::Connected),
        }
    }
}

pub struct CharList<Io: AsyncIo> {
    sequencer: GameSequencer,
    framer: Framed<Io, CompressionCodec<codecs::CharList>>,
}

impl<Io: AsyncIo> CharList<Io> {
    pub async fn send<'a, P>(&mut self, pkt: &'a P) -> Result<()>
    where
        P: codecs::CharListPacketSend + ::serde::ser::Serialize,
        Packet<&'a P>: From<&'a P>,
    {
        self.framer.send(pkt).await
    }
}

impl<Io: AsyncIo> From<Connected<Io>> for CharList<Io> {
    fn from(val: Connected<Io>) -> Self {
        Self {
            sequencer: val.sequencer,
            framer: val
                .framer
                .map_codec(|_| CompressionCodec::new(codecs::CharList {})),
        }
    }
}

pub struct ClientVersion<Io: AsyncIo> {
    sequencer: GameSequencer,
    framer: Framed<Io, codecs::ClientVersion>,
}

impl<Io: AsyncIo> ClientVersion<Io> {
    pub async fn recv(&mut self) -> Result<Option<codecs::ClientVersionFrameRecv>> {
        self.framer.try_next().await
    }
}

impl<Io: AsyncIo> From<CharList<Io>> for ClientVersion<Io> {
    fn from(val: CharList<Io>) -> Self {
        Self {
            sequencer: val.sequencer,
            framer: val.framer.map_codec(|_| codecs::ClientVersion),
        }
    }
}

pub struct CharSelect<Io: AsyncIo> {
    sequencer: GameSequencer,
    framer: Framed<Io, codecs::CharSelect>,
}

impl<Io: AsyncIo> CharSelect<Io> {
    pub async fn recv(&mut self) -> Result<Option<codecs::CharSelectFrameRecv>> {
        self.framer.try_next().await
    }
}

impl<Io: AsyncIo> From<ClientVersion<Io>> for CharSelect<Io> {
    fn from(val: ClientVersion<Io>) -> Self {
        Self {
            sequencer: val.sequencer,
            framer: val.framer.map_codec(|_| codecs::CharSelect),
        }
    }
}

pub struct CharLogin<Io: AsyncIo> {
    sequencer: GameSequencer,
    framer: Framed<Io, CompressionCodec<codecs::CharLogin>>,
}

impl<Io: AsyncIo> CharLogin<Io> {
    pub async fn send<'a, P>(&mut self, pkt: &'a P) -> Result<()>
    where
        P: codecs::CharLoginPacketSend + ::serde::ser::Serialize,
        Packet<&'a P>: From<&'a P>,
    {
        self.framer.send(pkt).await
    }
}

impl<Io: AsyncIo> From<CharSelect<Io>> for CharLogin<Io> {
    fn from(val: CharSelect<Io>) -> Self {
        Self {
            sequencer: val.sequencer,
            framer: val
                .framer
                .map_codec(|_| CompressionCodec::new(codecs::CharLogin)),
        }
    }
}

pub struct InWorld<Io: AsyncIo> {
    #[allow(dead_code)]
    sequencer: GameSequencer,
    framer: Framed<Io, CompressionCodec<codecs::InWorld>>,
}

impl<Io: AsyncIo> InWorld<Io> {
    pub async fn send<'a, P>(&mut self, pkt: &'a P) -> Result<()>
    where
        P: codecs::InWorldPacketSend + ::serde::ser::Serialize,
        Packet<&'a P>: From<&'a P>,
    {
        self.framer.send(pkt).await
    }

    pub async fn send_frame<'a>(&mut self, pkt: &'a codecs::InWorldFrameSend) -> Result<()> {
        self.framer.send(pkt).await
    }

    pub async fn recv(&mut self) -> Result<Option<codecs::InWorldFrameRecv>> {
        self.framer.try_next().await
    }
}

impl<Io: AsyncIo> From<CharLogin<Io>> for InWorld<Io> {
    fn from(val: CharLogin<Io>) -> Self {
        Self {
            sequencer: val.sequencer,
            framer: val
                .framer
                .map_codec(|_| CompressionCodec::new(codecs::InWorld {})),
        }
    }
}

pub trait ClientSender {
    type SendItem;
    fn send(&mut self, item: Self::SendItem) -> Result<()>;
}

pub trait ClientReceiver {
    type RecvItem;
    fn recv(&mut self) -> Result<Self::RecvItem>;

    fn close(&mut self);
}

pub struct Client {
    pub receiver: mpsc::UnboundedReceiver<codecs::InWorldFrameSend>,
    pub sender: mpsc::UnboundedSender<codecs::InWorldFrameRecv>,
}

impl ClientSender for Client {
    type SendItem = codecs::InWorldFrameRecv;
    fn send(&mut self, item: Self::SendItem) -> Result<()> {
        self.sender
            .send(item)
            .map_err(|_| Error::Message("TODO: MPSC send error".to_string()))
    }
}

impl ClientReceiver for Client {
    type RecvItem = codecs::InWorldFrameSend;
    fn recv(&mut self) -> Result<Self::RecvItem> {
        self.receiver
            .try_recv()
            .map_err(|_| Error::Message("TODO: MPSC recv error".to_string()))
    }

    fn close(&mut self) {
        self.receiver.close();
    }
}

pub struct WorldClient {
    pub receiver: mpsc::UnboundedReceiver<codecs::InWorldFrameRecv>,
    pub sender: mpsc::UnboundedSender<codecs::InWorldFrameSend>,
}

impl ClientSender for WorldClient {
    type SendItem = codecs::InWorldFrameSend;
    fn send(&mut self, item: Self::SendItem) -> Result<()> {
        self.sender
            .send(item)
            .map_err(|_| Error::Message("TODO: MPSC send error".to_string()))
    }
}

impl ClientReceiver for WorldClient {
    type RecvItem = codecs::InWorldFrameRecv;
    fn recv(&mut self) -> Result<Self::RecvItem> {
        self.receiver
            .try_recv()
            .map_err(|_| Error::Message("TODO: MPSC recv error".to_string()))
    }

    fn close(&mut self) {
        self.receiver.close();
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
