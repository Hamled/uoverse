use eyre::{eyre, Context, Result};
use std::{
    convert::TryInto,
    env,
    net::{Ipv4Addr, SocketAddrV4},
};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tracing::{debug_span, debug, info_span, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use uoverse_server::login::client::*;

const DEFAULT_LISTEN_ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const DEFAULT_LISTEN_PORT: u16 = 2593;

const DEFAULT_GAME_ADDR: Ipv4Addr = DEFAULT_LISTEN_ADDR;
const DEFAULT_GAME_PORT: u16 = DEFAULT_LISTEN_PORT + 1;

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut listen_addr = DEFAULT_LISTEN_ADDR;
    let mut listen_port = DEFAULT_LISTEN_PORT;

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        listen_addr = args[1]
            .parse()
            .expect(format!("Invalid listen address: {}", &args[1]).as_str());
    }
    if args.len() > 2 {
        listen_port = u16::from_str_radix(&args[2], 10)
            .expect(format!("Invalid listen port: {}", &args[2]).as_str());
    }

    let listen_socket = SocketAddrV4::new(listen_addr, listen_port);

    let mut game_addr = DEFAULT_GAME_ADDR;
    let mut game_port = DEFAULT_GAME_PORT;
    if args.len() > 3 {
        game_addr = args[3]
            .parse()
            .expect(format!("Invalid game server address: {}", &args[3]).as_str())
    }
    if args.len() > 4 {
        game_port = u16::from_str_radix(&args[4], 10)
            .expect(format!("Invalid game server port: {}", &args[4]).as_str());
    }
    let game_socket = SocketAddrV4::new(game_addr, game_port);

    tracing_subscriber::registry().with(fmt::layer()).with(EnvFilter::from_default_env()).init();

    let span = info_span!("server");
    let _ = span.enter();

    let listener = TcpListener::bind(listen_socket).await.unwrap();
    info!(socket = %listen_socket, "Login server listening on {}", listen_socket);
    info!(socket = %game_socket, "Using game server socket {}", game_socket);

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(&mut socket, game_socket)
                .await
                .wrap_err("Client had error during login")?;

            info!("Client disconnected.");
            socket.shutdown().await.unwrap();
            Ok::<(), eyre::Report>(())
        });
    }
}

async fn process<Io: AsyncIo>(socket: Io, game_socket: SocketAddrV4) -> Result<()> {
    use ultimaonline_net::packets::login as packets;

    let span = debug_span!("client_process");
    let _ = span.enter();

    let mut state = Connected::new(socket);
    let hello = match state.recv().await? {
        Some(codecs::ConnectedFrameRecv::ClientHello(hello)) => hello,
        _ => return Err(eyre!("Did not get ClientHello packet")),
    };

    debug!(
        seed = hello.seed, version = %hello.version,
        "Got client hello. Seed: {}, Version: {}",
        hello.seed, hello.version
    );

    let mut state = Hello::<Io>::from(state);
    let login = match state.recv().await? {
        Some(codecs::HelloFrameRecv::AccountLogin(login)) => login,
        _ => return Err(eyre!("Did not get AccountLogin packet")),
    };

    let username = TryInto::<&str>::try_into(&login.username).expect("Invalid UTF-8 in username");
    let password = TryInto::<&str>::try_into(&login.password).expect("Invalid UTF-8 in password");
    debug!(
        %username, %password,
        "Got account login. Username: {}, Password: {}",
        username, password
    );

    let mut state = Login::<Io>::from(state);
    // TODO: Actually authenticate user and authorize for logging in
    // Check the password
    if &password[..4] != "test" {
        debug!("Account password invalid, rejecting login request");
        // Reject login
        state
            .send(&packets::LoginRejection {
                reason: packets::LoginRejectionReason::BadPass,
            })
            .await?;
        return Ok(());
    }

    // Send server list
    state
        .send(&packets::ServerList {
            flags: 0x5D,
            list: vec![packets::ServerInfo {
                index: 0,
                name: "Test Server".into(),
                fullness: 0,
                timezone: 0,
                ip_address: *game_socket.ip(),
            }]
            .into(),
        })
        .await?;

    let mut state = ServerSelect::<Io>::from(state);

    // Get the server that they've selected
    let selection = match state.recv().await? {
        Some(codecs::ServerSelectFrameRecv::ServerSelection(packets::ServerSelection {
            index,
        })) => index,
        _ => return Err(eyre!("Did not get ServerSelection packet")),
    };

    debug!("Got server selection: {}", selection);

    let mut state = Handoff::<Io>::from(state);

    // Send the information to hand-off to the game server
    state
        .send(&packets::GameServerHandoff {
            socket: game_socket,
            ticket: rand::random::<u32>(),
        })
        .await?;

    Ok(())
}
