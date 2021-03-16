use futures::sink::SinkExt;
use std::convert::TryInto;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, FramedWrite};
use ultimaonline_net::packets::login;

pub trait AsyncIo = AsyncRead + AsyncWrite + Sized + Unpin;

pub struct LoginFsm<Io: AsyncIo> {
    states: States<Io>,
}

impl<Io: AsyncIo> LoginFsm<Io> {
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
                Hello(state) => state.step().await,
                Login(state) => state.step().await,
                ServerSelect(state) => state.step().await,
                Handoff(state) => state.step().await,
                Disconnect(_) => unreachable!(),
            };
        }
    }
}

enum States<Io: AsyncIo> {
    Disconnect(FsmState<Io, Disconnect>),
    Connected(FsmState<Io, Connected>),
    Hello(FsmState<Io, Hello>),
    Login(FsmState<Io, Login>),
    ServerSelect(FsmState<Io, ServerSelect>),
    Handoff(FsmState<Io, Handoff>),
}

struct FsmState<Io: AsyncIo, State> {
    io: Io,
    state: State,
}

struct Disconnect;

struct Connected;
impl<Io: AsyncIo> FsmState<Io, Connected> {
    async fn step(mut self) -> States<Io> {
        let mut codec = (codecs::Connected {}).framed(&mut self.io);

        use codecs::ConnectedFrameRecv::*;
        match codec.next().await {
            Some(Ok(ClientHello(hello))) => {
                println!(
                    "Got client hello. Seed: {}, Version: {}",
                    hello.seed, hello.version
                );

                States::Hello(self.into())
            }
            _ => States::Disconnect(self.into()),
        }
    }
}
impl<Io: AsyncIo> From<FsmState<Io, Connected>> for FsmState<Io, Disconnect> {
    fn from(val: FsmState<Io, Connected>) -> FsmState<Io, Disconnect> {
        FsmState {
            io: val.io,
            state: Disconnect {},
        }
    }
}
impl<Io: AsyncIo> From<FsmState<Io, Connected>> for FsmState<Io, Hello> {
    fn from(val: FsmState<Io, Connected>) -> FsmState<Io, Hello> {
        FsmState {
            io: val.io,
            state: Hello {},
        }
    }
}

struct Hello;
impl<Io: AsyncIo> FsmState<Io, Hello> {
    async fn step(mut self) -> States<Io> {
        let mut codec = (codecs::Hello {}).framed(&mut self.io);

        use codecs::HelloFrameRecv::*;
        match codec.next().await {
            Some(Ok(AccountLogin(login))) => {
                let username = TryInto::<&str>::try_into(&login.username)
                    .expect("Invalid UTF-8 in username")
                    .to_string();
                let password = TryInto::<&str>::try_into(&login.password)
                    .expect("Invalid UTF-8 in password")
                    .to_string();
                println!(
                    "Got account login. Username: {}, Password: {}",
                    username, password
                );

                // TODO: Should this be done with a From impl?
                States::Login(FsmState {
                    io: self.io,
                    state: Login { username, password },
                })
            }
            _ => States::Disconnect(self.into()),
        }
    }
}
impl<Io: AsyncIo> From<FsmState<Io, Hello>> for FsmState<Io, Disconnect> {
    fn from(val: FsmState<Io, Hello>) -> FsmState<Io, Disconnect> {
        FsmState {
            io: val.io,
            state: Disconnect {},
        }
    }
}

struct Login {
    username: String,
    password: String,
}
impl<Io: AsyncIo> FsmState<Io, Login> {
    async fn step(mut self) -> States<Io> {
        let mut codec = FramedWrite::new(&mut self.io, codecs::Login {});
        use codecs::LoginFrameSend::*;

        // TODO: Authenticate user and authorize for logging in
        /*
        codec
            .send(LoginRejection(login::LoginRejection {
                reason: login::LoginRejectionReason::Invalid,
            }))
            .await;
        */

        codec
            .send(ServerList(login::ServerList {
                flags: 0x5D,
                list: vec![login::ServerInfo {
                    index: 0,
                    name: "Test Server".into(),
                    fullness: 0,
                    timezone: 0,
                    ip_address: "127.0.0.1".parse().unwrap(),
                }],
            }))
            .await;

        States::ServerSelect(self.into())
    }
}
impl<Io: AsyncIo> From<FsmState<Io, Login>> for FsmState<Io, Disconnect> {
    fn from(val: FsmState<Io, Login>) -> FsmState<Io, Disconnect> {
        FsmState {
            io: val.io,
            state: Disconnect {},
        }
    }
}
impl<Io: AsyncIo> From<FsmState<Io, Login>> for FsmState<Io, ServerSelect> {
    fn from(val: FsmState<Io, Login>) -> FsmState<Io, ServerSelect> {
        FsmState {
            io: val.io,
            state: ServerSelect {},
        }
    }
}

struct ServerSelect;
impl<Io: AsyncIo> FsmState<Io, ServerSelect> {
    async fn step(mut self) -> States<Io> {
        let mut codec = (codecs::ServerSelect {}).framed(&mut self.io);

        use codecs::ServerSelectFrameRecv::*;
        match codec.next().await {
            Some(Ok(ServerSelection(login::ServerSelection { index }))) => {
                println!("Got server selection: {}", index);
                States::Handoff(self.into())
            }
            _ => States::Disconnect(self.into()),
        }
    }
}
impl<Io: AsyncIo> From<FsmState<Io, ServerSelect>> for FsmState<Io, Disconnect> {
    fn from(val: FsmState<Io, ServerSelect>) -> FsmState<Io, Disconnect> {
        FsmState {
            io: val.io,
            state: Disconnect {},
        }
    }
}
impl<Io: AsyncIo> From<FsmState<Io, ServerSelect>> for FsmState<Io, Handoff> {
    fn from(val: FsmState<Io, ServerSelect>) -> FsmState<Io, Handoff> {
        FsmState {
            io: val.io,
            state: Handoff {},
        }
    }
}

struct Handoff;
impl<Io: AsyncIo> FsmState<Io, Handoff> {
    async fn step(mut self) -> States<Io> {
        let mut codec = FramedWrite::new(&mut self.io, codecs::Handoff {});
        use codecs::HandoffFrameSend::*;

        codec
            .send(GameServerHandoff(login::GameServerHandoff {
                socket: "127.0.0.1:2594".parse().unwrap(),
                ticket: rand::random::<u32>(),
            }))
            .await;

        States::Disconnect(self.into())
    }
}
impl<Io: AsyncIo> From<FsmState<Io, Handoff>> for FsmState<Io, Disconnect> {
    fn from(val: FsmState<Io, Handoff>) -> FsmState<Io, Disconnect> {
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
