

use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};
use bevy_renet::renet::{ClientId, DefaultChannel, RenetServer, ServerEvent};

use crate::{net::{CPacket, RenetServerHelper, SPacket, PROTOCOL_ID}, util::current_timestamp_millis};

#[derive(Default)]
pub struct ServerInfo {
    online_players: HashMap<ClientId, String>,
}

pub fn server_sys(
    mut server_events: EventReader<ServerEvent>,
    mut server: ResMut<RenetServer>,
    mut serverinfo: Local<ServerInfo>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id: _ } => {
            }
            ServerEvent::ClientDisconnected { client_id, reason: _ } => {
                if let Some(player) = serverinfo.online_players.remove(client_id) {
                    server.broadcast_packet_chat(format!("Player {} left. ({}/N)", player, serverinfo.online_players.len()));
                }
            }
        }
    }

    // Receive message from all clients
    for client_id in server.clients_id() {
        while let Some(bytes) = server.receive_message(client_id, DefaultChannel::ReliableOrdered) {
            info!("Server Received: {}", String::from_utf8_lossy(&bytes));
            let packet: CPacket = bincode::deserialize(&bytes[..]).unwrap();
            match packet {
                CPacket::Handshake { protocol_version } => {
                    if protocol_version < PROTOCOL_ID {
                        server.send_packet_disconnect(client_id, "Client outdated.".into());
                    } else if protocol_version > PROTOCOL_ID {
                        server.send_packet_disconnect(client_id, "Server outdated.".into());
                    }
                }
                CPacket::ServerQuery {} => {
                    server.send_packet(
                        client_id,
                        &SPacket::ServerInfo {
                            motd: "Motd".into(),
                            num_players_limit: 64,
                            num_players_online: 0,
                            protocol_version: PROTOCOL_ID,
                            favicon: "None".into(),
                        },
                    );
                }
                CPacket::Ping { client_time } => {
                    server.send_packet(
                        client_id,
                        &SPacket::Pong {
                            client_time: client_time,
                            server_time: current_timestamp_millis(),
                        },
                    );
                }
                CPacket::Login { uuid, access_token, username } => {
                    if uuid != client_id.raw() {
                        server.send_packet_disconnect(client_id, "Login UUID not equals ClientID".into());
                        continue;
                    }
                    info!("Login Requested: {} {} {}", uuid, access_token, username);

                    if serverinfo.online_players.contains_key(&client_id) {
                        server.send_packet_disconnect(client_id, format!("Player {} already logged in", &username));
                        continue;
                    }
                    std::thread::sleep(Duration::from_millis(2000));

                    server.send_packet(client_id, &SPacket::LoginSuccess {});

                    server.broadcast_packet_chat(format!("Player {} joined. ({}/N)", &username, serverinfo.online_players.len()+1));
                    serverinfo.online_players.insert(client_id, username);
                }
                CPacket::ChatMessage { message } => {
                    server.broadcast_packet_chat(format!("<ClientX>: {}", message.clone()));
                }
            }

        }
    }
}