use futures::sink::SinkExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;
use ultimaonline_net::{error::Result, packets::Packet};

pub trait AsyncIo = AsyncRead + AsyncWrite + Unpin + Send + Sync;

// Ensures that the FSM must start with the initial state (Connected)
struct LoginSequencer;

pub struct Connected<Io: AsyncIo> {
    sequencer: LoginSequencer,
    framer: Framed<Io, codecs::Connected>,
}

impl<Io: AsyncIo> Connected<Io> {
    pub async fn recv(&mut self) -> Result<Option<codecs::ConnectedFrame>> {
        self.framer.try_next().await
    }

    pub fn new(io: Io) -> Self {
        Self {
            sequencer: LoginSequencer {},
            framer: Framed::new(io, codecs::Connected {}),
        }
    }
}

pub struct Hello<Io: AsyncIo> {
    sequencer: LoginSequencer,
    framer: Framed<Io, codecs::Hello>,
}

impl<Io: AsyncIo> Hello<Io> {
    pub async fn recv(&mut self) -> Result<Option<codecs::HelloFrame>> {
        self.framer.try_next().await
    }
}

impl<Io: AsyncIo> From<Connected<Io>> for Hello<Io> {
    fn from(val: Connected<Io>) -> Self {
        Self {
            sequencer: val.sequencer,
            framer: val.framer.map_codec(|_| codecs::Hello),
        }
    }
}

pub struct Login<Io: AsyncIo> {
    sequencer: LoginSequencer,
    framer: Framed<Io, codecs::Login>,
}

impl<Io: AsyncIo> Login<Io> {
    pub async fn send<'a, P>(&mut self, pkt: &'a P) -> Result<()>
    where
        P: codecs::LoginPacketSend + ::serde::ser::Serialize,
        Packet<&'a P>: From<&'a P>,
    {
        self.framer.send(pkt).await
    }
}

impl<Io: AsyncIo> From<Hello<Io>> for Login<Io> {
    fn from(val: Hello<Io>) -> Self {
        Self {
            sequencer: val.sequencer,
            framer: val.framer.map_codec(|_| codecs::Login),
        }
    }
}

pub struct ServerSelect<Io: AsyncIo> {
    sequencer: LoginSequencer,
    framer: Framed<Io, codecs::ServerSelect>,
}

impl<Io: AsyncIo> ServerSelect<Io> {
    pub async fn recv(&mut self) -> Result<Option<codecs::ServerSelectFrame>> {
        self.framer.try_next().await
    }
}

impl<Io: AsyncIo> From<Login<Io>> for ServerSelect<Io> {
    fn from(val: Login<Io>) -> Self {
        Self {
            sequencer: val.sequencer,
            framer: val.framer.map_codec(|_| codecs::ServerSelect),
        }
    }
}

pub struct Handoff<Io: AsyncIo> {
    #[allow(dead_code)] // This is a terminal state
    sequencer: LoginSequencer,
    framer: Framed<Io, codecs::Handoff>,
}

impl<Io: AsyncIo> Handoff<Io> {
    pub async fn send<'a, P>(&mut self, pkt: &'a P) -> Result<()>
    where
        P: codecs::HandoffPacketSend + ::serde::ser::Serialize,
        Packet<&'a P>: From<&'a P>,
    {
        self.framer.send(pkt).await
    }
}

impl<Io: AsyncIo> From<ServerSelect<Io>> for Handoff<Io> {
    fn from(val: ServerSelect<Io>) -> Self {
        Self {
            sequencer: val.sequencer,
            framer: val.framer.map_codec(|_| codecs::Handoff),
        }
    }
}

pub mod codecs {
    use crate::macros::define_codec;
    use ultimaonline_net::packets::login;

    define_codec! {
        pub Connected,
        send [],
        recv [
            login::ClientHello,
        ]
    }

    define_codec! {
        pub Hello,
        send [],
        recv [
            login::AccountLogin,
        ]
    }

    define_codec! {
        pub Login,
        send [
            login::LoginRejection,
            login::ServerList,
        ],
        recv []
    }

    define_codec! {
        pub ServerSelect,
        send [],
        recv [
            login::ServerSelection,
        ]
    }

    define_codec! {
        pub Handoff,
        send [
            login::GameServerHandoff,
        ],
        recv []
    }
}
