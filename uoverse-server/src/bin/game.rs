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
            match process(&mut socket).await {
                Err(Error::Data(err)) => println!("Client had error: {}", err),
                Err(Error::Io(err)) => println!("Client had error: {}", err),
                Err(Error::Message(err)) => println!("Client had error: {}", err),
                Ok(()) => {
                    println!("Client disconnected.");
                    socket.shutdown().await.unwrap();
                }
            }
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

async fn handshake<Io: AsyncIo>(mut socket: Io) -> Result<CharSelect<Io>> {
    use ultimaonline_net::packets::char_select as packets;

    // Client sends a 4 byte seed value, followed by the initial login packet.
    // The login packet itself includes the same seed value, so we can ignore
    // this one.
    let _ = socket.read_u32().await;

    let mut state = Connected::new(socket);
    let login = match state.recv().await? {
        Some(codecs::ConnectedFrame::GameLogin(login)) => login,
        _ => return Err(Error::data("Did not get GameLogin packet")),
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
        _ => return Err(Error::data("Did not get VersionResp packet")),
    };

    println!("Got client version: {}", version);

    Ok(CharSelect::<Io>::from(state))
}

async fn char_login<Io: AsyncIo>(mut state: CharSelect<Io>) -> Result<InWorld<Io>> {
    use ultimaonline_net::{packets::*, types};
    let create_info = match state.recv().await? {
        Some(codecs::CharSelectFrame::CreateCharacter(info)) => info,
        _ => return Err(Error::data("Did not get CreateCharacter packet")),
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

async fn in_world<Io: AsyncIo>(mut state: InWorld<Io>) -> Result<()> {
    use ultimaonline_net::{packets::*, types};

    state
        .send(&mobile::MobLightLevel {
            serial: PLAYER_SERIAL,
            level: 30,
        })
        .await?;

    state.send(&world::WorldLightLevel { level: 30 }).await?;

    // Character status
    state
        .send(&char_login::CharStatus {
            serial: PLAYER_SERIAL,
            name: "Hamled".into(),
            hitpoints: char_login::Attribute {
                current: 100,
                maximum: 100,
            },
            renamable: false,
            version: 6,    // Latest version for character status
            gender: false, // Male
            strength: 20,
            dexterity: 20,
            intelligence: 20,
            stamina: char_login::Attribute {
                current: 100,
                maximum: 100,
            },
            mana: char_login::Attribute {
                current: 100,
                maximum: 100,
            },
            gold: 0,
            phys_resist: 50,
            weight: char_login::Attribute {
                current: 0,
                maximum: 100,
            },
            race: types::Race::Human,
            stat_cap: 300,
            follower_count: 0,
            follower_max: 0,
            fire_resist: 50,
            cold_resist: 50,
            poison_resist: 50,
            energy_resist: 50,
            luck: 20,
            damage_min: 0,
            damage_max: 0,
            tithing_points: 0,
            aos_stats: [Default::default(); 15],
        })
        .await?;

    state
        .send(&mobile::Appearance {
            state: mobile::State {
                serial: 55858,
                body: 401,
                x: 3668,
                y: 2625,
                z: 0,
                direction: types::Direction::East,
                hue: 1003,
                flags: mobile::EntityFlags::None,
                notoriety: types::Notoriety::Ally,
            },
            items: vec![
                mobile::Item {
                    serial: 0x40000001,
                    type_id: 0x1EFD, // Fancy Shirt
                    layer: 0x05,     // Shirt
                    hue: 1837,
                },
                mobile::Item {
                    serial: 0x40000002,
                    type_id: 0x1539, // Long Pants
                    layer: 0x04,     // Pants
                    hue: 1897,
                },
                mobile::Item {
                    serial: 0x40000003,
                    type_id: 0x170B, // Boots
                    layer: 0x04,     // Shoes
                    hue: 1900,
                },
                mobile::Item {
                    serial: 0x40000004,
                    type_id: 0x1515, // Cloak
                    layer: 0x14,     // Cloak
                    hue: 1811,
                },
                mobile::Item {
                    serial: 0x40000005,
                    type_id: 0x203C, // Long hair
                    layer: 0x0B,     // Hair
                    hue: 1111,
                },
            ]
            .into(),
        })
        .await?;

    // TODO: Send lots of other stuff here
    state.send(&char_login::LoginComplete {}).await?;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
