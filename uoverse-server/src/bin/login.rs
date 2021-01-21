use rand;
use std::convert::TryInto;
use std::net::{TcpListener, TcpStream};
use ultimaonline_net::packets::{login::*, FromPacketData, ToPacket};

fn handle_client(mut stream: TcpStream) {
    // Expect the ClientHello first
    let hello = ClientHello::from_packet_data(&mut stream).expect("Couldn't parse ClientHello");
    println!(
        "Got client hello. Seed: {}, Version: {}",
        hello.seed, hello.version
    );

    // Expect the account login
    let login = AccountLogin::from_packet_data(&mut stream).expect("Couldn't parse AccountLogin");
    let username = TryInto::<&str>::try_into(&login.username).expect("Invalid UTF-8 in username");
    let password = TryInto::<&str>::try_into(&login.password).expect("Invalid UTF-8 in password");
    println!(
        "Got account login. Username: {}, Password: {}",
        username, password
    );

    // Check the password
    if &password[..4] != "test" {
        // Reject login
        LoginRejection {
            reason: LoginRejectionReason::BadPass,
        }
        .to_packet()
        .to_writer(&mut stream)
        .expect("Couldn't write login rejection");
        return;
    }
    // Send server list
    ServerList {
        flags: 0x5D,
        list: vec![ServerInfo {
            index: 0,
            name: "Test Server".into(),
            fullness: 0,
            timezone: 0,
            ip_address: "127.0.0.1".parse().unwrap(),
        }],
    }
    .to_packet()
    .to_writer(&mut stream)
    .expect("Couldn't write server list");

    // Get the server that they've selected
    let selection = ServerSelection::from_packet_data(&mut stream);
    if let Err(err) = selection {
        println!("Unable to get server selection: {}", err);
        return;
    }

    let selection = selection.unwrap();
    println!("Got server selection: {}", selection.index);

    // Send the information to hand-off to the game server
    GameServerHandoff {
        socket: "127.0.0.1:2594".parse().unwrap(),
        ticket: rand::random::<u32>(),
    }
    .to_packet()
    .to_writer(&mut stream)
    .expect("Couldn't write game server handoff");
}

fn main() -> std::io::Result<()> {
    let serve_addr = "127.0.0.1:2593";

    let listener = TcpListener::bind(serve_addr)?;
    println!("Login server listening on {}", serve_addr);

    for stream in listener.incoming() {
        let stream = stream?;
        stream.set_nonblocking(false)?;
        handle_client(stream);
    }

    Ok(())
}
