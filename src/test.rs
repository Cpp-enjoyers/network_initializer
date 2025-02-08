use std::{fs, iter::repeat_with, thread::{self, sleep}, time::Duration, vec};

use ap2024_unitn_cppenjoyers_webservers::{servers::{Media, Text}, GenericServer};
use common::{slc_commands::{ServerCommand, ServerEvent, TextMediaResponse, WebClientCommand, WebClientEvent}, Client, Server};
use crossbeam_channel::{Receiver, Sender};
use rand::{thread_rng, Rng};
use web_client::web_client::WebBrowser;
use wg_2024::{config::{Client as ClientConfig, Config, Drone as DroneConfig, Server as ServerConfig}, controller::{DroneCommand, DroneEvent}, drone::Drone as DroneTrait, packet::Packet};

use crate::{
    check_bidirectional_and_connected, check_client_connections, check_connected_only_drones,
    check_drone_connections, check_id_repetitions, check_pdr, check_server_connections, check_topology_constraints,
};

use dr_ones::Drone as DrDrone;
use drone_bettercalldrone::BetterCallDrone;
use getdroned::GetDroned;
use ap2024_rustinpeace_nosounddrone::NoSoundDroneRIP;
use rolling_drone::RollingDrone;
use rust_do_it::RustDoIt;
use rust_roveri::RustRoveri;
use rustafarian_drone::RustafarianDrone;
use rusteze_drone::RustezeDrone;
use rusty_drones::RustyDrone;

fn correct_config() -> Config {
    Config {
        drone: vec![
            DroneConfig {
                id: 0,
                connected_node_ids: vec![1, 3, 11],
                pdr: 0.,
            },
            DroneConfig {
                id: 1,
                connected_node_ids: vec![0, 2],
                pdr: 0.8,
            },
            DroneConfig {
                id: 2,
                connected_node_ids: vec![1, 3, 12],
                pdr: 1.,
            },
            DroneConfig {
                id: 3,
                connected_node_ids: vec![0, 2, 12],
                pdr: 0.4,
            },
        ],

        client: vec![ClientConfig {
            id: 11,
            connected_drone_ids: vec![0],
        }],

        server: vec![ServerConfig {
            id: 12,
            connected_drone_ids: vec![2, 3],
        }],
    }
}

fn instanciate_testing_topology<T: DroneTrait + 'static>() -> (
    Vec<Sender<DroneCommand>>,
    Receiver<DroneEvent>,
    Sender<ServerCommand>,
    Receiver<ServerEvent>,
    Sender<ServerCommand>,
    Receiver<ServerEvent>,
    Sender<WebClientCommand>,
    Receiver<WebClientEvent>,
) {
    // env::set_var("RUST_LOG", "info");
    // let _ = env_logger::try_init();

    let (st_events, st_eventr) = crossbeam_channel::unbounded();
    let (st_ctrls, st_ctrlr) = crossbeam_channel::unbounded();
    let (sm_events, sm_eventr) = crossbeam_channel::unbounded();
    let (sm_ctrls, sm_ctrlr) = crossbeam_channel::unbounded();
    let (c_events, c_eventr) = crossbeam_channel::unbounded();
    let (c_ctrls, c_ctrlr) = crossbeam_channel::unbounded();
    let (d_events, d_eventr) = crossbeam_channel::unbounded();
    let (tservers, tserverr) = crossbeam_channel::unbounded();
    let (smervers, mserverr) = crossbeam_channel::unbounded();
    let (clients, clientr) = crossbeam_channel::unbounded();
    let drone_command: Vec<(Sender<DroneCommand>, Receiver<DroneCommand>)> =
        repeat_with(|| crossbeam_channel::unbounded())
            .take(10)
            .collect();
    let drone_channels: Vec<(Sender<Packet>, Receiver<Packet>)> =
        repeat_with(|| crossbeam_channel::unbounded())
            .take(10)
            .collect();
    let drone_nbrs: [Vec<u8>; 10] = [
        vec![1, 5],
        vec![0, 2, 6],
        vec![1, 3, 7],
        vec![2, 4, 8],
        vec![3, 9],
        vec![0, 6],
        vec![5, 7, 1],
        vec![6, 8, 2],
        vec![7, 9, 3],
        vec![8, 4],
    ];
    for i in 0u8..10u8 {
        let mut map: std::collections::HashMap<u8, Sender<Packet>> = drone_nbrs[i as usize]
            .iter()
            .map(|&id| (id, drone_channels[id as usize].0.clone()))
            .collect();
        if i == 0 || i == 5 {
            map.insert(11, tservers.clone());
            map.insert(13, smervers.clone());
        } else if i == 4 || i == 9 {
            map.insert(12, clients.clone());
        }
        let mut drone = T::new(
            i,
            d_events.clone(),
            drone_command[i as usize].1.clone(),
            drone_channels[i as usize].1.clone(),
            map,
            // if too high, the test might fail for no reason besides me being unlucky
            Rng::gen_range(&mut thread_rng(), 0., 0.5),
        );
        thread::spawn(move || drone.run());
    }
    let mut server_t: GenericServer<Text> = Server::new(
        11,
        st_events.clone(),
        st_ctrlr.clone(),
        tserverr.clone(),
        [
            (0u8, drone_channels[0].0.clone()),
            (5u8, drone_channels[5].0.clone()),
        ]
        .into_iter()
        .collect(),
    );
    let mut server_m: GenericServer<Media> = Server::new(
        13,
        sm_events.clone(),
        sm_ctrlr.clone(),
        mserverr.clone(),
        [
            (0u8, drone_channels[0].0.clone()),
            (5u8, drone_channels[5].0.clone()),
        ]
        .into_iter()
        .collect(),
    );
    let mut client: WebBrowser = Client::new(
        12,
        c_events.clone(),
        c_ctrlr.clone(),
        clientr.clone(),
        [
            (4u8, drone_channels[4].0.clone()),
            (9u8, drone_channels[9].0.clone()),
        ]
        .into_iter()
        .collect(),
    );
    thread::spawn(move || server_t.run());
    thread::spawn(move || server_m.run());
    thread::spawn(move || client.run());

    (
        drone_command.into_iter().map(|(s, _)| s).collect(),
        d_eventr,
        st_ctrls,
        st_eventr,
        sm_ctrls,
        sm_eventr,
        c_ctrls,
        c_eventr,
    )
}

fn generic_full_file_request(
    devents: Receiver<DroneEvent>,
    stctrl: Sender<ServerCommand>,
    smctrl: Sender<ServerCommand>,
    cevents: Receiver<WebClientEvent>,
    cctrl: Sender<WebClientCommand>,
    file: String,
    check_file: impl Fn(TextMediaResponse) -> (),
) {
    sleep(Duration::from_secs(1));
    let _ = cctrl.send(WebClientCommand::AskServersTypes);
    let mut _flag: bool = false;
    loop {
        if let Ok(e) = devents.try_recv() {
            match e {
                DroneEvent::ControllerShortcut(p) => {
                    let &idx = p.routing_header.hops.last().unwrap();
                    if idx == 11 {
                        let _ = stctrl.send(ServerCommand::Shortcut(p));
                    } else if idx == 12 {
                        let _ = cctrl.send(WebClientCommand::Shortcut(p));
                    } else if idx == 13 {
                        let _ = smctrl.send(ServerCommand::Shortcut(p));
                    }
                }
                _ => {}
            }
        }
        if let Ok(e) = cevents.try_recv() {
            match e {
                WebClientEvent::FileFromClient(r, _) => {
                    check_file(r);
                    _flag = true;
                    break;
                }
                WebClientEvent::ServersTypes(_) => {
                    let _ = cctrl.send(WebClientCommand::AskListOfFiles(11));
                }
                WebClientEvent::ListOfFiles(_, _) => {
                    let _ = cctrl.send(WebClientCommand::RequestFile(file.clone(), 11));
                }
                WebClientEvent::UnsupportedRequest => {
                    panic!();
                }
                _ => {}
            }
        }
    }
    assert!(_flag);
}

#[test]
fn test_check_id_repetitions() {
    let Config {
        drone,
        client,
        server,
    } = correct_config();

    let mut drones_id: Vec<u8> = drone.iter().map(|drone| drone.id).collect();
    let clients_id: Vec<u8> = client.iter().map(|client| client.id).collect();
    let servers_id: Vec<u8> = server.iter().map(|server| server.id).collect();
    assert!(check_id_repetitions(&drones_id, &clients_id, &servers_id));

    drones_id[0] = 11;
    assert!(!check_id_repetitions(&drones_id, &clients_id, &servers_id));

    drones_id[0] = 12;
    assert!(!check_id_repetitions(&drones_id, &clients_id, &servers_id));

    drones_id[0] = 1;
    assert!(!check_id_repetitions(&drones_id, &clients_id, &servers_id));
}

#[test]
fn test_check_pdr() {
    let Config { mut drone, .. } = correct_config();

    assert!(check_pdr(&drone));

    drone[0].pdr = 3.;
    assert!(!check_pdr(&drone));

    drone[0].pdr = 0.98;
    drone[2].pdr = f32::NAN;
    assert!(!check_pdr(&drone));
}

#[test]
fn test_check_drone_connections() {
    let Config { mut drone, .. } = correct_config();

    assert!(check_drone_connections(&drone));

    drone[0].connected_node_ids[1] = drone[0].id;
    assert!(!check_drone_connections(&drone));

    drone[0].connected_node_ids[1] = drone[0].connected_node_ids[0];
    assert!(!check_drone_connections(&drone));
}

#[test]
fn test_check_client_connections() {
    let Config {
        drone, mut client, ..
    } = correct_config();

    let drones_id: Vec<u8> = drone.iter().map(|drone| drone.id).collect();

    assert!(check_client_connections(&client, &drones_id));

    client[0].connected_drone_ids[0] = client[0].id;
    assert!(!check_client_connections(&client, &drones_id));

    client[0].connected_drone_ids.push(0);
    assert!(!check_client_connections(&client, &drones_id));

    client[0].connected_drone_ids[0] = 123;
    assert!(!check_client_connections(&client, &drones_id));

    client[0].connected_drone_ids[0] = 0;
    client[0].connected_drone_ids[1] = 1;
    client[0].connected_drone_ids.push(2);
    client[0].connected_drone_ids.push(3);
    assert!(!check_client_connections(&client, &drones_id));
}

#[test]
fn test_check_server_connections() {
    let Config {
        drone,
        client,
        mut server,
    } = correct_config();

    let drones_id: Vec<u8> = drone.iter().map(|drone| drone.id).collect();

    assert!(check_server_connections(&server, &drones_id));

    server[0].connected_drone_ids[0] = client[0].id;
    assert!(!check_server_connections(&server, &drones_id));

    server[0].connected_drone_ids[1] = server[0].connected_drone_ids[0];
    assert!(!check_server_connections(&server, &drones_id));

    server[0].connected_drone_ids[0] = 123;
    assert!(!check_server_connections(&server, &drones_id));

    while server[0].connected_drone_ids.len() > 0 {
        server[0].connected_drone_ids.pop();
    }
    assert!(!check_server_connections(&server, &drones_id));
}

#[test]
fn test_bidirectional_and_connected() {
    let Config {
        drone,
        mut client,
        server,
    } = correct_config();

    assert!(check_bidirectional_and_connected(&drone, &client, &server));

    client.push(ClientConfig {
        id: 34,
        connected_drone_ids: vec![],
    });
    assert!(!check_bidirectional_and_connected(&drone, &client, &server));

    client.pop();
    client[0].connected_drone_ids.pop();
    assert!(!check_bidirectional_and_connected(&drone, &client, &server));
}

#[test]
fn test_connected_only_drones() {
    let Config { mut drone, .. } = correct_config();

    let drones_id: Vec<u8> = drone.iter().map(|drone| drone.id).collect();

    assert!(check_connected_only_drones(&drone, &drones_id));

    drone[0].connected_node_ids[1] = 10;
    assert!(!check_connected_only_drones(&drone, &drones_id));

    drone[0].connected_node_ids.pop();
    assert!(!check_connected_only_drones(&drone, &drones_id));
}

#[test]
fn double_chain(){
    let config_data: String =
        fs::read_to_string("config/double_chain.toml").expect("Unable to read config file");
    // having our structs implement the Deserialize trait allows us to use the toml::from_str function to deserialize the config file into each of them
    let Config {
        drone,
        client,
        server,
    }: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    assert!(check_topology_constraints(&drone, &client, &server));

}

#[test]
fn tree(){
    let config_data: String =
        fs::read_to_string("config/tree.toml").expect("Unable to read config file");
    // having our structs implement the Deserialize trait allows us to use the toml::from_str function to deserialize the config file into each of them
    let Config {
        drone,
        client,
        server,
    }: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    assert!(check_topology_constraints(&drone, &client, &server));

}

#[test]
fn subnet2(){
    let config_data: String =
        fs::read_to_string("config/sub_net_2.toml").expect("Unable to read config file");
    // having our structs implement the Deserialize trait allows us to use the toml::from_str function to deserialize the config file into each of them
    let Config {
        drone,
        client,
        server,
    }: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    assert!(check_topology_constraints(&drone, &client, &server));
}
fn star(){
    let config_data: String =
        fs::read_to_string("config/star.toml").expect("Unable to read config file");
    // having our structs implement the Deserialize trait allows us to use the toml::from_str function to deserialize the config file into each of them
    let Config {
        drone,
        client,
        server,
    }: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    assert!(check_topology_constraints(&drone, &client, &server));

}

#[test]
fn butterfly(){
    let config_data: String =
        fs::read_to_string("config/butterfly.toml").expect("Unable to read config file");
    // having our structs implement the Deserialize trait allows us to use the toml::from_str function to deserialize the config file into each of them
    let Config {
        drone,
        client,
        server,
    }: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    assert!(check_topology_constraints(&drone, &client, &server));

}

#[test]
fn test_drdrone(){
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
            instanciate_testing_topology::<DrDrone>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_bettercalldrone(){
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
            instanciate_testing_topology::<BetterCallDrone>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_getdroned(){
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
            instanciate_testing_topology::<GetDroned>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_rustinpeace(){
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
            instanciate_testing_topology::<NoSoundDroneRIP>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_rollingdrone(){
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
            instanciate_testing_topology::<RollingDrone>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_rustdoit(){
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
            instanciate_testing_topology::<RustDoIt>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_rustroveri(){
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
            instanciate_testing_topology::<RustRoveri>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_rustafarian(){
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
            instanciate_testing_topology::<RustafarianDrone>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_rusteze(){
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
            instanciate_testing_topology::<RustezeDrone>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}

#[test]
fn test_rustydrone(){
    let (_dcmds, devents, stctrl, _stevents, smctrl, _smevents, cctrl, cevents) =
            instanciate_testing_topology::<RustyDrone>();
    generic_full_file_request(
        devents,
        stctrl,
        smctrl,
        cevents,
        cctrl,
        "./public/file.html".to_owned(),
        |r: TextMediaResponse| {
            assert!(r.get_media_files().is_empty());
            assert!(r.get_html_file().1 == fs::read("./public/file.html").unwrap());
        },
    );
}