use std::{
    collections::HashMap, iter::repeat_with, thread::{self, sleep}, time::Duration, vec
};

use ap2024_unitn_cppenjoyers_webservers::{
    servers::{Media, ServerType, Text},
    GenericServer, TextServer,
};
use common::{
    slc_commands::{
        self, ServerCommand, ServerEvent, TextMediaResponse, WebClientCommand, WebClientEvent
    },
    Client, Server,
};
use crossbeam_channel::{Receiver, Sender};
use rand::{thread_rng, Rng};
use web_client::web_client::WebBrowser;
use wg_2024::{
    config::{Client as ClientConfig, Config, Drone as DroneConfig, Server as ServerConfig},
    controller::{DroneCommand, DroneEvent},
    drone::Drone as DroneTrait,
    packet::Packet,
};

#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod drone_tests;
#[cfg(test)]
mod topology_tests;

/// get a coorectr configuration for the network
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

/// instanciates a double chain topology for testing
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

/// generic full request between WebBrowser and Text/Media Servers
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
                WebClientEvent::ServersTypes(list) => {
                    if list == HashMap::from([(11, slc_commands::ServerType::FileServer), (13, slc_commands::ServerType::MediaServer)]){
                        let _ = cctrl.send(WebClientCommand::AskListOfFiles(11));

                    }

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
