use std::collections::HashMap;

use chat_server_client::client::ChatClient;
use common::Client as ClientTrait;
use crossbeam_channel::{Receiver, Sender};
use web_client::web_client::WebBrowser;
use wg_2024::{network::NodeId, packet::Packet};

/// handy alias for shorter naming
#[allow(type_alias_bounds)]
type ClientFn<C: ClientTrait> = fn(
    NodeId,
    Sender<C::U>,
    Receiver<C::T>,
    Receiver<Packet>,
    HashMap<NodeId, Sender<Packet>>,
) -> C;

/// create a closure that returns a scpecific Drone as a
/// boxed trait obj
#[macro_export]
macro_rules! create_boxed_drone {
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

/// enum to wrap closures with different signatures
pub(super) enum ClientFuncs {
    WebFn(ClientFn<WebBrowser>),
    ChatFn(ClientFn<ChatClient>),
}

/// create a closure that returns a scpecific Server as a
/// boxed trait obj
#[macro_export]
macro_rules! create_boxed_server {
    ($type:ty) => {
        |id: NodeId,
         controller_send: Sender<ServerEvent>,
         controller_recv: Receiver<ServerCommand>,
         packet_recv: Receiver<Packet>,
         packet_send: HashMap<NodeId, Sender<Packet>>|
         -> Box<dyn ServerTrait> {
            Box::new(<$type>::new(
                id,
                controller_send,
                controller_recv,
                packet_recv,
                packet_send,
            ))
        }
    };
}
