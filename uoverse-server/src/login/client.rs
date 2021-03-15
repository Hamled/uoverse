use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::StreamExt;
use tokio_util::codec::Decoder;

pub trait AsyncIo = AsyncRead + AsyncWrite + Sized + Unpin;

pub struct LoginFsm<Io>
where
    Io: AsyncIo,
{
    states: States<Io>,
}

impl<Io> LoginFsm<Io>
where
    Io: AsyncIo,
{
    pub fn new(io: Io) -> Self {
        LoginFsm {
            states: States::Connected(FsmState {
                io,
                state: Connected {},
            }),
        }
    }

    pub async fn run(mut self) {
        use States::*;
        loop {
            if let Disconnect(_) = self.states {
                return;
            }

            self.states = match self.states {
                Connected(state) => state.step().await,
                Disconnect(_) => unreachable!(),
            };
        }
    }
}

enum States<Io>
where
    Io: AsyncIo,
{
    Disconnect(FsmState<Io, Disconnect>),
    Connected(FsmState<Io, Connected>),
}

struct FsmState<Io: AsyncIo, State> {
    io: Io,
    state: State,
}

struct Disconnect;

struct Connected;
impl<Io> FsmState<Io, Connected>
where
    Io: AsyncIo,
{
    async fn step(mut self) -> States<Io> {
        let mut codec = (codecs::Connected {}).framed(&mut self.io);

        if let Some(Ok(frame)) = codec.next().await {
            match frame {
                codecs::ConnectedFrameRecv::ClientHello(hello) => println!(
                    "Got client hello. Seed: {}, Version: {}",
                    hello.seed, hello.version
                ),
            }
        }

        States::Disconnect(self.into())
    }
}
impl<Io> From<FsmState<Io, Connected>> for FsmState<Io, Disconnect>
where
    Io: AsyncIo,
{
    fn from(val: FsmState<Io, Connected>) -> FsmState<Io, Disconnect> {
        FsmState {
            io: val.io,
            state: Disconnect {},
        }
    }
}

mod codecs {
    use crate::macros::define_codec;
    use ultimaonline_net::packets::login;

    define_codec! {
        pub Connected,
        send [],
        recv [
            login::ClientHello,
        ]
    }
}
