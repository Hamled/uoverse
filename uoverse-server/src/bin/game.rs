use std::convert::TryInto;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use ultimaonline_net::{
    error::{Error, Result},
    types::Serial,
};
use uoverse_server::game::client::{self, *};

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
    let state = handshake(socket).await?;
    let state = char_login(state).await?;
    in_world(state).await?;

    Ok(())
}

const PLAYER_SERIAL: Serial = 3833;

async fn handshake<Io>(mut socket: Io) -> Result<CharSelect<Io>>
where
    Io: AsyncIo,
{
    use ultimaonline_net::packets::char_select as packets;

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

    let mut state = CharList::<Io>::from(state);
    state
        .send(&packets::Features {
            flags: client::FEATURES,
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
            flags: client::FLAGS,
            unknown_var1: -1,
        })
        .await?;

    state
        .send(&packets::VersionReq { unknown_00: 0x0003 })
        .await?;

    let mut state = ClientVersion::<Io>::from(state);
    let version = match state.recv().await? {
        Some(codecs::ClientVersionFrame::VersionResp(packets::VersionResp { version })) => version,
        _ => return Err(Error::Data),
    };

    println!("Got client version: {}", version);

    Ok(CharSelect::<Io>::from(state))
}

async fn char_login<Io>(mut state: CharSelect<Io>) -> Result<InWorld<Io>>
where
    Io: AsyncIo,
{
    use ultimaonline_net::{packets::*, types};
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

    let mut state = CharLogin::<Io>::from(state);

    state
        .send(&char_login::LoginConfirmation {
            serial: PLAYER_SERIAL,
            unknown_04: 0,
            body: 401, // Human male?
            x: 3667,
            y: 2625,
            z: 0,
            direction: types::Direction::South,
            unknown_10: 0,
            unknown_11: 0xFFFFFFFF,
            unknown_15: [0u8; 14],
        })
        .await?;

    Ok(InWorld::<Io>::from(state))
}

async fn in_world<Io>(mut state: InWorld<Io>) -> Result<()>
where
    Io: AsyncIo,
{
    use ultimaonline_net::packets::*;

    state
        .send(&mobile::MobLightLevel {
            serial: PLAYER_SERIAL,
            level: 30,
        })
        .await?;

    state.send(&world::WorldLightLevel { level: 30 }).await?;

    // TODO: Send lots of other stuff here
    state.send(&char_login::LoginComplete {}).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    Ok(())
}
