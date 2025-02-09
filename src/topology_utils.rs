use itertools::{chain, Itertools};
use log::error;
use petgraph::{
    algo::connected_components,
    prelude::DiGraphMap,
    Direction::{Incoming, Outgoing},
};
use wg_2024::{
    config::{Client, Drone, Server},
    network::NodeId,
};

#[inline]
/// checks that every PDR is in [0, 1]
pub(super) fn check_pdr(drones: &[Drone]) -> bool {
    drones.iter().all(|d| (0.0..=1.0).contains(&d.pdr))
}

#[inline]
fn check_connection(id: NodeId, connected: &[NodeId]) -> bool {
    !connected.contains(&id) && connected.iter().all_unique()
}

#[inline]
fn check_subset(connected: &[NodeId], drones_ids: &[NodeId]) -> bool {
    connected
        .iter()
        .all(|neighbor| drones_ids.contains(neighbor))
}

/// checks the uniqueness of each ID
pub(super) fn check_id_repetitions(
    drones_id: &[NodeId],
    clients_id: &[NodeId],
    servers_id: &[NodeId],
) -> bool {
    chain![drones_id, clients_id, servers_id].all_unique()
}

/// checks drone connections requirements according to WG
pub(super) fn check_drone_connections(drones: &[Drone]) -> bool {
    drones
        .iter()
        .all(|drone| check_connection(drone.id, &drone.connected_node_ids))
}

/// checks client connections requirements according to WG
pub(super) fn check_client_connections(clients: &[Client], drones_ids: &[NodeId]) -> bool {
    clients.iter().all(|client| {
        (1..3).contains(&client.connected_drone_ids.len())
            && check_subset(&client.connected_drone_ids, drones_ids)
            && check_connection(client.id, &client.connected_drone_ids)
    })
}

/// checks servers connections requirements according to WG
pub(super) fn check_server_connections(servers: &[Server], drones_ids: &[NodeId]) -> bool {
    servers.iter().all(|server| {
        server.connected_drone_ids.len() > 1
            && check_subset(&server.connected_drone_ids, drones_ids)
            && check_connection(server.id, &server.connected_drone_ids)
    })
}

fn check_bidirectional(graph: &DiGraphMap<u8, u8>) -> bool {
    for node in graph.nodes() {
        if !graph
            .edges_directed(node, Incoming)
            .map(|(_, b, _)| b)
            .sorted()
            .eq(graph
                .edges_directed(node, Outgoing)
                .map(|(a, _, _)| a)
                .sorted())
        {
            error!("Graph is not bidirectional, problematic node: {node}");
            return false;
        }
    }
    true
}

/// Checks that the topology respects all the necessary constraints
pub(super) fn check_topology_constraints(
    drones: &[Drone],
    clients: &[Client],
    servers: &[Server],
) -> bool {
    let drones_id: Vec<u8> = drones.iter().map(|drone| drone.id).collect();
    let client_id: Vec<u8> = clients.iter().map(|client| client.id).collect();
    let servers_id: Vec<u8> = servers.iter().map(|server| server.id).collect();

    if !check_id_repetitions(&drones_id, &client_id, &servers_id)
        || !check_pdr(drones)
        || !check_drone_connections(drones)
        || !check_client_connections(clients, &drones_id)
        || !check_server_connections(servers, &drones_id)
    {
        error!("Duplicates or self-loops in graph edges");
        return false;
    }

    let graph_init: Vec<(u8, u8, u8)> = drones
        .iter()
        .flat_map(|d1| {
            d1.connected_node_ids
                .iter()
                .map(|&d2| (d1.id, d2, 1))
                .collect::<Vec<(u8, u8, u8)>>()
        })
        .collect();
    let mut graph: DiGraphMap<NodeId, u8> = DiGraphMap::from_iter(graph_init);
    let components: usize = connected_components(&graph);
    for server in servers {
        for &nbr in &server.connected_drone_ids {
            graph.add_edge(server.id, nbr, 1);
        }
    }
    for client in clients {
        for &nbr in &client.connected_drone_ids {
            graph.add_edge(client.id, nbr, 1);
        }
    }

    if components > 1 || !check_bidirectional(&graph) {
        error!("Graph is not connected");
        return false;
    }

    true
}
