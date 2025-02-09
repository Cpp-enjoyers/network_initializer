use std::fs;
use wg_2024::config::Config;

use crate::check_topology_constraints;

#[test]
fn double_chain() {
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
fn tree() {
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
fn subnet1() {
    let config_data: String =
        fs::read_to_string("config/sub_net_1.toml").expect("Unable to read config file");
    // having our structs implement the Deserialize trait allows us to use the toml::from_str function to deserialize the config file into each of them
    let Config {
        drone,
        client,
        server,
    }: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    assert!(check_topology_constraints(&drone, &client, &server));
}

#[test]
fn subnet2() {
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

#[test]
fn star() {
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
fn butterfly() {
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
