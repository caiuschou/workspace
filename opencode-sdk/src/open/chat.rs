//! Chat flow: connect_and_maybe_chat, run_send_and_wait, maybe_send_chat.

use crate::client::Client;
use crate::event;
use crate::session::{CreateSessionRequest, MessageListItem, Part, SendMessageRequest, Session};
use std::path::Path;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tracing::info;

use super::logging;
use super::options::{OpenOptions, ServerHandle};

/// How to handle streaming text during `run_send_and_wait`.
#[derive(Clone)]
pub(crate) enum StreamMode {
    /// Stream text to stdout in real-time.
    StreamToStdout,
    /// Silently wait (no output).
    Silent,
}

/// Fetches the last assistant message with content in one request (no polling).
fn fetch_last_assistant_message(messages: &[MessageListItem]) -> Option<MessageListItem> {
    let last_assistant = messages
        .iter()
        .rev()
        .find(|m| m.info.role.eq_ignore_ascii_case("assistant"))?;
    let has_content =
        !last_assistant.text_content().is_empty() || !last_assistant.parts.is_empty();
    if has_content {
        Some(last_assistant.clone())
    } else {
        None
    }
}

/// Sends message, optionally subscribes to SSE and waits for completion, then returns last assistant message.
async fn run_send_and_wait(
    client: &Client,
    directory: Option<&Path>,
    session_id: &str,
    content: &str,
    wait_for_response_ms: u64,
    mode: StreamMode,
) -> Result<Option<MessageListItem>, crate::Error> {
    client
        .session_send_message_async(
            session_id,
            directory,
            SendMessageRequest::from_parts(vec![Part::text(content)]),
        )
        .await?;
    info!("message sent");

    if wait_for_response_ms == 0 {
        return Ok(None);
    }

    info!(
        timeout_ms = wait_for_response_ms,
        "step: wait for assistant response (SSE completion)"
    );
    let (tx, rx) = oneshot::channel::<()>();
    let client_clone = client.clone();
    let dir = directory.map(|p| p.to_path_buf());
    let session_id_clone = session_id.to_string();
    let mode_clone = mode.clone();

    let event_handle = tokio::spawn(async move {
        let dir_ref = dir.as_deref();
        match mode_clone {
            StreamMode::StreamToStdout => {
                // Each `text` is the latest incremental chunk for this event, not full reply so far.
                let _ = event::subscribe_and_stream_until_done(
                    &client_clone,
                    dir_ref,
                    &session_id_clone,
                    |text| {
                        info!("assistant stream: {}", text);
                        print!("{}", text);
                        let _ = std::io::Write::flush(&mut std::io::stdout());
                    },
                )
                .await;
            }
            StreamMode::Silent => {
                let _ = event::subscribe_and_stream_until_done(
                    &client_clone,
                    dir_ref,
                    &session_id_clone,
                    |_| {},
                )
                .await;
            }
        }
        let _ = tx.send(());
    });

    let timeout_duration = Duration::from_millis(wait_for_response_ms);
    match timeout(timeout_duration, rx).await {
        Ok(Ok(())) | Ok(Err(_)) => {
            event_handle.abort();
            let messages = client.session_list_messages(session_id, directory).await?;
            info!(
                count = messages.len(),
                "received message list (after SSE completion)"
            );
            Ok(fetch_last_assistant_message(&messages))
        }
        Err(_) => {
            event_handle.abort();
            info!(
                timeout_ms = wait_for_response_ms,
                "wait for SSE completion timeout"
            );
            Err(crate::Error::WaitResponseTimeout {
                timeout_ms: wait_for_response_ms,
            })
        }
    }
}

/// Creates session and sends message when chat_content is provided.
async fn maybe_send_chat(
    client: &Client,
    directory: Option<&Path>,
    chat_content: &Option<String>,
    wait_for_response_ms: u64,
    stream_output: bool,
) -> Result<(Option<Session>, Option<MessageListItem>), crate::Error> {
    let content = match chat_content {
        Some(s) if !s.is_empty() => s,
        _ => {
            tracing::debug!("no chat_content, skipping session creation");
            return Ok((None, None));
        }
    };

    info!("step: create session");
    let session = client
        .session_create(directory, CreateSessionRequest::default())
        .await?;
    info!(session_id = %session.id, "session created");

    info!("step: send message (prompt_async)");
    let session_id = session.id.clone();

    let mode = if stream_output {
        StreamMode::StreamToStdout
    } else {
        StreamMode::Silent
    };
    let assistant_reply =
        run_send_and_wait(client, directory, &session_id, content, wait_for_response_ms, mode)
            .await?;

    if let Some(ref reply) = assistant_reply {
        logging::log_assistant_reply(reply);
    }
    Ok((Some(session), assistant_reply))
}

/// Connects to the server and optionally creates a session and sends chat.
///
/// Returns `(Client, None, Option<Session>, Option<MessageListItem>)`.
/// Caller adds `server: Some(handle)` when they spawned the server.
pub(crate) async fn connect_and_maybe_chat(
    base_url: &str,
    working_dir: Option<&Path>,
    options: &OpenOptions,
) -> Result<
    (
        Client,
        Option<ServerHandle>,
        Option<Session>,
        Option<MessageListItem>,
    ),
    crate::Error,
> {
    let client = Client::builder(base_url).try_build()?;
    let (session, assistant_reply) = maybe_send_chat(
        &client,
        working_dir,
        &options.chat_content,
        options.wait_for_response_ms,
        options.stream_output,
    )
    .await?;
    Ok((client, None, session, assistant_reply))
}
