use flatbuffers::FlatBufferBuilder;
use shared::generated::{PlayerCommand, PlayerCommands, PlayerCommandsArgs};
use shared::state::GameState;
use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TEST_SERVER_ADDR: &str = "127.0.0.1:9001";
const TEST_CLIENT_ADDR: &str = "127.0.0.1:9002";

#[test]
fn test_client_server_communication() {
    // Create a test client socket
    let client_socket = UdpSocket::bind(TEST_CLIENT_ADDR).expect("Failed to bind client socket");
    client_socket
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();

    // Create and send a test packet
    let mut builder = FlatBufferBuilder::new();
    let commands = vec![PlayerCommand::MoveRight, PlayerCommand::Jump];
    let commands_vec = builder.create_vector(&commands);
    let player_commands = PlayerCommands::create(
        &mut builder,
        &PlayerCommandsArgs {
            sequence: 1,
            dt_micro: 16667, // ~60fps
            commands: Some(commands_vec),
            client_timestamp_micro: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64,
        },
    );
    builder.finish(player_commands, None);
    let packet = builder.finished_data();

    // Send packet to server
    client_socket
        .send_to(packet, TEST_SERVER_ADDR)
        .expect("Failed to send packet");

    // Wait for response
    let mut buf = [0u8; 2048];
    match client_socket.recv_from(&mut buf) {
        Ok((amt, _)) => {
            // Try to parse the response as a GameState
            let (game_state, _, _) = GameState::deserialize(&buf[..amt]);
            assert!(
                !game_state.players.is_empty(),
                "Should receive game state with players"
            );
        }
        Err(e) => panic!("Failed to receive response: {}", e),
    }
}
