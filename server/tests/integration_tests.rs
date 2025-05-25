use flatbuffers::FlatBufferBuilder;
use shared::generated::{PlayerCommand, PlayerCommands, PlayerCommandsArgs};
use shared::state::{GameState, PlayerState, SpawnPoint, Vec2};
use std::collections::HashMap;
use std::net::UdpSocket;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TEST_SERVER_ADDR: &str = "127.0.0.1:9001";
const TEST_CLIENT_ADDR: &str = "127.0.0.1:9002";

#[test]
fn test_client_server_communication() {
    // Start the server in a background thread
    let server_thread = thread::spawn(|| {
        let server_socket =
            UdpSocket::bind(TEST_SERVER_ADDR).expect("Failed to bind server socket");
        let mut buf = [0u8; 2048];

        // Just handle one message for the test
        if let Ok((amt, src)) = server_socket.recv_from(&mut buf) {
            // Create a simple game state response
            let mut builder = FlatBufferBuilder::new();
            let mut game_state = GameState {
                players: HashMap::new(),
                collidables: vec![],
                width: 800.0,
                height: 600.0,
                spawn_point: SpawnPoint { x: 100.0, y: 100.0 },
            };

            // Add a test player
            let player = PlayerState {
                id: 1,
                name: "test_player".to_string(),
                pos: Vec2::new(100.0, 100.0),
                vel: Vec2::new(0.0, 0.0),
                grounded: true,
                jump_timer: 0.0,
                color: shared::generated::Color::Red,
                size: 32.0,
            };
            game_state.players.insert(1, player);

            // Serialize and send back
            let response = game_state.serialize(&mut builder, 1, 0);
            server_socket
                .send_to(response, src)
                .expect("Failed to send response");
        }
    });

    // Give the server a moment to start
    thread::sleep(Duration::from_millis(100));

    // Create a test client socket
    let client_socket = UdpSocket::bind(TEST_CLIENT_ADDR).expect("Failed to bind client socket");
    client_socket
        .set_read_timeout(Some(Duration::from_secs(2))) // Increased timeout
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

    // Wait for server thread to finish
    server_thread.join().unwrap();
}
