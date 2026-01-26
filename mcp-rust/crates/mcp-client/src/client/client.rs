use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Sender, channel};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use mcp_core::stdio::Transport;
use mcp_core::{
    protocol::Protocol,
    stdio::JsonRpcMessage,
    types::{
        CreateMessageRequestParams, ElicitRequestFormParams, ElicitRequestUrlParams,
        ElicitationMode, ErrorCode, ErrorObject, ListRootsResult, MessageId, NotificationMessage,
        RequestMessage, ResultMessage,
    },
};

use crate::client::{
    BoxedFormElicitationHandler, BoxedSamplingHandler, BoxedUrlElicitationHandler,
    ClientCapabilities, ClientError, ClientOptions, Implementation, InitializeResult,
    JsonSchemaValidator, ListChangedHandlers, ListChangedKind, NoopJsonSchemaValidator,
    PromptListResult, RequestStream, ResourceListResult, ResponseMessage, ServerCapabilities,
    TaskGetResult, TaskInfo, TaskListResult, TaskResult, ToolCache, ToolCallResult, ToolListResult,
};

/// Minimal client that wires a `Transport` and `Protocol` together.
pub struct Client<T>
where
    T: Transport<Message = JsonRpcMessage>,
{
    #[allow(dead_code)]
    protocol: Protocol,
    transport: T,
    options: ClientOptions,
    capabilities: ClientCapabilities,
    #[allow(dead_code)]
    json_schema_validator: Arc<dyn JsonSchemaValidator>,
    pending_list_changed: Option<ListChangedHandlers>,
    list_changed_handlers: ListChangedHandlers,
    list_changed_due: HashMap<ListChangedKind, Instant>,
    list_changed_pending: HashMap<MessageId, ListChangedKind>,
    tool_cache: ToolCache,
    roots: Vec<mcp_core::types::Root>,
    server_capabilities: Option<ServerCapabilities>,
    server_info: Option<Implementation>,
    instructions: Option<String>,
    pending_initialize_id: Option<MessageId>,
    pending_requests: HashMap<MessageId, String>,
    pending_tool_calls: HashMap<MessageId, String>,
    pending_streams: HashMap<MessageId, Sender<ResponseMessage>>,
    next_id: i64,
    connected: bool,
    // Sampling/Elicitation handlers
    sampling_handler: Option<BoxedSamplingHandler>,
    form_elicitation_handler: Option<BoxedFormElicitationHandler>,
    url_elicitation_handler: Option<BoxedUrlElicitationHandler>,
}

impl<T> Client<T>
where
    T: Transport<Message = JsonRpcMessage>,
{
    /// Create a new client instance with the provided transport and options.
    pub fn new(transport: T, options: ClientOptions) -> Self {
        let mut capabilities = options.capabilities.clone().unwrap_or_default();
        if capabilities.roots.is_none() && options.roots.is_some() {
            capabilities.roots = Some(crate::client::RootsCapability::default());
        }
        let json_schema_validator = options
            .json_schema_validator
            .clone()
            .unwrap_or_else(|| Arc::new(NoopJsonSchemaValidator::default()));
        let roots = options.roots.clone().unwrap_or_default();

        Self {
            protocol: Protocol::default(),
            transport,
            options,
            capabilities,
            json_schema_validator,
            pending_list_changed: None,
            list_changed_handlers: ListChangedHandlers::default(),
            list_changed_due: HashMap::new(),
            list_changed_pending: HashMap::new(),
            tool_cache: ToolCache::default(),
            roots,
            server_capabilities: None,
            server_info: None,
            instructions: None,
            pending_initialize_id: None,
            pending_requests: HashMap::new(),
            pending_tool_calls: HashMap::new(),
            pending_streams: HashMap::new(),
            next_id: 1,
            connected: false,
            sampling_handler: None,
            form_elicitation_handler: None,
            url_elicitation_handler: None,
        }
    }

    fn handle_incoming_request(
        &mut self,
        request: RequestMessage,
    ) -> Result<(), ClientError<T::Error>> {
        match request.method.as_str() {
            "roots/list" => {
                let result = ListRootsResult {
                    roots: self.roots.clone(),
                };
                let payload = serde_json::to_value(result).map_err(ClientError::Serialization)?;
                let response = ResultMessage::success(request.id.clone(), payload);
                self.transport
                    .send(&JsonRpcMessage::Result(response))
                    .map_err(ClientError::Transport)?;
            }
            "sampling/createMessage" => {
                return self.handle_sampling_request(request);
            }
            "elicitation/create" => {
                return self.handle_elicitation_request(request);
            }
            _ => {
                let error = ErrorObject::new(
                    ErrorCode::MethodNotFound as i32,
                    format!("unknown client method: {}", request.method),
                    None,
                );
                let response = ResultMessage::failure(request.id.clone(), error);
                self.transport
                    .send(&JsonRpcMessage::Result(response))
                    .map_err(ClientError::Transport)?;
            }
        }
        Ok(())
    }

    fn handle_sampling_request(
        &mut self,
        request: RequestMessage,
    ) -> Result<(), ClientError<T::Error>> {
        // Check if sampling is supported
        if self.capabilities.sampling.is_none() {
            let error = ErrorObject::new(
                ErrorCode::InvalidRequest as i32,
                "client does not support sampling capability",
                None,
            );
            let response = ResultMessage::failure(request.id.clone(), error);
            self.transport
                .send(&JsonRpcMessage::Result(response))
                .map_err(ClientError::Transport)?;
            return Ok(());
        }

        // Check if handler is registered
        let handler = match &self.sampling_handler {
            Some(h) => h.clone(),
            None => {
                let error = ErrorObject::new(
                    ErrorCode::InternalError as i32,
                    "no sampling handler registered",
                    None,
                );
                let response = ResultMessage::failure(request.id.clone(), error);
                self.transport
                    .send(&JsonRpcMessage::Result(response))
                    .map_err(ClientError::Transport)?;
                return Ok(());
            }
        };

        // Parse the request params
        let params: CreateMessageRequestParams =
            serde_json::from_value(request.params.clone()).map_err(|e| {
                ClientError::Serialization(e)
            })?;

        // Execute handler synchronously
        let result = handler.handle(params);

        let response = match result {
            Ok(result) => {
                let payload = serde_json::to_value(result).map_err(ClientError::Serialization)?;
                ResultMessage::success(request.id.clone(), payload)
            }
            Err(err) => {
                let error = ErrorObject::new(
                    ErrorCode::InternalError as i32,
                    err.0,
                    None,
                );
                ResultMessage::failure(request.id.clone(), error)
            }
        };

        self.transport
            .send(&JsonRpcMessage::Result(response))
            .map_err(ClientError::Transport)?;
        Ok(())
    }

    fn handle_elicitation_request(
        &mut self,
        request: RequestMessage,
    ) -> Result<(), ClientError<T::Error>> {
        // Check if elicitation is supported
        if self.capabilities.elicitation.is_none() {
            let error = ErrorObject::new(
                ErrorCode::InvalidRequest as i32,
                "client does not support elicitation capability",
                None,
            );
            let response = ResultMessage::failure(request.id.clone(), error);
            self.transport
                .send(&JsonRpcMessage::Result(response))
                .map_err(ClientError::Transport)?;
            return Ok(());
        }

        // Determine the mode from the request
        let mode = request
            .params
            .get("mode")
            .and_then(|v| v.as_str())
            .map(|s| {
                if s == "url" {
                    ElicitationMode::Url
                } else {
                    ElicitationMode::Form
                }
            })
            .unwrap_or(ElicitationMode::Form);

        match mode {
            ElicitationMode::Form => {
                let handler = match &self.form_elicitation_handler {
                    Some(h) => h.clone(),
                    None => {
                        let error = ErrorObject::new(
                            ErrorCode::InternalError as i32,
                            "no form elicitation handler registered",
                            None,
                        );
                        let response = ResultMessage::failure(request.id.clone(), error);
                        self.transport
                            .send(&JsonRpcMessage::Result(response))
                            .map_err(ClientError::Transport)?;
                        return Ok(());
                    }
                };

                let params: ElicitRequestFormParams =
                    serde_json::from_value(request.params.clone()).map_err(ClientError::Serialization)?;

                let result = handler.handle(params);

                let response = match result {
                    Ok(result) => {
                        let payload = serde_json::to_value(result).map_err(ClientError::Serialization)?;
                        ResultMessage::success(request.id.clone(), payload)
                    }
                    Err(err) => {
                        let error = ErrorObject::new(
                            ErrorCode::InternalError as i32,
                            err.0,
                            None,
                        );
                        ResultMessage::failure(request.id.clone(), error)
                    }
                };

                self.transport
                    .send(&JsonRpcMessage::Result(response))
                    .map_err(ClientError::Transport)?;
            }
            ElicitationMode::Url => {
                let handler = match &self.url_elicitation_handler {
                    Some(h) => h.clone(),
                    None => {
                        let error = ErrorObject::new(
                            ErrorCode::InternalError as i32,
                            "no URL elicitation handler registered",
                            None,
                        );
                        let response = ResultMessage::failure(request.id.clone(), error);
                        self.transport
                            .send(&JsonRpcMessage::Result(response))
                            .map_err(ClientError::Transport)?;
                        return Ok(());
                    }
                };

                let params: ElicitRequestUrlParams =
                    serde_json::from_value(request.params.clone()).map_err(ClientError::Serialization)?;

                let result = handler.handle(params);

                let response = match result {
                    Ok(result) => {
                        let payload = serde_json::to_value(result).map_err(ClientError::Serialization)?;
                        ResultMessage::success(request.id.clone(), payload)
                    }
                    Err(err) => {
                        let error = ErrorObject::new(
                            ErrorCode::InternalError as i32,
                            err.0,
                            None,
                        );
                        ResultMessage::failure(request.id.clone(), error)
                    }
                };

                self.transport
                    .send(&JsonRpcMessage::Result(response))
                    .map_err(ClientError::Transport)?;
            }
        }

        Ok(())
    }

    /// Register client capabilities before connecting.
    pub fn register_capabilities(
        &mut self,
        capabilities: ClientCapabilities,
    ) -> Result<(), ClientError<T::Error>> {
        if self.connected {
            return Err(ClientError::Capability(
                "cannot register capabilities after connect".to_string(),
            ));
        }
        self.capabilities = self.capabilities.merge(&capabilities);
        Ok(())
    }

    /// Set the handler for sampling/createMessage requests from the server.
    /// This should be called before connecting if sampling capability is declared.
    pub fn set_sampling_handler<H>(&mut self, handler: H)
    where
        H: crate::client::SamplingHandler,
    {
        self.sampling_handler = Some(Arc::new(handler));
    }

    /// Set the handler for form-based elicitation/create requests from the server.
    /// This should be called before connecting if form elicitation capability is declared.
    pub fn set_form_elicitation_handler<H>(&mut self, handler: H)
    where
        H: crate::client::FormElicitationHandler,
    {
        self.form_elicitation_handler = Some(Arc::new(handler));
    }

    /// Set the handler for URL-based elicitation/create requests from the server.
    /// This should be called before connecting if URL elicitation capability is declared.
    pub fn set_url_elicitation_handler<H>(&mut self, handler: H)
    where
        H: crate::client::UrlElicitationHandler,
    {
        self.url_elicitation_handler = Some(Arc::new(handler));
    }

    /// Connect to the transport and send an initialize request.
    pub fn connect(&mut self) -> Result<(), ClientError<T::Error>> {
        if self.connected {
            return Ok(());
        }
        self.transport.start()?;
        self.connected = true;

        let id = self.next_message_id();
        self.pending_initialize_id = Some(id.clone());

        let request = RequestMessage::new(
            id,
            "initialize",
            json!({
                "protocolVersion": self.options.protocol_version,
                "capabilities": self.capabilities,
                "clientInfo": self.options.client_info,
            }),
        );

        self.transport.send(&JsonRpcMessage::Request(request))?;
        self.pending_list_changed = self.options.list_changed.clone();
        Ok(())
    }

    /// Handle a JSON-RPC message from the transport.
    pub fn handle_message(&mut self, message: JsonRpcMessage) -> Result<(), ClientError<T::Error>> {
        match message {
            JsonRpcMessage::Result(result) => {
                if let Some(pending_id) = &self.pending_initialize_id {
                    if &result.id == pending_id {
                        return self.handle_initialize_result(result);
                    }
                }
                if let Some(sender) = self.pending_streams.remove(&result.id) {
                    if let Some(error) = result.error {
                        let _ = sender.send(ResponseMessage::Error(error.message));
                    } else {
                        let value = result.result.unwrap_or(Value::Null);
                        let _ = sender.send(ResponseMessage::Result(value));
                    }
                    return Ok(());
                }
                if let Some(method) = self.pending_requests.remove(&result.id) {
                    let id = result.id.clone();
                    let response = self.handle_request_result(id, method, result);
                    self.flush_debounced_list_changed();
                    return response;
                }
                self.flush_debounced_list_changed();
                Ok(())
            }
            JsonRpcMessage::Notification(notification) => {
                if self.try_send_task_notification(&notification) {
                    self.flush_debounced_list_changed();
                    return Ok(());
                }
                self.handle_notification(notification);
                self.flush_debounced_list_changed();
                Ok(())
            }
            JsonRpcMessage::Request(request) => self.handle_incoming_request(request),
        }
    }

    /// Retrieve the server capabilities after initialization.
    pub fn get_server_capabilities(&self) -> Option<&ServerCapabilities> {
        self.server_capabilities.as_ref()
    }

    /// Retrieve the server implementation after initialization.
    pub fn get_server_version(&self) -> Option<&Implementation> {
        self.server_info.as_ref()
    }

    /// Retrieve server instructions after initialization.
    pub fn get_instructions(&self) -> Option<&str> {
        self.instructions.as_deref()
    }

    /// Send a plain request message through the transport.
    pub fn send_request(
        &mut self,
        method: impl Into<String>,
        params: Value,
    ) -> Result<MessageId, ClientError<T::Error>> {
        let method = method.into();
        self.assert_capability_for_method(&method)?;

        let request = RequestMessage::new(self.next_message_id(), method, params);
        let id = request.id.clone();
        self.pending_requests
            .insert(id.clone(), request.method.clone());
        self.transport.send(&JsonRpcMessage::Request(request))?;
        Ok(id)
    }

    /// Send a notification through the transport.
    pub fn send_notification(
        &mut self,
        method: impl Into<String>,
        params: Option<Value>,
    ) -> Result<(), ClientError<T::Error>> {
        let method = method.into();
        self.assert_notification_capability(&method)?;
        let notification = NotificationMessage::new(method, params);
        self.transport
            .send(&JsonRpcMessage::Notification(notification))?;
        Ok(())
    }

    /// Shutdown the transport.
    pub fn close(&mut self) -> Result<(), ClientError<T::Error>> {
        self.transport.close()?;
        self.connected = false;
        Ok(())
    }

    /// Send a tools/list request.
    pub fn list_tools(&mut self) -> Result<MessageId, ClientError<T::Error>> {
        self.send_request("tools/list", json!({}))
    }

    /// Send a prompts/list request.
    pub fn list_prompts(&mut self) -> Result<MessageId, ClientError<T::Error>> {
        self.send_request("prompts/list", json!({}))
    }

    /// Send a resources/list request.
    pub fn list_resources(&mut self) -> Result<MessageId, ClientError<T::Error>> {
        self.send_request("resources/list", json!({}))
    }

    /// Send a tools/call request.
    pub fn call_tool(
        &mut self,
        name: impl Into<String>,
        arguments: Value,
    ) -> Result<MessageId, ClientError<T::Error>> {
        let name = name.into();
        if self.tool_cache.is_task_required(&name) {
            return Err(ClientError::Capability(format!(
                "tool \"{name}\" requires task-based execution"
            )));
        }
        let id = self.send_request(
            "tools/call",
            json!({
                "name": name,
                "arguments": arguments
            }),
        )?;
        self.pending_tool_calls.insert(id.clone(), name);
        Ok(id)
    }

    /// Call a tool using a streaming request interface.
    pub fn call_tool_stream(
        &mut self,
        name: impl Into<String>,
        arguments: Value,
    ) -> Result<RequestStream, ClientError<T::Error>> {
        let name = name.into();
        let params = json!({ "name": name, "arguments": arguments, "task": {} });
        self.request_stream("tools/call", params)
    }

    /// Request task status by task id.
    pub fn get_task(
        &mut self,
        task_id: impl Into<String>,
    ) -> Result<MessageId, ClientError<T::Error>> {
        self.send_request("tasks/get", json!({ "taskId": task_id.into() }))
    }

    /// Request task result by task id.
    pub fn get_task_result(
        &mut self,
        task_id: impl Into<String>,
    ) -> Result<MessageId, ClientError<T::Error>> {
        self.send_request("tasks/result", json!({ "taskId": task_id.into() }))
    }

    /// List tasks with optional cursor.
    pub fn list_tasks(
        &mut self,
        cursor: Option<String>,
    ) -> Result<MessageId, ClientError<T::Error>> {
        let params = cursor
            .map(|cursor| json!({ "cursor": cursor }))
            .unwrap_or_else(|| json!({}));
        self.send_request("tasks/list", params)
    }

    /// Cancel a running task.
    pub fn cancel_task(
        &mut self,
        task_id: impl Into<String>,
    ) -> Result<MessageId, ClientError<T::Error>> {
        self.send_request("tasks/cancel", json!({ "taskId": task_id.into() }))
    }

    /// Send a request and receive responses through a stream interface.
    pub fn request_stream(
        &mut self,
        method: impl Into<String>,
        params: Value,
    ) -> Result<RequestStream, ClientError<T::Error>> {
        let method = method.into();
        self.assert_capability_for_method(&method)?;

        let request = RequestMessage::new(self.next_message_id(), method, params);
        let id = request.id.clone();

        let (sender, receiver) = channel();
        self.pending_streams.insert(id, sender);
        self.transport.send(&JsonRpcMessage::Request(request))?;
        Ok(RequestStream::new(receiver))
    }

    #[allow(dead_code)]
    pub(crate) fn is_tool_task_required(&self, tool_name: &str) -> bool {
        self.tool_cache.is_task_required(tool_name)
    }

    fn next_message_id(&mut self) -> MessageId {
        let id = self.next_id;
        self.next_id += 1;
        MessageId::Number(id)
    }

    fn handle_initialize_result(
        &mut self,
        result: mcp_core::types::ResultMessage,
    ) -> Result<(), ClientError<T::Error>> {
        if let Some(error) = result.error {
            return Err(ClientError::Initialization(error.message));
        }

        let payload = result.result.ok_or_else(|| {
            ClientError::Initialization("initialize returned empty result".to_string())
        })?;

        let init: InitializeResult =
            serde_json::from_value(payload).map_err(ClientError::Serialization)?;
        if init.protocol_version != self.options.protocol_version {
            return Err(ClientError::Initialization(format!(
                "unsupported protocol version: {}",
                init.protocol_version
            )));
        }

        self.server_capabilities = Some(init.capabilities);
        self.server_info = Some(init.server_info);
        self.instructions = init.instructions;
        self.pending_initialize_id = None;

        if let Some(list_changed) = self.pending_list_changed.take() {
            self.list_changed_handlers = list_changed;
        }

        self.send_notification("notifications/initialized", None)?;
        Ok(())
    }

    fn handle_request_result(
        &mut self,
        id: MessageId,
        method: String,
        result: mcp_core::types::ResultMessage,
    ) -> Result<(), ClientError<T::Error>> {
        if let Some(error) = result.error {
            return Err(ClientError::Initialization(error.message));
        }

        let payload = result.result.unwrap_or(Value::Null);
        match method.as_str() {
            "tools/list" => self.handle_tools_list(id, payload),
            "tools/call" => self.handle_tool_call(id, payload),
            "prompts/list" => self.handle_prompts_list(id, payload),
            "resources/list" => self.handle_resources_list(id, payload),
            "tasks/list" => self.handle_tasks_list(id, payload),
            "tasks/get" => self.handle_task_get(id, payload),
            "tasks/result" => self.handle_task_result(id, payload),
            _ => Ok(()),
        }
    }

    fn handle_tools_list(
        &mut self,
        id: MessageId,
        payload: Value,
    ) -> Result<(), ClientError<T::Error>> {
        let list: ToolListResult =
            serde_json::from_value(payload).map_err(ClientError::Serialization)?;
        self.tool_cache.update(&list.tools);

        self.handle_list_changed_items(
            id,
            ListChangedKind::Tools,
            serde_json::json!({ "tools": list.tools }),
        )?;
        Ok(())
    }

    fn handle_tool_call(
        &mut self,
        id: MessageId,
        payload: Value,
    ) -> Result<(), ClientError<T::Error>> {
        let result: ToolCallResult =
            serde_json::from_value(payload).map_err(ClientError::Serialization)?;
        if result.is_error.unwrap_or(false) {
            return Ok(());
        }
        let Some(tool_name) = self.pending_tool_calls.remove(&id) else {
            return Ok(());
        };

        if let Some(schema) = self.tool_cache.output_schema(&tool_name) {
            if let Some(structured) = result.structured_content.as_ref() {
                self.json_schema_validator
                    .validate(schema, structured)
                    .map_err(ClientError::Validation)?;
            } else {
                return Err(ClientError::Validation(
                    "tool has output schema but returned no structured content".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn handle_prompts_list(
        &mut self,
        id: MessageId,
        payload: Value,
    ) -> Result<(), ClientError<T::Error>> {
        let list: PromptListResult =
            serde_json::from_value(payload).map_err(ClientError::Serialization)?;
        let prompts_value =
            serde_json::to_value(list.prompts).map_err(ClientError::Serialization)?;
        self.handle_list_changed_items(
            id,
            ListChangedKind::Prompts,
            json!({ "prompts": prompts_value }),
        )?;
        Ok(())
    }

    fn handle_resources_list(
        &mut self,
        id: MessageId,
        payload: Value,
    ) -> Result<(), ClientError<T::Error>> {
        let list: ResourceListResult =
            serde_json::from_value(payload).map_err(ClientError::Serialization)?;
        let resources_value =
            serde_json::to_value(list.resources).map_err(ClientError::Serialization)?;
        self.handle_list_changed_items(
            id,
            ListChangedKind::Resources,
            json!({ "resources": resources_value }),
        )?;
        Ok(())
    }

    fn handle_tasks_list(
        &mut self,
        _id: MessageId,
        payload: Value,
    ) -> Result<(), ClientError<T::Error>> {
        let _list: TaskListResult =
            serde_json::from_value(payload).map_err(ClientError::Serialization)?;
        Ok(())
    }

    fn handle_task_get(
        &mut self,
        _id: MessageId,
        payload: Value,
    ) -> Result<(), ClientError<T::Error>> {
        let _task: TaskGetResult =
            serde_json::from_value(payload).map_err(ClientError::Serialization)?;
        Ok(())
    }

    fn handle_task_result(
        &mut self,
        _id: MessageId,
        payload: Value,
    ) -> Result<(), ClientError<T::Error>> {
        let _result: TaskResult =
            serde_json::from_value(payload).map_err(ClientError::Serialization)?;
        Ok(())
    }

    fn handle_notification(&mut self, notification: NotificationMessage) {
        let kind = match notification.method.as_str() {
            "notifications/tools/list_changed" => Some(ListChangedKind::Tools),
            "notifications/prompts/list_changed" => Some(ListChangedKind::Prompts),
            "notifications/resources/list_changed" => Some(ListChangedKind::Resources),
            _ => None,
        };

        if let Some(kind) = kind {
            self.on_list_changed(kind);
        }
    }

    fn try_send_task_notification(&mut self, notification: &NotificationMessage) -> bool {
        let method = notification.method.as_str();
        let kind = match method {
            "notifications/tasks/created" => Some(true),
            "notifications/tasks/status" => Some(false),
            _ => None,
        };

        let Some(is_created) = kind else {
            return false;
        };

        let Some(params) = notification.params.clone() else {
            return false;
        };

        let task_info: TaskInfo = match serde_json::from_value(params) {
            Ok(value) => value,
            Err(_) => return false,
        };

        let message = if is_created {
            ResponseMessage::TaskCreated(task_info)
        } else {
            ResponseMessage::TaskStatus(task_info)
        };

        for sender in self.pending_streams.values() {
            let _ = sender.send(message.clone());
        }

        true
    }

    fn on_list_changed(&mut self, kind: ListChangedKind) {
        if !self.is_list_changed_supported(kind) {
            return;
        }

        let handler = match kind {
            ListChangedKind::Tools => self.list_changed_handlers.tools.clone(),
            ListChangedKind::Prompts => self.list_changed_handlers.prompts.clone(),
            ListChangedKind::Resources => self.list_changed_handlers.resources.clone(),
        };

        let Some(options) = handler else {
            return;
        };
        if let Some(debounce_ms) = options.debounce_ms {
            self.list_changed_due
                .insert(kind, Instant::now() + Duration::from_millis(debounce_ms));
            return;
        }

        self.trigger_list_changed_refresh(kind, options);
    }

    fn handle_list_changed_items(
        &mut self,
        id: MessageId,
        kind: ListChangedKind,
        payload: Value,
    ) -> Result<(), ClientError<T::Error>> {
        if self.list_changed_pending.remove(&id) != Some(kind) {
            return Ok(());
        }

        let items = match kind {
            ListChangedKind::Tools => payload.get("tools").cloned().unwrap_or(Value::Null),
            ListChangedKind::Prompts => payload.get("prompts").cloned().unwrap_or(Value::Null),
            ListChangedKind::Resources => payload.get("resources").cloned().unwrap_or(Value::Null),
        };

        let list = items.as_array().cloned().unwrap_or_default();
        let handler = match kind {
            ListChangedKind::Tools => self.list_changed_handlers.tools.clone(),
            ListChangedKind::Prompts => self.list_changed_handlers.prompts.clone(),
            ListChangedKind::Resources => self.list_changed_handlers.resources.clone(),
        };

        if let Some(options) = handler {
            (options.on_changed)(Ok(Some(list)));
        }

        Ok(())
    }

    fn flush_debounced_list_changed(&mut self) {
        if self.list_changed_due.is_empty() {
            return;
        }

        let now = Instant::now();
        let due_kinds: Vec<ListChangedKind> = self
            .list_changed_due
            .iter()
            .filter_map(|(kind, due)| if *due <= now { Some(*kind) } else { None })
            .collect();

        for kind in due_kinds {
            self.list_changed_due.remove(&kind);
            let handler = match kind {
                ListChangedKind::Tools => self.list_changed_handlers.tools.clone(),
                ListChangedKind::Prompts => self.list_changed_handlers.prompts.clone(),
                ListChangedKind::Resources => self.list_changed_handlers.resources.clone(),
            };

            if let Some(options) = handler {
                self.trigger_list_changed_refresh(kind, options);
            }
        }
    }

    fn trigger_list_changed_refresh(
        &mut self,
        kind: ListChangedKind,
        options: crate::client::ListChangedOptions<Value>,
    ) {
        if !options.auto_refresh {
            (options.on_changed)(Ok(None));
            return;
        }

        let request_id = match kind {
            ListChangedKind::Tools => self.list_tools(),
            ListChangedKind::Prompts => self.list_prompts(),
            ListChangedKind::Resources => self.list_resources(),
        };

        match request_id {
            Ok(id) => {
                self.list_changed_pending.insert(id, kind);
            }
            Err(_) => {
                (options.on_changed)(Err("list refresh request failed".to_string()));
            }
        }
    }

    fn is_list_changed_supported(&self, kind: ListChangedKind) -> bool {
        let Some(server) = self.server_capabilities.as_ref() else {
            return false;
        };
        match kind {
            ListChangedKind::Tools => server
                .tools
                .as_ref()
                .and_then(|caps| caps.list_changed)
                .unwrap_or(false),
            ListChangedKind::Prompts => server
                .prompts
                .as_ref()
                .and_then(|caps| caps.list_changed)
                .unwrap_or(false),
            ListChangedKind::Resources => server
                .resources
                .as_ref()
                .and_then(|caps| caps.list_changed)
                .unwrap_or(false),
        }
    }

    fn assert_capability_for_method(&self, method: &str) -> Result<(), ClientError<T::Error>> {
        let server = self.server_capabilities.as_ref().ok_or_else(|| {
            ClientError::Capability("server capabilities unavailable".to_string())
        })?;

        match method {
            "logging/setLevel" => {
                if server.logging.is_none() {
                    return Err(ClientError::Capability(format!(
                        "server does not support logging (required for {method})",
                    )));
                }
            }
            "prompts/get" | "prompts/list" => {
                if server.prompts.is_none() {
                    return Err(ClientError::Capability(format!(
                        "server does not support prompts (required for {method})",
                    )));
                }
            }
            "resources/list"
            | "resources/templates/list"
            | "resources/read"
            | "resources/subscribe"
            | "resources/unsubscribe" => {
                let resources = server.resources.as_ref().ok_or_else(|| {
                    ClientError::Capability(format!(
                        "server does not support resources (required for {method})",
                    ))
                })?;
                if method == "resources/subscribe" && resources.subscribe != Some(true) {
                    return Err(ClientError::Capability(format!(
                        "server does not support resource subscriptions (required for {method})",
                    )));
                }
            }
            "tools/call" | "tools/list" => {
                if server.tools.is_none() {
                    return Err(ClientError::Capability(format!(
                        "server does not support tools (required for {method})",
                    )));
                }
            }
            "completion/complete" => {
                if server.completions.is_none() {
                    return Err(ClientError::Capability(format!(
                        "server does not support completions (required for {method})",
                    )));
                }
            }
            "initialize" | "ping" => {}
            "tasks/get" | "tasks/list" | "tasks/result" | "tasks/cancel" => {
                if server.tasks.is_none() {
                    return Err(ClientError::Capability(format!(
                        "server does not support tasks (required for {method})",
                    )));
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn assert_notification_capability(&self, method: &str) -> Result<(), ClientError<T::Error>> {
        match method {
            "notifications/roots/list_changed" => {
                let roots = self
                    .capabilities
                    .roots
                    .as_ref()
                    .ok_or_else(|| ClientError::Capability(format!(
                        "client does not support roots list changed notifications (required for {method})",
                    )))?;
                if roots.list_changed != Some(true) {
                    return Err(ClientError::Capability(format!(
                        "client does not support roots list changed notifications (required for {method})",
                    )));
                }
            }
            "notifications/initialized" => {}
            "notifications/cancelled" => {}
            "notifications/progress" => {}
            _ => {}
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn assert_request_handler_capability(&self, method: &str) -> Result<(), ClientError<T::Error>> {
        match method {
            "sampling/createMessage" => {
                if self.capabilities.sampling.is_none() {
                    return Err(ClientError::Capability(format!(
                        "client does not support sampling capability (required for {method})",
                    )));
                }
            }
            "elicitation/create" => {
                if self.capabilities.elicitation.is_none() {
                    return Err(ClientError::Capability(format!(
                        "client does not support elicitation capability (required for {method})",
                    )));
                }
            }
            "roots/list" => {
                if self.capabilities.roots.is_none() {
                    return Err(ClientError::Capability(format!(
                        "client does not support roots capability (required for {method})",
                    )));
                }
            }
            "tasks/get" | "tasks/list" | "tasks/result" | "tasks/cancel" => {
                if self.capabilities.tasks.is_none() {
                    return Err(ClientError::Capability(format!(
                        "client does not support tasks capability (required for {method})",
                    )));
                }
            }
            "ping" => {}
            _ => {}
        }
        Ok(())
    }
}
