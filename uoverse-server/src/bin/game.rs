use eyre::{eyre, Context, Result};
use std::{
    convert::TryInto,
    env,
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Notify,
};
use tokio::{net::TcpListener, task::JoinHandle};
use tracing::{debug, debug_span, error, info, info_span};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use ultimaonline_net::types::Serial;
use uoverse_server::game::client::{self, *};
use uoverse_server::game::server;

const DEFAULT_LISTEN_ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const DEFAULT_LISTEN_PORT: u16 = 2594;

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

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let span = info_span!("server");
    let _ = span.enter();

    let listener = TcpListener::bind(listen_socket).await.unwrap();
    info!(socket = %listen_socket, "Game server listening on {}", listen_socket);

    let server = Arc::new(server::Server::new());
    let shutdown_notice = Arc::new(Notify::new());
    {
        let server = server.clone();
        let shutdown_notice = shutdown_notice.clone();
        ctrlc::set_handler(move || {
            server.shutdown();
            shutdown_notice.notify_one();
        })
        .expect("Error setting Ctrl-C signal handler");
    }

    let server_task: JoinHandle<Result<()>> = {
        let server = server.clone();
        tokio::spawn(async move { Ok(server.run_loop().await?) })
    };

    loop {
        tokio::select! {
            Ok((mut socket, _)) = listener.accept() => {
                let server = server.clone();
                tokio::spawn(async move {
                    match process(&mut socket, server).await {
                        Err(err) => error!("{:#}", err),
                        Ok(()) => {}
                    }
                });
            }

            _ = shutdown_notice.notified() => {
                info!(socket = %listen_socket, "Stopped listening on {}", listen_socket);
                break;
            }
        }
    }

    server_task
        .await
        .expect("Error joining server task")
        .wrap_err("Server error")?;

    info!("Shutdown complete.");
    Ok(())
}

async fn process<Io: AsyncIo>(mut socket: Io, server: Arc<server::Server>) -> Result<()> {
    let span = debug_span!("client");
    let _ = span.enter();

    let preworld_span = debug_span!(parent: &span, "preworld");
    let span_guard = preworld_span.enter();
    let state = preworld(&mut socket)
        .await
        .wrap_err("Client did not complete pre-world")?;

    debug!("Client completed pre-world.");
    drop(span_guard);

    let inworld_span = debug_span!(parent: &span, "in-world");
    let span_guard = inworld_span.enter();
    in_world(server, state)
        .await
        .wrap_err("Client had error during in-world")?;
    drop(span_guard);

    debug!("Client disconnected.");
    socket.shutdown().await?;

    Ok(())
}

async fn preworld<Io: AsyncIo>(socket: Io) -> Result<InWorld<Io>> {
    let state = handshake(socket).await?;
    let state = char_login(state).await?;

    Ok(state)
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
        Some(codecs::ConnectedFrameRecv::GameLogin(login)) => login,
        _ => return Err(eyre!("Did not get GameLogin packet")),
    };

    let username = TryInto::<&str>::try_into(&login.username).expect("Invalid UTF-8 in username");
    let password = TryInto::<&str>::try_into(&login.password).expect("Invalid UTF-8 in password");
    debug!(
        %username, %password, seed = login.seed,
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
        Some(codecs::ClientVersionFrameRecv::VersionResp(packets::VersionResp { version })) => {
            version
        }
        _ => return Err(eyre!("Did not get VersionResp packet")),
    };

    debug!(version = %version, "Got client version: {}", version);

    Ok(CharSelect::<Io>::from(state))
}

async fn char_login<Io: AsyncIo>(mut state: CharSelect<Io>) -> Result<InWorld<Io>> {
    use ultimaonline_net::{packets::*, types};
    let create_info = match state.recv().await? {
        Some(codecs::CharSelectFrameRecv::CreateCharacter(info)) => info,
        _ => return Err(eyre!("Did not get CreateCharacter packet")),
    };

    let name: &str = (&create_info.name).try_into()?;
    debug!(
        char_name = %name,
        "Create character named: {}", name
    );

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    debug!(char_name = %name, "Sending character into world");

    let mut state = CharLogin::<Io>::from(state);

    // Set the map first
    state
        .send(&map::MapChange {
            map_id: 0x0, // Britannia
        })
        .await?;

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

    state.send(&char_login::LoginComplete {}).await?;

    Ok(InWorld::<Io>::from(state))
}

async fn in_world<Io: AsyncIo>(server: Arc<server::Server>, mut state: InWorld<Io>) -> Result<()> {
    use codecs::InWorldFrameRecv;
    use ultimaonline_net::packets::network::{PingAck, PingReq};

    let mut client = server.new_client()?;

    loop {
        tokio::select! {
            res = state.recv() => {
                match res? {
                    Some(InWorldFrameRecv::PingReq(PingReq {val})) => {
                        state.send(&PingAck{val}).await?
                    },
                    Some(packet) => client.send(packet)?,
                    None => {
                        debug!("Client connection closed.");
                        break;
                    },
                }
            },

            packet = client.receiver.recv() => {
                match packet {
                    Some(packet) => state.send_frame(&packet).await?,
                    None => {
                        // TODO: Send packets that inform the client of removal
                        debug!("Client removed from world.");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
