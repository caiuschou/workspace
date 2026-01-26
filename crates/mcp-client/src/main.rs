use std::{env, thread::sleep, time::Duration};

use mcp_client::{
    client::{Client, ClientError, ClientOptions},
    stdio::{StdioClientTransport, StdioClientTransportError, StdioServerParameters, StdioStream},
};
use mcp_core::{CoreConfig, Role};

fn main() {
    if let Err(error) = run() {
        eprintln!("Client error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), ClientError<StdioClientTransportError>> {
    let config = CoreConfig::dev("mcp-client");
    announce_role(Role::Client, &config);

    let (command, args) = resolve_server_command();
    let transport = StdioClientTransport::new(
        StdioServerParameters::new(command)
            .args(args)
            .stderr(StdioStream::Inherit),
    );

    let mut client = Client::new(
        transport,
        ClientOptions::new(&config.service_name).with_version("0.1.0"),
    );

    client.connect()?;

    println!(
        "Client expects handshake on port {} and will reuse {:?}.",
        config.port, config.environment
    );

    sleep(Duration::from_secs(1));
    client.close()?;
    Ok(())
}

fn announce_role(role: Role, config: &CoreConfig) {
    println!(
        "Running {} in {:?} mode (role: {:?})",
        config.service_name, config.environment, role
    );
}
#[cfg(target_os = "windows")]
const DEFAULT_SERVER_COMMAND: &str = "powershell";

#[cfg(not(target_os = "windows"))]
const DEFAULT_SERVER_COMMAND: &str = "cat";

#[cfg(target_os = "windows")]
const DEFAULT_SERVER_ARGS: &[&str] = &["-NoLogo", "-Command", "Get-Content -Raw -"];

#[cfg(not(target_os = "windows"))]
const DEFAULT_SERVER_ARGS: &[&str] = &[];

fn resolve_server_command() -> (String, Vec<String>) {
    if let Ok(command) = env::var("MCP_CLIENT_STDIO_COMMAND") {
        let args = env::var("MCP_CLIENT_STDIO_ARGS")
            .map(|value| value.split_whitespace().map(String::from).collect())
            .unwrap_or_default();
        return (command, args);
    }

    let mut args_iter = env::args().skip(1);
    let command = args_iter
        .next()
        .unwrap_or_else(|| DEFAULT_SERVER_COMMAND.to_string());
    let rest: Vec<String> = args_iter.collect();

    if rest.is_empty() {
        (
            command,
            DEFAULT_SERVER_ARGS
                .iter()
                .map(|arg| arg.to_string())
                .collect(),
        )
    } else {
        (command, rest)
    }
}
