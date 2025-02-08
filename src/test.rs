use std::{fs, vec};

use wg_2024::config::{Client, Config, Drone, Server};

use crate::{
    check_bidirectional_and_connected, check_client_connections, check_connected_only_drones,
    check_drone_connections, check_id_repetitions, check_pdr, check_server_connections, check_topology_constraints,
};

fn correct_config() -> Config {
    Config {
        drone: vec![
            Drone {
                id: 0,
                connected_node_ids: vec![1, 3, 11],
                pdr: 0.,
            },
            Drone {
                id: 1,
                connected_node_ids: vec![0, 2],
                pdr: 0.8,
            },
            Drone {
                id: 2,
                connected_node_ids: vec![1, 3, 12],
                pdr: 1.,
            },
            Drone {
                id: 3,
                connected_node_ids: vec![0, 2, 12],
                pdr: 0.4,
            },
        ],

        client: vec![Client {
            id: 11,
            connected_drone_ids: vec![0],
        }],

        server: vec![Server {
            id: 12,
            connected_drone_ids: vec![2, 3],
        }],
    }
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

    client.push(Client {
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