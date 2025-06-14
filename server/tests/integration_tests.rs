use flatbuffers::FlatBufferBuilder;
use shared::generated::{PlayerCommand, PlayerCommands, PlayerCommandsArgs};
use shared::state::{GameState, PlayerState, SceneObject, SpawnPoint, Vec2};
use std::collections::{BinaryHeap, HashMap};
use std::net::UdpSocket;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TEST_SERVER_ADDR: &str = "127.0.0.1:9001";
const TEST_CLIENT_ADDR: &str = "127.0.0.1:9002";

#[test]
fn test_client_server_communication() {
    println!("Starting test_client_server_communication");

    // Start the server in a background thread
    let server_thread = thread::spawn(|| {
        println!("Server thread starting");
        let server_socket =
            UdpSocket::bind(TEST_SERVER_ADDR).expect("Failed to bind server socket");
        println!("Server socket bound successfully");
        let mut buf = [0u8; 2048];

        // Just handle one message for the test
        match server_socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                println!("Server received {} bytes from {}", amt, src);
                // Create a simple game state response
                let mut builder = FlatBufferBuilder::new();
                let mut game_state = GameState {
                    players: HashMap::new(),
                    collidables: vec![],
                    width: 800.0,
                    height: 600.0,
                    spawn_point: SpawnPoint { x: 100.0, y: 100.0 },
                    win_point: SceneObject {
                        x: 2750.0,
                        y: 159.0,
                        w: 100.0,
                        h: 20.0,
                    },
                    cached_dt_micros: 0,
                    scheduled_commands: BinaryHeap::new(),
                };

                // Add a client player (id: 1)
                let client_player = PlayerState {
                    id: 1,
                    name: "client_player".to_string(),
                    pos: Vec2::new(100.0, 100.0),
                    vel: Vec2::new(0.0, 0.0),
                    grounded: true,
                    jump_timer: 0.0,
                    color: shared::generated::Color::Red,
                    size: 32.0,
                };

                // Add another player (id: 2) that should appear in the players list
                let other_player = PlayerState {
                    id: 2,
                    name: "other_player".to_string(),
                    pos: Vec2::new(200.0, 200.0),
                    vel: Vec2::new(0.0, 0.0),
                    grounded: true,
                    jump_timer: 0.0,
                    color: shared::generated::Color::Blue,
                    size: 32.0,
                };

                game_state.players.insert(1, client_player);
                game_state.players.insert(2, other_player);
                println!(
                    "Created game state with {} players",
                    game_state.players.len()
                );

                // Serialize and send back
                let response = game_state.serialize(
                    &mut builder,
                    1,
                    0,
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_micros() as u64,
                );
                println!("Serialized response is {} bytes", response.len());
                match server_socket.send_to(response, src) {
                    Ok(sent) => println!("Server sent {} bytes", sent),
                    Err(e) => println!("Server failed to send response: {}", e),
                }
            }
            Err(e) => {
                println!("Server failed to receive: {}", e);
                panic!("Server failed to receive: {}", e);
            }
        }
        println!("Server thread finishing");
    });

    // Give the server a moment to start
    thread::sleep(Duration::from_millis(500)); // Increased startup delay
    println!("Main thread continuing after server start delay");

    // Create a test client socket
    let client_socket = UdpSocket::bind(TEST_CLIENT_ADDR).expect("Failed to bind client socket");
    client_socket
        .set_read_timeout(Some(Duration::from_secs(5))) // Increased timeout further
        .unwrap();
    println!("Client socket bound successfully");

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
    println!("Created command packet of {} bytes", packet.len());

    // Send packet to server
    match client_socket.send_to(packet, TEST_SERVER_ADDR) {
        Ok(sent) => println!("Client sent {} bytes", sent),
        Err(e) => println!("Client failed to send: {}", e),
    }

    // Wait for response
    let mut buf = [0u8; 2048];
    println!("Client waiting for response...");
    match client_socket.recv_from(&mut buf) {
        Ok((amt, src)) => {
            println!("Client received {} bytes from {}", amt, src);
            // Try to parse the response as a GameState
            let (game_state, client_player, sequence, server_delay) =
                GameState::deserialize(&buf[..amt]);
            println!(
                "Deserialized game state has {} players, client_player_id: {}, sequence: {}",
                game_state.players.len(),
                client_player.id,
                sequence
            );

            // Verify we got both the client player and other player
            assert_eq!(client_player.id, 1, "Client player should have id 1");
            assert_eq!(game_state.players.len(), 1, "Should have 1 other player");
            assert!(
                game_state.players.contains_key(&2),
                "Other player should have id 2"
            );
        }
        Err(e) => {
            println!("Client failed to receive: {}", e);
            panic!("Failed to receive response: {}", e);
        }
    }

    // Wait for server thread to finish
    match server_thread.join() {
        Ok(_) => println!("Server thread joined successfully"),
        Err(e) => println!("Server thread join failed: {:?}", e),
    }
    println!("Test finishing");
}
