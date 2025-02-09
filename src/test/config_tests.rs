use petgraph::prelude::DiGraphMap;
use wg_2024::config::Config;

use crate::{
    test::correct_config,
    topology_utils::{
        check_bidirectional, check_client_connections, check_drone_connections,
        check_id_repetitions, check_pdr, check_server_connections,
    },
};

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
fn test_graph_checks() {
    let g1 = DiGraphMap::from_edges([(1, 2, 1), (2, 1, 1), (2, 3, 1), (3, 2, 1)]);
    assert!(check_bidirectional(&g1));

    let g2 = DiGraphMap::from_edges([(1, 2, 1), (2, 1, 1), (2, 3, 1), (3, 2, 1), (3, 1, 1)]);
    assert!(!check_bidirectional(&g2));

    let g4 = DiGraphMap::from_edges([
        (1, 2, 1),
        (2, 1, 1),
        (2, 3, 1),
        (3, 2, 1),
        (3, 1, 1),
        (3, 5, 1),
    ]);
    assert!(!check_bidirectional(&g4));
}
