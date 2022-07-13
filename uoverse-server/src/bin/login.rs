use std::convert::TryInto;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use ultimaonline_net::error::{Error, Result};

#[tokio::main]
pub async fn main() {
    let listener = TcpListener::bind("127.0.0.1:2593").await.unwrap();

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            if process(&mut socket).await.is_err() {
                println!("Client had error");
            }

            println!("Client disconnected.");
            socket.shutdown().await.unwrap();
        });
    }
}

async fn process(socket: &mut TcpStream) -> Result<()> {
    use ultimaonline_net::packets::login as packets;
    use uoverse_server::login::client::*;

    let mut state = Connected::new(socket);
    let hello = match state.recv().await? {
        Some(codecs::ConnectedFrame::ClientHello(hello)) => hello,
        _ => return Err(Error::Data),
    };

    println!(
        "Got client hello. Seed: {}, Version: {}",
        hello.seed, hello.version
    );

    let mut state = Hello::<&mut TcpStream>::from(state);
    let login = match state.recv().await? {
        Some(codecs::HelloFrame::AccountLogin(login)) => login,
        _ => return Err(Error::Data),
    };

    let username = TryInto::<&str>::try_into(&login.username).expect("Invalid UTF-8 in username");
    let password = TryInto::<&str>::try_into(&login.password).expect("Invalid UTF-8 in password");
    println!(
        "Got account login. Username: {}, Password: {}",
        username, password
    );

    let mut state = Login::<&mut TcpStream>::from(state);
    // TODO: Actually authenticate user and authorize for logging in
    // Check the password
    if &password[..4] != "test" {
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
                ip_address: "127.0.0.1".parse().unwrap(),
            }]
            .into(),
        })
        .await?;

    let mut state = ServerSelect::<&mut TcpStream>::from(state);

    // Get the server that they've selected
    let selection = match state.recv().await? {
        Some(codecs::ServerSelectFrame::ServerSelection(packets::ServerSelection { index })) => {
            index
        }
        _ => return Err(Error::Data),
    };

    println!("Got server selection: {}", selection);

    let mut state = Handoff::<&mut TcpStream>::from(state);

    // Send the information to hand-off to the game server
    state
        .send(&packets::GameServerHandoff {
            socket: "127.0.0.1:2594".parse().unwrap(),
            ticket: rand::random::<u32>(),
        })
        .await?;

    Ok(())
}
