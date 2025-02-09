/*!
    C++Enjoyers: Network Initializer

    This crate implements the network initializer as described in WG protocol.
    It reads a TOML file containing the topology informations, it checks all the
    constraints that the protocol put on the topology and, finally, it spawns
    all the nodes, channels and the simulation controller.
*/

#![warn(clippy::pedantic)]
#![deny(nonstandard_style)]
#![warn(missing_docs)]
#![deny(unsafe_code)]

use ap2024_rustinpeace_nosounddrone::NoSoundDroneRIP;
use ap2024_unitn_cppenjoyers_webservers::{MediaServer, TextServer};
use chat_server_client::client::ChatClient;
use chat_server_client::server::ChatServer;
use common::slc_commands::{
    ChatClientCommand, ChatClientEvent, ServerCommand, ServerEvent, WebClientCommand,
    WebClientEvent,
};
use common::{Client as ClientTrait, Server as ServerTrait};
use crossbeam_channel::{Receiver, Sender};
use dr_ones::Drone as DrDrone;
use drone_bettercalldrone::BetterCallDrone;
use factories::ClientFuncs;
use getdroned::GetDroned;
use itertools::chain;
use log::error;
use rolling_drone::RollingDrone;
use rust_do_it::RustDoIt;
use rust_roveri::RustRoveri;
use rustafarian_drone::RustafarianDrone;
use rusteze_drone::RustezeDrone;
use rusty_drones::RustyDrone;
use std::collections::HashMap;
use std::env;
use std::fs;
use topology_utils::check_topology_constraints;
use web_client::web_client::WebBrowser;
use wg_2024::config::Config;
use wg_2024::config::{Client, Drone, Server};
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone as DroneTrait;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;
// use chat_common;

mod factories;
mod topology_utils;

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

// IDK how I should name it so that's it
#[allow(clippy::type_complexity)]
fn create_scl_channels<T, U>(
    v: &[U],
    f: fn(&U) -> (NodeId, (Sender<T>, Receiver<T>)),
) -> HashMap<NodeId, (Sender<T>, Receiver<T>)> {
    v.iter().map(f).collect()
}

fn create_channels<'a, T>(
    drones: &'a [Drone],
    clients: &'a [Client],
    servers: &'a [Server],
) -> impl Iterator<Item = (NodeId, (Sender<T>, Receiver<T>))> + use<'a, T> {
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

// It's fair to have a longer main here... most of the lines are just constructors
// after the "cargo fmt" command
#[allow(clippy::too_many_lines)]
fn main() {
    env::set_var("RUST_LOG", "info");
    let _ = env_logger::try_init();

    let drone_factory = [
        create_boxed_drone!(DrDrone),
        create_boxed_drone!(RustDoIt),
        create_boxed_drone!(RustRoveri),
        create_boxed_drone!(RollingDrone),
        create_boxed_drone!(RustafarianDrone),
        create_boxed_drone!(RustezeDrone),
        create_boxed_drone!(RustyDrone),
        create_boxed_drone!(GetDroned),
        create_boxed_drone!(NoSoundDroneRIP),
        create_boxed_drone!(BetterCallDrone),
    ];
    let client_factory = [
        ClientFuncs::WebFn(WebBrowser::new),
        ClientFuncs::ChatFn(ChatClient::new),
    ];
    let server_factory = [
        create_boxed_server!(TextServer),
        create_boxed_server!(MediaServer),
        create_boxed_server!(ChatServer),
    ];

    let config_data: String =
        fs::read_to_string("config/test_chat_config.toml").expect("Unable to read config file");
    // having our structs implement the Deserialize trait allows us to use the toml::from_str function to deserialize the config file into each of them
    let Config {
        drone,
        client,
        server,
    }: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    // check topology constraints
    if !check_topology_constraints(&drone, &client, &server) {
        error!("Inconsistent topology");
        return;
    }

    let mut scl_drones_channels: DroneChannels = HashMap::new();
    let mut scl_web_clients_channels: WebClientChannels = HashMap::new();
    let mut scl_chat_clients_channels: ChatClientChannels = HashMap::new();
    let mut scl_servers_channels: ServerChannels = HashMap::new();

    let scl_events: HashMap<NodeId, (Sender<DroneEvent>, Receiver<DroneEvent>)> =
        create_scl_channels(&drone, |d| (d.id, crossbeam_channel::unbounded()));
    let scl_commands: HashMap<NodeId, (Sender<DroneCommand>, Receiver<DroneCommand>)> =
        create_scl_channels(&drone, |d| (d.id, crossbeam_channel::unbounded()));
    let scl_server_events: HashMap<NodeId, (Sender<ServerEvent>, Receiver<ServerEvent>)> =
        create_scl_channels(&server, |s| (s.id, crossbeam_channel::unbounded()));
    let scl_server_commands: HashMap<NodeId, (Sender<ServerCommand>, Receiver<ServerCommand>)> =
        create_scl_channels(&server, |s| (s.id, crossbeam_channel::unbounded()));
    let channels: HashMap<NodeId, (Sender<Packet>, Receiver<Packet>)> =
        create_channels(&drone, &client, &server).collect();

    // drones and servers spawn (trivial)
    for d in &drone {
        let nbrs: HashMap<NodeId, Sender<Packet>> = d
            .connected_node_ids
            .iter()
            .map(|id: &NodeId| (*id, channels[id].0.clone()))
            .collect();
        let mut new_drone: Box<dyn DroneTrait> = drone_factory
            [usize::from(d.id) % drone_factory.len()](
            d.id,
            scl_events[&d.id].0.clone(),
            scl_commands[&d.id].1.clone(),
            channels[&d.id].1.clone(),
            nbrs,
            d.pdr,
        );
        std::thread::spawn(move || new_drone.run());
    }
    for s in &server {
        let nbrs: HashMap<NodeId, Sender<Packet>> = s
            .connected_drone_ids
            .iter()
            .map(|id: &NodeId| (*id, channels[id].0.clone()))
            .collect();
        let mut new_server: Box<dyn ServerTrait> = server_factory
            [usize::from(s.id) % server_factory.len()](
            s.id,
            scl_server_events[&s.id].0.clone(),
            scl_server_commands[&s.id].1.clone(),
            channels[&s.id].1.clone(),
            nbrs,
        );
        std::thread::spawn(move || new_server.run());
    }

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

    // clients spawn
    for c in &client {
        let nbrs: HashMap<NodeId, Sender<Packet>> = c
            .connected_drone_ids
            .iter()
            .map(|id: &NodeId| (*id, channels[id].0.clone()))
            .collect();
        match client_factory[usize::from(c.id) % client_factory.len()] {
            ClientFuncs::WebFn(f) => {
                let (c1, c2) = crossbeam_channel::unbounded();
                let (c3, c4) = crossbeam_channel::unbounded();
                let mut new_client = f(
                    c.id,
                    c1.clone(),
                    c4.clone(),
                    channels[&c.id].1.clone(),
                    nbrs,
                );
                scl_web_clients_channels.insert(
                    c.id,
                    (
                        c3.clone(),
                        c2.clone(),
                        channels[&c.id].0.clone(),
                        channels[&c.id].1.clone(),
                    ),
                );
                std::thread::spawn(move || new_client.run());
            }
            ClientFuncs::ChatFn(f) => {
                let (c1, c2) = crossbeam_channel::unbounded();
                let (c3, c4) = crossbeam_channel::unbounded();
                let mut new_client = f(
                    c.id,
                    c1.clone(),
                    c4.clone(),
                    channels[&c.id].1.clone(),
                    nbrs,
                );
                scl_chat_clients_channels.insert(
                    c.id,
                    (
                        c3.clone(),
                        c2.clone(),
                        channels[&c.id].0.clone(),
                        channels[&c.id].1.clone(),
                    ),
                );
                std::thread::spawn(move || new_client.run());
            }
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
