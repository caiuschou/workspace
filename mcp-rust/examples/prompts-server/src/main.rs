//! Example: MCP Prompts Server
//!
//! This example demonstrates how to register and use MCP prompts.
//! Prompts are reusable templates that generate messages for LLM interactions.
//!
//! Features:
//! - Prompt registration with arguments
//! - Prompt list and get operations
//! - Dynamic message generation based on arguments
//!
//! Run with: cargo run -p mcp-prompts-server
//!
//! Test with curl:
//! ```bash
//! # Initialize
//! curl -X POST http://localhost:8080/mcp \
//!      -H "Content-Type: application/json" \
//!      -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}'
//!
//! # List prompts
//! curl -X POST http://localhost:8080/mcp \
//!      -H "Content-Type: application/json" \
//!      -d '{"jsonrpc":"2.0","id":2,"method":"prompts/list","params":{}}'
//!
//! # Get a prompt
//! curl -X POST http://localhost:8080/mcp \
//!      -H "Content-Type: application/json" \
//!      -d '{"jsonrpc":"2.0","id":3,"method":"prompts/get","params":{"name":"code_review","arguments":{"language":"rust","code":"fn main() { println!(\"Hello\"); }"}}}'
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use mcp_core::protocol::RequestContext;
use mcp_core::types::{
    BaseMetadata, ContentBlock, GetPromptResult, Icons, Implementation, Prompt, PromptArgument,
    PromptMessage, Role, ServerCapabilities, TextContent,
};
use mcp_server::{
    AxumHandlerConfig, AxumHandlerState, McpServer, ServerError, ServerOptions, create_router,
};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Create server info
    let server_info = Implementation {
        base: BaseMetadata {
            name: "mcp-prompts-server".to_string(),
            title: Some("MCP Prompts Server Example".to_string()),
        },
        icons: Icons::default(),
        version: "0.1.0".to_string(),
        website_url: None,
        description: Some("Example MCP server demonstrating prompts functionality".to_string()),
    };

    // Configure server capabilities with prompts support
    let mut server_options = ServerOptions::default();
    server_options.capabilities = Some(ServerCapabilities {
        prompts: Some(mcp_core::types::PromptCapabilities {
            list_changed: Some(true),
        }),
        ..Default::default()
    });
    server_options.instructions =
        Some("This server provides reusable prompt templates for LLM interactions.".to_string());

    // Create MCP server
    let mut mcp_server = McpServer::new(server_info, server_options);

    // Register example prompts
    register_prompts(&mut mcp_server)?;

    let mcp_server = Arc::new(mcp_server);

    // Configure HTTP handler
    let config = AxumHandlerConfig {
        base_url: Some("http://localhost:8080".to_string()),
        endpoint_path: "/mcp".to_string(),
        keep_alive_interval: Duration::from_secs(30),
        broadcast_capacity: 100,
        enable_cors: true,
        ..Default::default()
    };

    // Create handler state and router
    let state = Arc::new(AxumHandlerState::new(mcp_server, config));
    let app = create_router(state);

    // Start server
    let addr = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("MCP Prompts Server listening on http://{}", addr);
    println!();
    println!("Available prompts:");
    println!("  - code_review: Review code in any language");
    println!("  - explain_concept: Explain a technical concept");
    println!("  - translate: Translate text between languages");
    println!("  - summarize: Summarize text content");
    println!();
    println!("Example requests:");
    println!();
    println!("  # List prompts");
    println!(
        r#"  curl -X POST http://localhost:8080/mcp -H "Content-Type: application/json" -d '{{"jsonrpc":"2.0","id":1,"method":"prompts/list","params":{{}}}}'"#
    );
    println!();
    println!("  # Get code_review prompt");
    println!(
        r#"  curl -X POST http://localhost:8080/mcp -H "Content-Type: application/json" -d '{{"jsonrpc":"2.0","id":2,"method":"prompts/get","params":{{"name":"code_review","arguments":{{"language":"rust","code":"fn main() {{}}"}}}}}}"#
    );
    println!();

    axum::serve(listener, app).await?;

    Ok(())
}

fn register_prompts(server: &mut McpServer) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Code Review Prompt
    server.register_prompt(
        Prompt {
            base: BaseMetadata {
                name: "code_review".to_string(),
                title: Some("Code Review".to_string()),
            },
            icons: Icons::default(),
            description: Some(
                "Review code for best practices, bugs, and improvements".to_string(),
            ),
            arguments: Some(vec![
                PromptArgument {
                    name: "language".to_string(),
                    description: Some("Programming language of the code".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "code".to_string(),
                    description: Some("The code to review".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "focus".to_string(),
                    description: Some(
                        "Optional focus area (security, performance, readability)".to_string(),
                    ),
                    required: Some(false),
                },
            ]),
            meta: None,
        },
        |args: Option<HashMap<String, String>>, _ctx: RequestContext| async move {
            let args = args.unwrap_or_default();
            let language = args.get("language").map(|s| s.as_str()).unwrap_or("unknown");
            let code = args
                .get("code")
                .map(|s| s.as_str())
                .unwrap_or("(no code provided)");
            let focus = args.get("focus").map(|s| s.as_str());

            let focus_instruction = match focus {
                Some("security") => "Focus especially on security vulnerabilities.",
                Some("performance") => "Focus especially on performance optimizations.",
                Some("readability") => "Focus especially on code readability and clarity.",
                _ => "Review all aspects including correctness, style, and best practices.",
            };

            let system_message = format!(
                "You are an expert {} code reviewer. {}",
                language, focus_instruction
            );

            let user_message = format!(
                "Please review the following {} code:\n\n```{}\n{}\n```",
                language, language, code
            );

            Ok::<_, ServerError>(GetPromptResult {
                description: Some(format!("Code review for {} code", language)),
                messages: vec![
                    PromptMessage {
                        role: Role::Assistant,
                        content: ContentBlock::Text(TextContent::new(system_message)),
                    },
                    PromptMessage {
                        role: Role::User,
                        content: ContentBlock::Text(TextContent::new(user_message)),
                    },
                ],
                meta: None,
            })
        },
    )?;

    // 2. Explain Concept Prompt
    server.register_prompt(
        Prompt {
            base: BaseMetadata {
                name: "explain_concept".to_string(),
                title: Some("Explain Concept".to_string()),
            },
            icons: Icons::default(),
            description: Some("Explain a technical concept at a specified level".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "concept".to_string(),
                    description: Some("The concept to explain".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "level".to_string(),
                    description: Some(
                        "Explanation level: beginner, intermediate, or expert".to_string(),
                    ),
                    required: Some(false),
                },
            ]),
            meta: None,
        },
        |args: Option<HashMap<String, String>>, _ctx: RequestContext| async move {
            let args = args.unwrap_or_default();
            let concept = args
                .get("concept")
                .map(|s| s.as_str())
                .unwrap_or("(no concept provided)");
            let level = args
                .get("level")
                .map(|s| s.as_str())
                .unwrap_or("intermediate");

            let level_instruction = match level {
                "beginner" => "Explain in simple terms, avoiding jargon. Use analogies.",
                "expert" => "Provide a detailed technical explanation with advanced concepts.",
                _ => "Provide a balanced explanation suitable for someone with basic knowledge.",
            };

            let message = format!(
                "Please explain the concept of \"{}\".\n\n{}",
                concept, level_instruction
            );

            Ok::<_, ServerError>(GetPromptResult {
                description: Some(format!("Explanation of {} at {} level", concept, level)),
                messages: vec![PromptMessage {
                    role: Role::User,
                    content: ContentBlock::Text(TextContent::new(message)),
                }],
                meta: None,
            })
        },
    )?;

    // 3. Translation Prompt
    server.register_prompt(
        Prompt {
            base: BaseMetadata {
                name: "translate".to_string(),
                title: Some("Translate Text".to_string()),
            },
            icons: Icons::default(),
            description: Some("Translate text between languages".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "text".to_string(),
                    description: Some("The text to translate".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "source_language".to_string(),
                    description: Some("Source language (or 'auto' for detection)".to_string()),
                    required: Some(false),
                },
                PromptArgument {
                    name: "target_language".to_string(),
                    description: Some("Target language".to_string()),
                    required: Some(true),
                },
            ]),
            meta: None,
        },
        |args: Option<HashMap<String, String>>, _ctx: RequestContext| async move {
            let args = args.unwrap_or_default();
            let text = args
                .get("text")
                .map(|s| s.as_str())
                .unwrap_or("(no text provided)");
            let source = args
                .get("source_language")
                .map(|s| s.as_str())
                .unwrap_or("auto");
            let target = args
                .get("target_language")
                .map(|s| s.as_str())
                .unwrap_or("English");

            let message = if source == "auto" {
                format!(
                    "Please translate the following text to {}:\n\n{}",
                    target, text
                )
            } else {
                format!(
                    "Please translate the following text from {} to {}:\n\n{}",
                    source, target, text
                )
            };

            Ok::<_, ServerError>(GetPromptResult {
                description: Some(format!("Translation to {}", target)),
                messages: vec![PromptMessage {
                    role: Role::User,
                    content: ContentBlock::Text(TextContent::new(message)),
                }],
                meta: None,
            })
        },
    )?;

    // 4. Summarize Prompt
    server.register_prompt(
        Prompt {
            base: BaseMetadata {
                name: "summarize".to_string(),
                title: Some("Summarize Text".to_string()),
            },
            icons: Icons::default(),
            description: Some("Summarize text content".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "text".to_string(),
                    description: Some("The text to summarize".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "style".to_string(),
                    description: Some(
                        "Summary style: brief, detailed, or bullet_points".to_string(),
                    ),
                    required: Some(false),
                },
                PromptArgument {
                    name: "max_length".to_string(),
                    description: Some("Maximum length in words (optional)".to_string()),
                    required: Some(false),
                },
            ]),
            meta: None,
        },
        |args: Option<HashMap<String, String>>, _ctx: RequestContext| async move {
            let args = args.unwrap_or_default();
            let text = args
                .get("text")
                .map(|s| s.as_str())
                .unwrap_or("(no text provided)");
            let style = args.get("style").map(|s| s.as_str()).unwrap_or("brief");
            let max_length = args.get("max_length").map(|s| s.as_str());

            let style_instruction = match style {
                "detailed" => "Provide a comprehensive summary covering all main points.",
                "bullet_points" => "Summarize using bullet points for key takeaways.",
                _ => "Provide a concise summary of the main idea.",
            };

            let length_instruction = match max_length {
                Some(len) if !len.is_empty() => format!(" Keep the summary under {} words.", len),
                _ => String::new(),
            };

            let message = format!(
                "Please summarize the following text.\n\n{}{}\n\nText:\n{}",
                style_instruction, length_instruction, text
            );

            Ok::<_, ServerError>(GetPromptResult {
                description: Some(format!("{} summary", style)),
                messages: vec![PromptMessage {
                    role: Role::User,
                    content: ContentBlock::Text(TextContent::new(message)),
                }],
                meta: None,
            })
        },
    )?;

    Ok(())
}
