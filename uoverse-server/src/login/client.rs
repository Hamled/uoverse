use futures::sink::SinkExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};
use ultimaonline_net::{error::Result, packets::ToPacket};

pub trait AsyncIo = AsyncRead + AsyncWrite + Unpin + Send + Sync;

// Ensures that the FSM must start with the initial state (Connected)
struct LoginSequencer;

pub struct Connected<Io: AsyncIo> {
    io: Io,
    sequencer: LoginSequencer,
}

impl<Io: AsyncIo> Connected<Io> {
    pub async fn recv(&mut self) -> Result<Option<codecs::ConnectedFrame>> {
        let mut reader = FramedRead::new(&mut self.io, codecs::Connected {});
        reader.try_next().await
    }

    pub fn new(io: Io) -> Self {
        Self {
            io,
            sequencer: LoginSequencer {},
        }
    }
}

pub struct Hello<Io: AsyncIo> {
    io: Io,
    sequencer: LoginSequencer,
}

impl<Io: AsyncIo> Hello<Io> {
    pub async fn recv(&mut self) -> Result<Option<codecs::HelloFrame>> {
        let mut reader = FramedRead::new(&mut self.io, codecs::Hello {});
        reader.try_next().await
    }
}

impl<Io: AsyncIo> From<Connected<Io>> for Hello<Io> {
    fn from(val: Connected<Io>) -> Self {
        Self {
            io: val.io,
            sequencer: val.sequencer,
        }
    }
}

pub struct Login<Io: AsyncIo> {
    io: Io,
    sequencer: LoginSequencer,
}

impl<Io: AsyncIo> Login<Io> {
    pub async fn send<'a, P>(&mut self, pkt: &'a P) -> Result<()>
    where
        P: codecs::LoginEncode + ToPacket<'a> + ::serde::ser::Serialize,
    {
        let mut writer = FramedWrite::new(&mut self.io, codecs::Login {});
        writer.send(pkt).await
    }
}

impl<Io: AsyncIo> From<Hello<Io>> for Login<Io> {
    fn from(val: Hello<Io>) -> Self {
        Self {
            io: val.io,
            sequencer: val.sequencer,
        }
    }
}

pub struct ServerSelect<Io: AsyncIo> {
    io: Io,
    sequencer: LoginSequencer,
}

impl<Io: AsyncIo> ServerSelect<Io> {
    pub async fn recv(&mut self) -> Result<Option<codecs::ServerSelectFrame>> {
        let mut reader = FramedRead::new(&mut self.io, codecs::ServerSelect {});
        reader.try_next().await
    }
}

impl<Io: AsyncIo> From<Login<Io>> for ServerSelect<Io> {
    fn from(val: Login<Io>) -> Self {
        Self {
            io: val.io,
            sequencer: val.sequencer,
        }
    }
}

pub struct Handoff<Io: AsyncIo> {
    io: Io,
    #[allow(dead_code)] // This is a terminal state
    sequencer: LoginSequencer,
}

impl<Io: AsyncIo> Handoff<Io> {
    pub async fn send<'a, P>(&mut self, pkt: &'a P) -> Result<()>
    where
        P: codecs::HandoffEncode + ToPacket<'a> + ::serde::ser::Serialize,
    {
        let mut writer = FramedWrite::new(&mut self.io, codecs::Handoff {});
        writer.send(pkt).await
    }
}

impl<Io: AsyncIo> From<ServerSelect<Io>> for Handoff<Io> {
    fn from(val: ServerSelect<Io>) -> Self {
        Self {
            io: val.io,
            sequencer: val.sequencer,
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
