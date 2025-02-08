#![warn(clippy::pedantic)]

use ap2024_rustinpeace_nosounddrone::NoSoundDroneRIP;
use common::slc_commands::{
    ChatClientCommand, ChatClientEvent, ServerCommand, ServerEvent, WebClientCommand,
    WebClientEvent,
};
use common::{Client as ClientTrait, Server as ServerTrait};
use crossbeam_channel::{Receiver, Sender};
use dr_ones::Drone as DrDrone;
use drone_bettercalldrone::BetterCallDrone;
use getdroned::GetDroned;
use itertools::{chain, Itertools};
use rolling_drone::RollingDrone;
use rust_do_it::RustDoIt;
use rust_roveri::RustRoveri;
use rustafarian_drone::RustafarianDrone;
use rusteze_drone::RustezeDrone;
use rusty_drones::RustyDrone;
use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs;
use wg_2024::config::Config;
use wg_2024::config::{Client, Drone, Server};
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone as DroneTrait;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;
// use chat_common;

#[cfg(test)]
mod test;

type DroneChannels = HashMap<
    NodeId,
    (
        Sender<DroneCommand>,
        Receiver<DroneEvent>,
        Sender<Packet>,
        Receiver<Packet>,
    ),
>;
type WebClientChannels = HashMap<
    NodeId,
    (
        Sender<WebClientCommand>,
        Receiver<WebClientEvent>,
        Sender<Packet>,
        Receiver<Packet>,
    ),
>;
type ChatClientChannels = HashMap<
    NodeId,
    (
        Sender<ChatClientCommand>,
        Receiver<ChatClientEvent>,
        Sender<Packet>,
        Receiver<Packet>,
    ),
>;
type ServerChannels = HashMap<
    NodeId,
    (
        Sender<ServerCommand>,
        Receiver<ServerEvent>,
        Sender<Packet>,
        Receiver<Packet>,
    ),
>;

macro_rules! create_boxed {
    ($type:ty) => {
        |id: NodeId,
         controller_send: Sender<DroneEvent>,
         controller_recv: Receiver<DroneCommand>,
         packet_recv: Receiver<Packet>,
         packet_send: HashMap<NodeId, Sender<Packet>>,
         pdr: f32|
         -> Box<dyn DroneTrait> {
            Box::new(<$type>::new(
                id,
                controller_send,
                controller_recv,
                packet_recv,
                packet_send,
                pdr,
            ))
        }
    };
}

fn create_channels<'a, T>(
    drones: &'a [Drone],
    clients: &'a [Client],
    servers: &'a [Server],
) -> impl Iterator<Item = (u8, (Sender<T>, Receiver<T>))> + use<'a, T> {
    chain![
        drones
            .iter()
            .map(|d: &Drone| (d.id, crossbeam_channel::unbounded::<T>())),
        clients
            .iter()
            .map(|c: &Client| (c.id, crossbeam_channel::unbounded::<T>())),
        servers
            .iter()
            .map(|s: &Server| (s.id, crossbeam_channel::unbounded::<T>())),
    ]
}

/// checks the uniqueness of each ID
fn check_id_repetitions(
    drones_id: &[NodeId],
    clients_id: &[NodeId],
    servers_id: &[NodeId],
) -> bool {
    [drones_id, clients_id, servers_id]
        .concat()
        .iter()
        .all_unique()
}

/// checks that every PDR is in [0, 1]
fn check_pdr(drones: &[Drone]) -> bool {
    drones.iter().all(|d| (0.0..=1.0).contains(&d.pdr))
}

/// checks drone connections requirements according to WG
fn check_drone_connections(drones: &[Drone]) -> bool {
    drones.iter().all(|drone| {
        !drone.connected_node_ids.contains(&drone.id)
            && drone.connected_node_ids.iter().all_unique()
    })
}

/// checks client connections requirements according to WG
fn check_client_connections(clients: &[Client], drones_id: &[NodeId]) -> bool {
    clients.iter().all(|client| {
        !client.connected_drone_ids.contains(&client.id)
            && client.connected_drone_ids.iter().all_unique()
            && client
                .connected_drone_ids
                .iter()
                .all(|neighbor| drones_id.contains(neighbor))
            && !client.connected_drone_ids.is_empty()
            && client.connected_drone_ids.len() < 3
    })
}

/// checks servers connections requirements according to WG
fn check_server_connections(servers: &[Server], drones_id: &[NodeId]) -> bool {
    servers.iter().all(|server| {
        !server.connected_drone_ids.contains(&server.id)
            && server.connected_drone_ids.iter().all_unique()
            && server
                .connected_drone_ids
                .iter()
                .all(|neighbor| drones_id.contains(neighbor))
            && server.connected_drone_ids.len() > 1
    })
}

/// check that the topology is a bidirectional and connected graph
fn check_bidirectional_and_connected(
    drones: &[Drone],
    clients: &[Client],
    servers: &[Server],
) -> bool {
    let mut out = vec![drones[0].id];
    let mut queue: VecDeque<(u8, u8)> = VecDeque::new();
    for conn in &drones[0].connected_node_ids {
        queue.push_back((drones[0].id, *conn));
    }

    while let Some((parent, next_id)) = queue.pop_front() {
        if let Some(next_drone) = drones.iter().find(|d| d.id == next_id) {
            if !next_drone.connected_node_ids.contains(&parent) {
                return false;
            }
            if out.contains(&next_id) {
                continue;
            }
            out.push(next_drone.id);
            for conn in &next_drone.connected_node_ids {
                queue.push_back((next_drone.id, *conn));
            }
        } else if let Some(next_client) = clients.iter().find(|c| c.id == next_id) {
            if !next_client.connected_drone_ids.contains(&parent) {
                return false;
            }
            if out.contains(&next_id) {
                continue;
            }
            out.push(next_client.id);
            for conn in &next_client.connected_drone_ids {
                queue.push_back((next_client.id, *conn));
            }
        } else if let Some(next_server) = servers.iter().find(|c| c.id == next_id) {
            if !next_server.connected_drone_ids.contains(&parent) {
                return false;
            }
            if out.contains(&next_id) {
                continue;
            }
            out.push(next_server.id);
            for conn in &next_server.connected_drone_ids {
                queue.push_back((next_server.id, *conn));
            }
        } else {
            return false;
        }
    }

    out.len() == drones.len() + clients.len() + servers.len()
}

/// checks that client/server are leaves of the network accoring to WG requirememts
fn check_connected_only_drones(drones: &[Drone], drones_is: &[NodeId]) -> bool {
    let only_drones: Vec<Drone> = drones
        .iter()
        .map(|d| Drone {
            id: d.id,
            connected_node_ids: d
                .connected_node_ids
                .iter()
                .filter(|neighbor| drones_is.contains(neighbor))
                .copied()
                .collect(),
            pdr: d.pdr,
        })
        .collect();

    check_bidirectional_and_connected(&only_drones, &[], &[])
}

fn check_topology_constraints(drone: &[Drone], client: &[Client], server: &[Server]) -> bool{
    let drones_id: Vec<u8> = drone.iter().map(|drone| drone.id).collect();
    let client_id: Vec<u8> = client.iter().map(|client| client.id).collect();
    let servers_id: Vec<u8> = server.iter().map(|server| server.id).collect();

    if !check_id_repetitions(&drones_id, &client_id, &servers_id) {
        println!("Some IDs are repeated");
        return false;
    }
    if !check_pdr(&drone) {
        println!("Some PDRs are not in the range [0, 1]");
        return false;
    }
    if !check_drone_connections(&drone) {
        println!("Some drones have bad connections");
        return false;
    }
    if !check_client_connections(&client, &drones_id) {
        println!("Some clients have bad connections");
        return false;
    }
    if !check_server_connections(&server, &drones_id) {
        println!("Some servers have bad connections");
        return false;
    }
    if !check_bidirectional_and_connected(&drone, &client, &server) {
        println!("The graph is not bidirectional or connected");
        return false;
    }
    if !check_connected_only_drones(&drone, &drones_id) {
        println!("The graph contains clients/servers that are not at the edges of the network");
        return false;
    }

    true
}

fn main() {
    env::set_var("RUST_LOG", "info");
    let _ = env_logger::try_init();
    let v = [
        create_boxed!(DrDrone),
        create_boxed!(RustDoIt),
        create_boxed!(RustRoveri),
        create_boxed!(RollingDrone),
        create_boxed!(RustafarianDrone),
        create_boxed!(RustezeDrone),
        create_boxed!(RustyDrone),
        create_boxed!(GetDroned),
        create_boxed!(NoSoundDroneRIP),
        create_boxed!(BetterCallDrone),
    ];

    let config_data: String =
        fs::read_to_string("config/test_config.toml").expect("Unable to read config file");
    // having our structs implement the Deserialize trait allows us to use the toml::from_str function to deserialize the config file into each of them
    let Config {
        drone,
        client,
        server,
    }: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    // check topology constraints
    if !check_topology_constraints(&drone, &client, &server){
        return;
    }


    // TODO modulation
    let scl_events: HashMap<NodeId, (Sender<DroneEvent>, Receiver<DroneEvent>)> = drone
        .iter()
        .map(|d: &Drone| (d.id, crossbeam_channel::unbounded()))
        .collect();

    let scl_commands: HashMap<NodeId, (Sender<DroneCommand>, Receiver<DroneCommand>)> = drone
        .iter()
        .map(|d: &Drone| (d.id, crossbeam_channel::unbounded()))
        .collect();

    // TODO: channels for servers and clients
    let scl_web_client_events: HashMap<NodeId, (Sender<WebClientEvent>, Receiver<WebClientEvent>)> =
        client
            .iter()
            .map(|c: &Client| (c.id, crossbeam_channel::unbounded()))
            .collect();

    let scl_web_client_commands: HashMap<
        NodeId,
        (Sender<WebClientCommand>, Receiver<WebClientCommand>),
    > = client
        .iter()
        .map(|c: &Client| (c.id, crossbeam_channel::unbounded()))
        .collect();

    let scl_server_events: HashMap<NodeId, (Sender<ServerEvent>, Receiver<ServerEvent>)> = server
        .iter()
        .map(|s: &Server| (s.id, crossbeam_channel::unbounded()))
        .collect();

    let scl_server_commands: HashMap<NodeId, (Sender<ServerCommand>, Receiver<ServerCommand>)> =
        server
            .iter()
            .map(|s: &Server| (s.id, crossbeam_channel::unbounded()))
            .collect();

    let channels: HashMap<NodeId, (Sender<Packet>, Receiver<Packet>)> =
        create_channels(&drone, &client, &server).collect();

    // spawn drones
    for d in &drone {
        let nbrs: HashMap<NodeId, Sender<Packet>> = d
            .connected_node_ids
            .iter()
            .map(|id: &NodeId| (*id, channels[id].0.clone()))
            .collect();
        let mut new_drone: Box<dyn DroneTrait> = v[usize::from(d.id) % v.len()](
            d.id,
            scl_events[&d.id].0.clone(),
            scl_commands[&d.id].1.clone(),
            channels[&d.id].1.clone(),
            nbrs,
            d.pdr,
        );
        std::thread::spawn(move || new_drone.run());
    }

    // TODO spawn client/server + start scl
    let mut scl_drones_channels: DroneChannels = HashMap::new();
    let mut scl_web_clients_channels: WebClientChannels = HashMap::new();
    let scl_chat_clients_channels: ChatClientChannels = HashMap::new();
    let mut scl_servers_channels: ServerChannels = HashMap::new();
    for d in &drone {
        scl_drones_channels.insert(
            d.id,
            (
                scl_commands[&d.id].0.clone(),
                scl_events[&d.id].1.clone(),
                channels[&d.id].0.clone(),
                channels[&d.id].1.clone(),
            ),
        );
    }
    for c in &client {
        scl_web_clients_channels.insert(
            c.id,
            (
                scl_web_client_commands[&c.id].0.clone(),
                scl_web_client_events[&c.id].1.clone(),
                channels[&c.id].0.clone(),
                channels[&c.id].1.clone(),
            ),
        );
    }
    for s in &server {
        scl_servers_channels.insert(
            s.id,
            (
                scl_server_commands[&s.id].0.clone(),
                scl_server_events[&s.id].1.clone(),
                channels[&s.id].0.clone(),
                channels[&s.id].1.clone(),
            ),
        );
    }

    // spawn clients
    for c in &client {
        let nbrs: HashMap<NodeId, Sender<Packet>> = c
            .connected_drone_ids
            .iter()
            .map(|id: &NodeId| (*id, channels[id].0.clone()))
            .collect();
        let mut new_client = web_client::web_client::WebBrowser::new(
            c.id,
            scl_web_client_events[&c.id].0.clone(),
            scl_web_client_commands[&c.id].1.clone(),
            channels[&c.id].1.clone(),
            nbrs,
        );
        std::thread::spawn(move || new_client.run());
    }

    // spawn servers
    for (idx, s) in server.iter().enumerate() {
        if idx == 0 {
            let nbrs: HashMap<NodeId, Sender<Packet>> = s
                .connected_drone_ids
                .iter()
                .map(|id: &NodeId| (*id, channels[id].0.clone()))
                .collect();
            let mut new_server = servers::servers::TextServer::new(
                s.id,
                scl_server_events[&s.id].0.clone(),
                scl_server_commands[&s.id].1.clone(),
                channels[&s.id].1.clone(),
                nbrs,
            );
            std::thread::spawn(move || new_server.run());
        } else {
            let nbrs: HashMap<NodeId, Sender<Packet>> = s
                .connected_drone_ids
                .iter()
                .map(|id: &NodeId| (*id, channels[id].0.clone()))
                .collect();
            let mut new_server = servers::servers::MediaServer::new(
                s.id,
                scl_server_events[&s.id].0.clone(),
                scl_server_commands[&s.id].1.clone(),
                channels[&s.id].1.clone(),
                nbrs,
            );
            std::thread::spawn(move || new_server.run());
        }
    }

    simulation_controller::run(
        scl_drones_channels,
        scl_web_clients_channels,
        scl_chat_clients_channels,
        scl_servers_channels,
        drone.clone(),
        client.clone(),
        server.clone(),
    );
}
