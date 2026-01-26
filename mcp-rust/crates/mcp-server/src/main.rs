use mcp_core::{CoreConfig, Message, Role};

fn main() {
    let config = CoreConfig::dev("mcp-server");
    announce_role(Role::Server, &config);

    let handshake = Message::new("mcp-client", &config.service_name, "hello from client");
    handle_message(&config, handshake);
}

/// Show a short description of what the running binary is doing.
fn announce_role(role: Role, config: &CoreConfig) {
    println!(
        "Starting {} on port {} ({:?})",
        config.service_name, config.port, role
    );
}

/// Process a message that arrived at the server.
fn handle_message(config: &CoreConfig, message: Message) {
    println!("Received: {}", message.summary());

    let reply = Message::new(
        &config.service_name,
        &message.sender,
        "welcome to the mcp mesh",
    );

    println!("Reply: {}", reply.summary());
}
