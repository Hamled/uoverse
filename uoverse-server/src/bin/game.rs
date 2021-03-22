use std::convert::TryInto;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use ultimaonline_net::error::{Error, Result};

#[tokio::main]
pub async fn main() {
    let listener = TcpListener::bind("127.0.0.1:2594").await.unwrap();

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
    use ultimaonline_net::packets::char_select as packets;
    use uoverse_server::game::client::*;

    // Client sends a 4 byte seed value, followed by the initial login packet.
    // The login packet itself includes the same seed value, so we can ignore
    // this one.
    let _ = socket.read_u32().await;

    let mut state = Connected::new(socket);
    let login = match state.recv().await? {
        Some(codecs::ConnectedFrame::GameLogin(login)) => login,
        _ => return Err(Error::Data),
    };

    let username = TryInto::<&str>::try_into(&login.username).expect("Invalid UTF-8 in username");
    let password = TryInto::<&str>::try_into(&login.password).expect("Invalid UTF-8 in password");
    println!(
        "Got account login. Username: {}, Password: {}, Seed: {}",
        username, password, login.seed
    );

    let mut state = CharList::<&mut TcpStream>::from(state);
    state
        .send(&packets::Features {
            // Enable all flags except:
            // Third Dawn =                             0100
            // 6th Char Slot =                     0010 0000
            // 8th Age =                      0001 0000 0000
            // 10th Age =                     0100 0000 0000
            // Increased Storage =            1000 0000 0000
            // Roleplay Faces =          0010 0000 0000 0000
            // Trial Account =           0100 0000 0000 0000
            flags: 0b0000_0000_1111_1111_1001_0010_1101_1011,
        })
        .await?;

    state
        .send(&packets::CharList {
            chars: vec![Default::default(); 7].into(),
            cities: vec![
                packets::CityInfo {
                    index: 0,
                    city: "Name Haven".into(),
                    building: "New Haven Bank".into(),
                    location: packets::MapLocation {
                        x: 3667,
                        y: 2625,
                        z: 0,
                        id: 1,
                    },
                    description: 1150168,
                    unknown_15: 0,
                },
                packets::CityInfo {
                    index: 1,
                    city: "Yew".into(),
                    building: "The Empath Abbey".into(),
                    location: packets::MapLocation {
                        x: 633,
                        y: 858,
                        z: 0,
                        id: 1,
                    },
                    description: 1075072,
                    unknown_15: 0,
                },
                packets::CityInfo {
                    index: 2,
                    city: "Minoc".into(),
                    building: "The Barnacle".into(),
                    location: packets::MapLocation {
                        x: 2476,
                        y: 413,
                        z: 15,
                        id: 1,
                    },
                    description: 1075073,
                    unknown_15: 0,
                },
                packets::CityInfo {
                    index: 3,
                    city: "Britain".into(),
                    building: "The Wayfarer's Inn".into(),
                    location: packets::MapLocation {
                        x: 1602,
                        y: 1591,
                        z: 20,
                        id: 1,
                    },
                    description: 1075074,
                    unknown_15: 0,
                },
                packets::CityInfo {
                    index: 4,
                    city: "Moonglow".into(),
                    building: "The Scholar's Inn".into(),
                    location: packets::MapLocation {
                        x: 4408,
                        y: 1168,
                        z: 0,
                        id: 1,
                    },
                    description: 1075075,
                    unknown_15: 0,
                },
                packets::CityInfo {
                    index: 5,
                    city: "Trinsic".into(),
                    building: "The Traveler's Inn".into(),
                    location: packets::MapLocation {
                        x: 1845,
                        y: 2745,
                        z: 0,
                        id: 1,
                    },
                    description: 1075076,
                    unknown_15: 0,
                },
                packets::CityInfo {
                    index: 6,
                    city: "Jhelom".into(),
                    building: "The Mercenary Inn".into(),
                    location: packets::MapLocation {
                        x: 1374,
                        y: 3826,
                        z: 0,
                        id: 1,
                    },
                    description: 1075078,
                    unknown_15: 0,
                },
                packets::CityInfo {
                    index: 7,
                    city: "Skara Brae".into(),
                    building: "The Falconer's Inn".into(),
                    location: packets::MapLocation {
                        x: 618,
                        y: 2234,
                        z: 0,
                        id: 1,
                    },
                    description: 1075079,
                    unknown_15: 0,
                },
                packets::CityInfo {
                    index: 8,
                    city: "Vesper".into(),
                    building: "The Ironwood Inn".into(),
                    location: packets::MapLocation {
                        x: 2771,
                        y: 976,
                        z: 0,
                        id: 1,
                    },
                    description: 1075080,
                    unknown_15: 0,
                },
            ]
            .into(),

            // Disable all flags except:
            // Context Menus =                          1000
            // AOS Expansion =                     0010 0000
            // SE Expansion =                      1000 0000
            // ML Expansion =                 0001 0000 0000
            // Seventh Char Slot =       0001 0000 0000 0000
            flags: 0b0000_0000_0000_0000_0001_0001_1010_1000,
            unknown_var1: -1,
        })
        .await?;

    state
        .send(&packets::VersionReq { unknown_00: 0x0003 })
        .await?;

    let mut state = ClientVersion::<&mut TcpStream>::from(state);
    let version = match state.recv().await? {
        Some(codecs::ClientVersionFrame::VersionResp(packets::VersionResp { version })) => version,
        _ => return Err(Error::Data),
    };

    println!("Got client version: {}", version);

    let mut state = CharSelect::<&mut TcpStream>::from(state);
    let create_info = match state.recv().await? {
        Some(codecs::CharSelectFrame::CreateCharacter(info)) => info,
        _ => return Err(Error::Data),
    };

    println!(
        "Create character named: {}",
        TryInto::<&str>::try_into(&create_info.name).unwrap()
    );

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    println!("Sending character into world");

    let mut state = CharLogin::<&mut TcpStream>::from(state);
    {
        use ultimaonline_net::packets::char_login as packets;
        state
            .send(&packets::LoginConfirmation {
                serial: 3833,
                unknown_04: 0,
                body_id: 401, // Human male?
                x: 3667,
                y: 2625,
                z: 0,
                direction: packets::Direction::South,
                unknown_10: 0,
                unknown_11: 0xFFFFFFFF,
                unknown_15: [0u8; 14],
            })
            .await?;

        // TODO: Send lots of other stuff here

        state.send(&packets::LoginComplete {}).await?;
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    Ok(())
}
