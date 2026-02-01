//! SSE connection: connect_sse and SseEvent.

use crate::client::Client;
use crate::request::RequestBuilderExt;
use crate::Error;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use std::path::Path;

use super::SSE_STREAM_TIMEOUT_SECS;

/// Single SSE event with raw data (JSON string). Used by `connect_sse` stream.
#[derive(Debug)]
pub struct SseEvent {
    /// Raw event data (typically JSON).
    pub data: String,
}

/// Connects to GET /event (instance-level SSE) and returns a stream of parsed events.
///
/// Each item is `Ok(SseEvent)` with raw data or `Err(Error)` on stream error.
/// Consumers parse `ev.data` as JSON and handle completion/text deltas.
pub async fn connect_sse(
    client: &Client,
    directory: Option<&Path>,
) -> Result<
    impl futures::Stream<Item = Result<SseEvent, Error>> + Send + Unpin,
    Error,
> {
    let url = format!("{}/event", client.base_url());
    let req = client.http().get(&url).with_directory(directory);
    let response = req
        .header("Accept", "text/event-stream")
        .timeout(std::time::Duration::from_secs(SSE_STREAM_TIMEOUT_SECS))
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(Error::Http(response.error_for_status().unwrap_err()));
    }
    let stream = response.bytes_stream().eventsource();
    let stream = stream.map(|result| match result {
        Ok(ev) => Ok(SseEvent { data: ev.data }),
        Err(e) => Err(Error::EventStream(e.to_string())),
    });
    Ok(stream)
}
