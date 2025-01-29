#![warn(clippy::pedantic)]

use ap2024_rustinpeace_nosounddrone::NoSoundDroneRIP;
use common::slc_commands::{ClientCommand, ClientEvent, ServerCommand, ServerEvent};
use common::{Client as ClientTrait, Server as ServerTrait};
use crossbeam_channel::{Receiver, Sender};
use dr_ones::Drone as DrDrone;
use getdroned::GetDroned;
use itertools::chain;
use rolling_drone::RollingDrone;
use rust_do_it::RustDoIt;
use rust_roveri::RustRoveri;
use rustafarian_drone::RustafarianDrone;
use rusteze_drone::RustezeDrone;
use rusty_drones::RustyDrone;
use simulation_controller::SimulationController;
use std::collections::HashMap;
use std::fs;
use wg_2024::config::Config;
use wg_2024::config::{Client, Drone, Server};
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone as DroneTrait;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

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

fn main() {
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
    ];

    let config_data: String =
        fs::read_to_string("config/config.toml").expect("Unable to read config file");
    // having our structs implement the Deserialize trait allows us to use the toml::from_str function to deserialize the config file into each of them
    let Config {
        drone,
        client,
        server,
    }: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

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
    let scl_client_events: HashMap<NodeId, (Sender<ClientEvent>, Receiver<ClientEvent>)> = client
        .iter()
        .map(|c: &Client| (c.id, crossbeam_channel::unbounded()))
        .collect();

    let scl_client_commands: HashMap<NodeId, (Sender<ClientCommand>, Receiver<ClientCommand>)> =
        client
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
    let mut scl_drones_channels: HashMap<NodeId, (Sender<DroneCommand>, Receiver<DroneEvent>)> =
        HashMap::new();
    let mut scl_clients_channels: HashMap<NodeId, (Sender<ClientCommand>, Receiver<ClientEvent>)> =
        HashMap::new();
    let mut scl_servers_channels: HashMap<NodeId, (Sender<ServerCommand>, Receiver<ServerEvent>)> =
        HashMap::new();
    for d in &drone {
        scl_drones_channels.insert(
            d.id,
            (scl_commands[&d.id].0.clone(), scl_events[&d.id].1.clone()),
        );
    }
    for c in &client {
        scl_clients_channels.insert(
            c.id,
            (
                scl_client_commands[&c.id].0.clone(),
                scl_client_events[&c.id].1.clone(),
            ),
        );
    }
    for s in &server {
        scl_servers_channels.insert(
            s.id,
            (
                scl_server_commands[&s.id].0.clone(),
                scl_server_events[&s.id].1.clone(),
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
        let mut new_client = web_client::WebBrowser::new(
            c.id,
            scl_client_events[&c.id].0.clone(),
            scl_client_commands[&c.id].1.clone(),
            channels[&c.id].1.clone(),
            nbrs,
        );
        std::thread::spawn(move || new_client.run());
    }

    // spawn servers
    for s in &server {
        let nbrs: HashMap<NodeId, Sender<Packet>> = s
            .connected_drone_ids
            .iter()
            .map(|id: &NodeId| (*id, channels[id].0.clone()))
            .collect();
        let mut new_server = servers::GenericServer::new(
            s.id,
            scl_server_events[&s.id].0.clone(),
            scl_server_commands[&s.id].1.clone(),
            channels[&s.id].1.clone(),
            nbrs,
        );
        std::thread::spawn(move || new_server.run());
    }


    let mut scl = SimulationController::new(
        0,
        scl_drones_channels,
        scl_clients_channels,
        scl_servers_channels,
        drone.clone(),
        client.clone(),
        server.clone(),
    );
    // std::thread::spawn(move || scl.run()); // Apparently this cant be done because "EventLoop must be created on the main thread"
    scl.run();
}
