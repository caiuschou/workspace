use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures::future::BoxFuture;
use schemars::schema::RootSchema;
use serde_json::Value;

use mcp_core::protocol::{
    NotificationContext, NotificationHandler, Protocol, ProtocolError, RequestContext,
    RequestHandler, TaskStore,
};
use mcp_core::schema::JsonSchemaValidator;
use mcp_core::types::{
    CancelTaskRequestParams, CancelTaskResult, CapabilityFlag, ClientCapabilities,
    CreateMessageRequestParams, ElicitRequestFormParams, ElicitRequestUrlParams,
    ElicitationCompleteNotificationParams, ErrorCode, ErrorObject, GetTaskPayloadRequestParams,
    GetTaskRequestParams, GetTaskResult, InitializeRequestParams, InitializeResult, ListTasksResult,
    MessageId, NotificationMessage, PaginatedRequestParams, PaginatedResult, RequestMessage,
    ResultMessage, SUPPORTED_PROTOCOL_VERSIONS, ServerCapabilities, ServerTasksCapability,
    ServerTasksRequestCapabilities, ServerTasksToolCapabilities, SetLevelRequestParams, Task,
    TaskStatusNotificationParams,
};

use crate::server::handlers::{NotificationHandlerFn, RequestHandlerFn};
use crate::server::server_capability_checker::ServerCapabilityChecker;
use crate::server::server_error::ServerError;
use crate::server::server_options::ServerOptions;
use crate::server::server_state::ServerState;

/// Low-level MCP server wrapper around the protocol runtime.
pub struct Server {
    protocol: Protocol,
    state: Arc<Mutex<ServerState>>,
    server_info: mcp_core::types::Implementation,
    on_initialized: Arc<Mutex<Option<Arc<dyn Fn() + Send + Sync>>>>,
    task_store: Option<Arc<dyn TaskStore>>,
    logging_handler_registered: bool,
    task_handlers_registered: bool,
}

impl Server {
    pub fn new(server_info: mcp_core::types::Implementation, options: ServerOptions) -> Self {
        let capabilities = options.capabilities.clone().unwrap_or_default();
        let state = Arc::new(Mutex::new(ServerState::new(
            capabilities.clone(),
            options.instructions.clone(),
        )));

        let mut protocol = Protocol::with_options(
            JsonSchemaValidator::default(),
            options.protocol_options.clone().unwrap_or_default(),
        );

        let task_store = options
            .protocol_options
            .as_ref()
            .and_then(|opts| opts.task_store.clone());

        protocol
            .set_capability_checker(Some(Arc::new(ServerCapabilityChecker::new(state.clone()))));

        let on_initialized = Arc::new(Mutex::new(None));

        let mut server = Self {
            protocol,
            state,
            server_info,
            on_initialized,
            task_store,
            logging_handler_registered: false,
            task_handlers_registered: false,
        };

        server.register_initialize_handlers();
        server.register_logging_handler_if_needed();
        server.register_task_handlers_if_needed();

        server
    }

    pub fn set_on_initialized(&mut self, callback: Option<Arc<dyn Fn() + Send + Sync>>) {
        *self.on_initialized.lock().expect("init callback") = callback;
    }

    pub fn register_capabilities(
        &mut self,
        capabilities: ServerCapabilities,
    ) -> Result<(), ServerError> {
        let mut state = self.state.lock().expect("server state");
        if state.capabilities_locked {
            return Err(ServerError::CapabilitiesLocked);
        }
        state.capabilities = merge_server_capabilities(&state.capabilities, &capabilities);
        drop(state);
        self.register_logging_handler_if_needed();
        self.register_task_handlers_if_needed();
        Ok(())
    }

    pub fn get_capabilities(&self) -> ServerCapabilities {
        self.state
            .lock()
            .expect("server state")
            .capabilities
            .clone()
    }

    pub fn get_client_capabilities(&self) -> Option<ClientCapabilities> {
        self.state
            .lock()
            .expect("server state")
            .client_capabilities
            .clone()
    }

    pub fn get_client_info(&self) -> Option<mcp_core::types::Implementation> {
        self.state.lock().expect("server state").client_info.clone()
    }

    pub fn tool_list_changed_notification(&self) -> NotificationMessage {
        NotificationMessage::new("notifications/tools/list_changed", None)
    }

    pub fn resource_list_changed_notification(&self) -> NotificationMessage {
        NotificationMessage::new("notifications/resources/list_changed", None)
    }

    pub fn prompt_list_changed_notification(&self) -> NotificationMessage {
        NotificationMessage::new("notifications/prompts/list_changed", None)
    }

    pub fn task_status_notification(&self, task: Task) -> Result<NotificationMessage, ServerError> {
        let params = TaskStatusNotificationParams { task };
        Ok(NotificationMessage::new(
            "notifications/tasks/status",
            Some(serde_json::to_value(params)?),
        ))
    }

    /// Create a sampling/createMessage request to send to the client.
    /// Returns the request message that should be sent via the transport.
    ///
    /// # Errors
    /// Returns an error if the client does not support sampling capability.
    pub fn create_message_request(
        &self,
        id: MessageId,
        params: CreateMessageRequestParams,
    ) -> Result<RequestMessage, ServerError> {
        // Check client capability
        let state = self.state.lock().expect("server state");
        let client_caps = state.client_capabilities.as_ref().ok_or_else(|| {
            ServerError::Capability("client capabilities not available (not initialized)".into())
        })?;

        if client_caps.sampling.is_none() {
            return Err(ServerError::Capability(
                "client does not support sampling capability".into(),
            ));
        }

        // Check tools capability if tools are provided
        if params.tools.is_some() || params.tool_choice.is_some() {
            if client_caps
                .sampling
                .as_ref()
                .and_then(|s| s.tools.as_ref())
                .is_none()
            {
                return Err(ServerError::Capability(
                    "client does not support sampling tools capability".into(),
                ));
            }
        }

        let params_value = serde_json::to_value(&params)?;
        Ok(RequestMessage::new(id, "sampling/createMessage", params_value))
    }

    /// Create an elicitation/create request for form-based elicitation.
    /// Returns the request message that should be sent via the transport.
    ///
    /// # Errors
    /// Returns an error if the client does not support form elicitation.
    pub fn elicit_form_request(
        &self,
        id: MessageId,
        params: ElicitRequestFormParams,
    ) -> Result<RequestMessage, ServerError> {
        let state = self.state.lock().expect("server state");
        let client_caps = state.client_capabilities.as_ref().ok_or_else(|| {
            ServerError::Capability("client capabilities not available (not initialized)".into())
        })?;

        // Check elicitation capability - form is default when elicitation is declared
        let elicitation = client_caps.elicitation.as_ref().ok_or_else(|| {
            ServerError::Capability("client does not support elicitation capability".into())
        })?;

        // Form mode is supported if elicitation is declared (form is the default)
        // unless only url is explicitly declared
        let has_form = elicitation.form.is_some();
        let has_url = elicitation.url.is_some();

        if !has_form && has_url {
            return Err(ServerError::Capability(
                "client does not support form elicitation (only url mode)".into(),
            ));
        }

        let params_value = serde_json::to_value(&params)?;
        Ok(RequestMessage::new(id, "elicitation/create", params_value))
    }

    /// Create an elicitation/create request for URL-based elicitation.
    /// Returns the request message that should be sent via the transport.
    ///
    /// # Errors
    /// Returns an error if the client does not support URL elicitation.
    pub fn elicit_url_request(
        &self,
        id: MessageId,
        params: ElicitRequestUrlParams,
    ) -> Result<RequestMessage, ServerError> {
        let state = self.state.lock().expect("server state");
        let client_caps = state.client_capabilities.as_ref().ok_or_else(|| {
            ServerError::Capability("client capabilities not available (not initialized)".into())
        })?;

        let elicitation = client_caps.elicitation.as_ref().ok_or_else(|| {
            ServerError::Capability("client does not support elicitation capability".into())
        })?;

        if elicitation.url.is_none() {
            return Err(ServerError::Capability(
                "client does not support URL elicitation".into(),
            ));
        }

        let params_value = serde_json::to_value(&params)?;
        Ok(RequestMessage::new(id, "elicitation/create", params_value))
    }

    /// Create a notification for URL elicitation completion.
    /// This should be sent after the external URL flow has completed.
    ///
    /// # Errors
    /// Returns an error if the client does not support URL elicitation.
    pub fn elicitation_complete_notification(
        &self,
        elicitation_id: impl Into<String>,
    ) -> Result<NotificationMessage, ServerError> {
        let state = self.state.lock().expect("server state");
        let client_caps = state.client_capabilities.as_ref().ok_or_else(|| {
            ServerError::Capability("client capabilities not available (not initialized)".into())
        })?;

        let elicitation = client_caps.elicitation.as_ref().ok_or_else(|| {
            ServerError::Capability("client does not support elicitation capability".into())
        })?;

        if elicitation.url.is_none() {
            return Err(ServerError::Capability(
                "client does not support URL elicitation".into(),
            ));
        }

        let params = ElicitationCompleteNotificationParams::new(elicitation_id);
        Ok(NotificationMessage::new(
            "notifications/elicitation/complete",
            Some(serde_json::to_value(params)?),
        ))
    }

    /// Check if the client supports sampling.
    pub fn client_supports_sampling(&self) -> bool {
        self.state
            .lock()
            .expect("server state")
            .client_capabilities
            .as_ref()
            .and_then(|c| c.sampling.as_ref())
            .is_some()
    }

    /// Check if the client supports sampling with tools.
    pub fn client_supports_sampling_tools(&self) -> bool {
        self.state
            .lock()
            .expect("server state")
            .client_capabilities
            .as_ref()
            .and_then(|c| c.sampling.as_ref())
            .and_then(|s| s.tools.as_ref())
            .is_some()
    }

    /// Check if the client supports form elicitation.
    pub fn client_supports_form_elicitation(&self) -> bool {
        let state = self.state.lock().expect("server state");
        let client_caps = match state.client_capabilities.as_ref() {
            Some(c) => c,
            None => return false,
        };
        let elicitation = match client_caps.elicitation.as_ref() {
            Some(e) => e,
            None => return false,
        };
        // Form is default if elicitation is declared
        elicitation.form.is_some() || elicitation.url.is_none()
    }

    /// Check if the client supports URL elicitation.
    pub fn client_supports_url_elicitation(&self) -> bool {
        self.state
            .lock()
            .expect("server state")
            .client_capabilities
            .as_ref()
            .and_then(|c| c.elicitation.as_ref())
            .and_then(|e| e.url.as_ref())
            .is_some()
    }

    pub async fn handle_request(
        &self,
        request: RequestMessage,
        session_id: Option<String>,
    ) -> Result<ResultMessage, ServerError> {
        let id = request.id.clone();
        let mut context = RequestContext::default();
        context.session_id = session_id;
        match self
            .protocol
            .handle_request_with_context(request, context)
            .await
        {
            Ok(result) => Ok(result),
            Err(err) => Ok(ResultMessage::failure(id, map_protocol_error(err))),
        }
    }

    pub async fn handle_notification(
        &self,
        notification: NotificationMessage,
        session_id: Option<String>,
    ) -> Result<(), ServerError> {
        let mut context = NotificationContext::default();
        context.session_id = session_id;
        self.protocol
            .handle_notification_with_context(notification, context)
            .await?;
        Ok(())
    }

    pub fn register_request_handler<H>(
        &mut self,
        method: impl Into<String>,
        schema: RootSchema,
        handler: H,
    ) where
        H: RequestHandler,
    {
        self.protocol
            .register_request_handler(method, schema, handler);
    }

    pub fn register_notification_handler<H>(
        &mut self,
        method: impl Into<String>,
        schema: RootSchema,
        handler: H,
    ) where
        H: NotificationHandler,
    {
        self.protocol
            .register_notification_handler(method, schema, handler);
    }

    fn register_initialize_handlers(&mut self) {
        let state = self.state.clone();
        let server_info = self.server_info.clone();
        let handler = RequestHandlerFn::new(
            move |request: &RequestMessage,
                  _context: &RequestContext|
                  -> BoxFuture<'static, Result<Value, ProtocolError>> {
                let state = state.clone();
                let server_info = server_info.clone();
                let params_value = request.params.clone();
                Box::pin(async move {
                    let params: InitializeRequestParams = serde_json::from_value(params_value)?;
                    let requested_version = params.protocol_version;
                    let protocol_version = if SUPPORTED_PROTOCOL_VERSIONS
                        .iter()
                        .any(|version| *version == requested_version)
                    {
                        requested_version
                    } else {
                        mcp_core::types::LATEST_PROTOCOL_VERSION.to_string()
                    };

                    let mut state = state.lock().expect("server state");
                    state.client_capabilities = Some(params.capabilities);
                    state.client_info = Some(params.client_info);
                    state.capabilities_locked = true;

                    let result = InitializeResult {
                        protocol_version,
                        capabilities: state.capabilities.clone(),
                        server_info,
                        instructions: state.instructions.clone(),
                        meta: None,
                    };
                    Ok(serde_json::to_value(result)?)
                })
            },
        );

        self.protocol.register_request_handler(
            "initialize",
            JsonSchemaValidator::schema_for::<InitializeRequestParams>(),
            handler,
        );

        let on_initialized = self.on_initialized.clone();
        let notification_handler = NotificationHandlerFn::new(
            move |_notification: &NotificationMessage,
                  _context: &NotificationContext|
                  -> BoxFuture<'static, Result<(), ProtocolError>> {
                let on_initialized = on_initialized.clone();
                Box::pin(async move {
                    if let Some(callback) = on_initialized.lock().expect("init callback").as_ref() {
                        callback();
                    }
                    Ok(())
                })
            },
        );

        self.protocol.register_notification_handler(
            "notifications/initialized",
            JsonSchemaValidator::schema_for::<Value>(),
            notification_handler,
        );
    }

    fn register_logging_handler_if_needed(&mut self) {
        if self.logging_handler_registered {
            return;
        }
        let logging_enabled = self
            .state
            .lock()
            .expect("server state")
            .capabilities
            .logging
            .is_some();
        if !logging_enabled {
            return;
        }

        let state = self.state.clone();
        let handler = RequestHandlerFn::new(
            move |request: &RequestMessage,
                  context: &RequestContext|
                  -> BoxFuture<'static, Result<Value, ProtocolError>> {
                let state = state.clone();
                let params_value = request.params.clone();
                let context = context.clone();
                Box::pin(async move {
                    let params: SetLevelRequestParams = serde_json::from_value(params_value)?;
                    let mut state = state.lock().expect("server state");
                    state
                        .logging_levels
                        .insert(context.session_id.clone(), params.level);
                    Ok(Value::Object(Default::default()))
                })
            },
        );

        self.protocol.register_request_handler(
            "logging/setLevel",
            JsonSchemaValidator::schema_for::<SetLevelRequestParams>(),
            handler,
        );
        self.logging_handler_registered = true;
    }

    fn register_task_handlers_if_needed(&mut self) {
        if self.task_handlers_registered {
            return;
        }
        let task_store = match self.task_store.clone() {
            Some(store) => store,
            None => return,
        };

        ensure_task_capabilities(self.state.clone());

        let store_for_get = task_store.clone();
        let get_handler = RequestHandlerFn::new(
            move |request: &RequestMessage,
                  _context: &RequestContext|
                  -> BoxFuture<'static, Result<Value, ProtocolError>> {
                let store = store_for_get.clone();
                let params_value = request.params.clone();
                Box::pin(async move {
                    let params: GetTaskRequestParams = serde_json::from_value(params_value)?;
                    let task = store
                        .get_task(&params.task_id)
                        .await?
                        .ok_or_else(|| ProtocolError::Handler("task not found".to_string()))?;
                    let result = GetTaskResult { task };
                    Ok(serde_json::to_value(result)?)
                })
            },
        );

        self.protocol.register_request_handler(
            "tasks/get",
            JsonSchemaValidator::schema_for::<GetTaskRequestParams>(),
            get_handler,
        );

        let store_for_list = task_store.clone();
        let list_handler = RequestHandlerFn::new(
            move |request: &RequestMessage,
                  _context: &RequestContext|
                  -> BoxFuture<'static, Result<Value, ProtocolError>> {
                let store = store_for_list.clone();
                let params_value = request.params.clone();
                Box::pin(async move {
                    let params: PaginatedRequestParams = if params_value.is_null() {
                        PaginatedRequestParams::default()
                    } else {
                        serde_json::from_value(params_value)?
                    };
                    let (tasks, next_cursor) = store.list_tasks(params.cursor).await?;
                    let result = ListTasksResult {
                        pagination: PaginatedResult {
                            next_cursor,
                            meta: None,
                        },
                        tasks,
                    };
                    Ok(serde_json::to_value(result)?)
                })
            },
        );

        self.protocol.register_request_handler(
            "tasks/list",
            JsonSchemaValidator::schema_for::<Option<PaginatedRequestParams>>(),
            list_handler,
        );

        let store_for_result = task_store.clone();
        let result_handler = RequestHandlerFn::new(
            move |request: &RequestMessage,
                  _context: &RequestContext|
                  -> BoxFuture<'static, Result<Value, ProtocolError>> {
                let store = store_for_result.clone();
                let params_value = request.params.clone();
                Box::pin(async move {
                    let params: GetTaskPayloadRequestParams = serde_json::from_value(params_value)?;
                    let result =
                        store
                            .get_task_result(&params.task_id)
                            .await?
                            .ok_or_else(|| {
                                ProtocolError::Handler("task result not available".to_string())
                            })?;
                    match result {
                        Ok(value) => Ok(value),
                        Err(error) => Err(ProtocolError::Handler(error.message)),
                    }
                })
            },
        );

        self.protocol.register_request_handler(
            "tasks/result",
            JsonSchemaValidator::schema_for::<GetTaskPayloadRequestParams>(),
            result_handler,
        );

        let store_for_cancel = task_store.clone();
        let cancel_handler = RequestHandlerFn::new(
            move |request: &RequestMessage,
                  _context: &RequestContext|
                  -> BoxFuture<'static, Result<Value, ProtocolError>> {
                let store = store_for_cancel.clone();
                let params_value = request.params.clone();
                Box::pin(async move {
                    let params: CancelTaskRequestParams = serde_json::from_value(params_value)?;
                    let task = store
                        .cancel_task(&params.task_id)
                        .await?
                        .ok_or_else(|| ProtocolError::Handler("task not found".to_string()))?;
                    let result = CancelTaskResult { task };
                    Ok(serde_json::to_value(result)?)
                })
            },
        );

        self.protocol.register_request_handler(
            "tasks/cancel",
            JsonSchemaValidator::schema_for::<CancelTaskRequestParams>(),
            cancel_handler,
        );

        self.task_handlers_registered = true;
    }
}

fn ensure_task_capabilities(state: Arc<Mutex<ServerState>>) {
    let mut state = state.lock().expect("server state");
    if state.capabilities.tasks.is_some() {
        return;
    }
    state.capabilities.tasks = Some(ServerTasksCapability {
        list: Some(CapabilityFlag::default()),
        cancel: Some(CapabilityFlag::default()),
        requests: Some(ServerTasksRequestCapabilities {
            tools: Some(ServerTasksToolCapabilities {
                call: Some(CapabilityFlag::default()),
            }),
        }),
    });
}

fn map_protocol_error(error: ProtocolError) -> ErrorObject {
    match error {
        ProtocolError::UnknownMethod(method) => ErrorObject::new(
            ErrorCode::MethodNotFound as i32,
            format!("unknown method: {method}"),
            None,
        ),
        ProtocolError::Validation(err) => {
            ErrorObject::new(ErrorCode::InvalidParams as i32, err.to_string(), None)
        }
        ProtocolError::Timeout => {
            ErrorObject::new(ErrorCode::RequestTimeout as i32, "request timed out", None)
        }
        ProtocolError::Cancelled => ErrorObject::new(
            ErrorCode::ConnectionClosed as i32,
            "request cancelled",
            None,
        ),
        ProtocolError::Capability(message) => {
            ErrorObject::new(ErrorCode::InvalidRequest as i32, message, None)
        }
        ProtocolError::TaskUnsupported => ErrorObject::new(
            ErrorCode::InvalidRequest as i32,
            "task support not available",
            None,
        ),
        ProtocolError::Handler(message) => {
            ErrorObject::new(ErrorCode::InternalError as i32, message, None)
        }
        ProtocolError::Serialization(err) => {
            ErrorObject::new(ErrorCode::InternalError as i32, err.to_string(), None)
        }
    }
}

fn merge_server_capabilities(
    current: &ServerCapabilities,
    updates: &ServerCapabilities,
) -> ServerCapabilities {
    ServerCapabilities {
        experimental: merge_experimental(&current.experimental, &updates.experimental),
        logging: merge_flag(current.logging.clone(), updates.logging.clone()),
        completions: merge_flag(current.completions.clone(), updates.completions.clone()),
        prompts: merge_prompt_capabilities(&current.prompts, &updates.prompts),
        resources: merge_resource_capabilities(&current.resources, &updates.resources),
        tools: merge_tool_capabilities(&current.tools, &updates.tools),
        tasks: merge_task_capabilities(&current.tasks, &updates.tasks),
    }
}

fn merge_flag(
    current: Option<CapabilityFlag>,
    updates: Option<CapabilityFlag>,
) -> Option<CapabilityFlag> {
    if updates.is_some() {
        Some(CapabilityFlag::default())
    } else {
        current
    }
}

fn merge_bool(current: Option<bool>, updates: Option<bool>) -> Option<bool> {
    match (current, updates) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        (Some(false), None) => Some(false),
        (None, Some(false)) => Some(false),
        (None, None) => None,
    }
}

fn merge_prompt_capabilities(
    current: &Option<mcp_core::types::PromptCapabilities>,
    updates: &Option<mcp_core::types::PromptCapabilities>,
) -> Option<mcp_core::types::PromptCapabilities> {
    match (current, updates) {
        (_, Some(update)) => Some(mcp_core::types::PromptCapabilities {
            list_changed: merge_bool(
                current.as_ref().and_then(|c| c.list_changed),
                update.list_changed,
            ),
        }),
        (Some(existing), None) => Some(existing.clone()),
        (None, None) => None,
    }
}

fn merge_tool_capabilities(
    current: &Option<mcp_core::types::ToolCapabilities>,
    updates: &Option<mcp_core::types::ToolCapabilities>,
) -> Option<mcp_core::types::ToolCapabilities> {
    match (current, updates) {
        (_, Some(update)) => Some(mcp_core::types::ToolCapabilities {
            list_changed: merge_bool(
                current.as_ref().and_then(|c| c.list_changed),
                update.list_changed,
            ),
        }),
        (Some(existing), None) => Some(existing.clone()),
        (None, None) => None,
    }
}

fn merge_resource_capabilities(
    current: &Option<mcp_core::types::ResourceCapabilities>,
    updates: &Option<mcp_core::types::ResourceCapabilities>,
) -> Option<mcp_core::types::ResourceCapabilities> {
    match (current, updates) {
        (_, Some(update)) => Some(mcp_core::types::ResourceCapabilities {
            subscribe: merge_bool(current.as_ref().and_then(|c| c.subscribe), update.subscribe),
            list_changed: merge_bool(
                current.as_ref().and_then(|c| c.list_changed),
                update.list_changed,
            ),
        }),
        (Some(existing), None) => Some(existing.clone()),
        (None, None) => None,
    }
}

fn merge_task_capabilities(
    current: &Option<ServerTasksCapability>,
    updates: &Option<ServerTasksCapability>,
) -> Option<ServerTasksCapability> {
    match (current, updates) {
        (_, Some(update)) => Some(ServerTasksCapability {
            list: merge_flag(
                current.as_ref().and_then(|c| c.list.clone()),
                update.list.clone(),
            ),
            cancel: merge_flag(
                current.as_ref().and_then(|c| c.cancel.clone()),
                update.cancel.clone(),
            ),
            requests: merge_task_request_capabilities(
                &current.as_ref().and_then(|c| c.requests.clone()),
                &update.requests,
            ),
        }),
        (Some(existing), None) => Some(existing.clone()),
        (None, None) => None,
    }
}

fn merge_task_request_capabilities(
    current: &Option<ServerTasksRequestCapabilities>,
    updates: &Option<ServerTasksRequestCapabilities>,
) -> Option<ServerTasksRequestCapabilities> {
    match (current, updates) {
        (_, Some(update)) => Some(ServerTasksRequestCapabilities {
            tools: merge_task_tool_capabilities(
                &current.as_ref().and_then(|c| c.tools.clone()),
                &update.tools,
            ),
        }),
        (Some(existing), None) => Some(existing.clone()),
        (None, None) => None,
    }
}

fn merge_task_tool_capabilities(
    current: &Option<ServerTasksToolCapabilities>,
    updates: &Option<ServerTasksToolCapabilities>,
) -> Option<ServerTasksToolCapabilities> {
    match (current, updates) {
        (_, Some(update)) => Some(ServerTasksToolCapabilities {
            call: merge_flag(
                current.as_ref().and_then(|c| c.call.clone()),
                update.call.clone(),
            ),
        }),
        (Some(existing), None) => Some(existing.clone()),
        (None, None) => None,
    }
}

fn merge_experimental(
    current: &Option<HashMap<String, CapabilityFlag>>,
    updates: &Option<HashMap<String, CapabilityFlag>>,
) -> Option<HashMap<String, CapabilityFlag>> {
    match (current, updates) {
        (Some(existing), Some(update)) => {
            let mut merged = existing.clone();
            for (key, value) in update {
                merged.insert(key.clone(), *value);
            }
            Some(merged)
        }
        (Some(existing), None) => Some(existing.clone()),
        (None, Some(update)) => Some(update.clone()),
        (None, None) => None,
    }
}
