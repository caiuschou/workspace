mod client;
mod config;
mod tools;

use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;

use futures::executor::block_on;
use mcp_core::stdio::{serialize_message, JsonRpcMessage, ReadBuffer};
use mcp_core::types::*;
use mcp_server::{McpServer, ServerOptions};

use client::GithubState;
use tools::register_all_tools;

fn main() {
    if let Err(error) = run() {
        eprintln!("GitHub MCP server error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // 检查环境变量
    let token = std::env::var("GITHUB_TOKEN").ok();
    if token.is_none() {
        eprintln!("Warning: GITHUB_TOKEN not set. Set it for authenticated requests.");
    }

    // 服务器信息
    let server_info = Implementation {
        base: BaseMetadata {
            name: "github-mcp".to_string(),
            title: Some("GitHub MCP Server".to_string()),
        },
        icons: Icons::default(),
        version: "0.2.0".to_string(),
        website_url: Some("https://github.com".to_string()),
        description: Some(
            "GitHub MCP server providing access to GitHub REST API v3.\n\n\
            Supports repositories, issues, pull requests, files, branches, and commits.\n\n\
            Set GITHUB_TOKEN environment variable for authenticated requests."
                .to_string(),
        ),
    };

    // 服务器配置
    let mut server_options = ServerOptions::default();
    server_options.capabilities = Some(ServerCapabilities {
        tools: Some(mcp_core::types::ToolCapabilities {
            list_changed: None,
        }),
        ..Default::default()
    });
    server_options.instructions = Some(
        "GitHub MCP server that provides access to GitHub API.\n\n\
        Token Management:\n\
        - list_tokens: List all configured tokens\n\
        - add_token: Add a new named token\n\
        - remove_token: Remove a token\n\
        - set_default_token: Set default token\n\
        - use_token: Switch to a specific token\n\n\
        Repository Tools:\n\
        - get_repository, list_branches, get_branch, list_commits\n\n\
        Issue Tools:\n\
        - list_issues, get_issue, create_issue, update_issue, list_issue_comments, create_issue_comment\n\n\
        Pull Request Tools:\n\
        - list_pulls, get_pull, create_pull, merge_pull, list_pull_files, list_pull_comments, create_pull_comment\n\n\
        File Tools:\n\
        - get_file, list_directory, create_file, update_file, delete_file, get_readme\n\n\
        Branch & Commit Tools:\n\
        - merge_branch, get_commit, compare_commits\n\n\
        Set GITHUB_TOKEN environment variable or use add_token to configure."
            .to_string(),
    );

    // 创建服务器
    let mut server = McpServer::new(server_info.clone(), server_options);
    let state = Arc::new(GithubState::new().with_config());

    // 注册所有工具
    register_all_tools(&mut server, state)?;

    // 启动 stdio 通信循环
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut buffer = String::new();
    let mut read_buffer = ReadBuffer::default();

    loop {
        buffer.clear();
        let bytes_read = reader.read_line(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        read_buffer.append(buffer.as_bytes());
        while let Ok(Some(message)) = read_buffer.read_message() {
            match message {
                JsonRpcMessage::Request(request) => {
                    let response = block_on(server.server().handle_request(request, None))?;
                    let response_msg = JsonRpcMessage::Result(response);
                    let serialized = serialize_message(&response_msg)?;
                    stdout.write_all(serialized.as_bytes())?;
                    stdout.flush()?;
                }
                JsonRpcMessage::Notification(notification) => {
                    block_on(server.server().handle_notification(notification, None))?;
                }
                JsonRpcMessage::Result(_) => {}
            }
        }
    }

    Ok(())
}
