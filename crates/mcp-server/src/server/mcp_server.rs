use std::sync::{Arc, Mutex};

use futures::future::BoxFuture;
use serde_json::Value;

use mcp_core::protocol::{ProtocolError, RequestContext};
use mcp_core::schema::JsonSchemaValidator;
use mcp_core::types::{
    CallToolRequestParams, CreateMessageRequestParams, ElicitRequestFormParams,
    ElicitRequestUrlParams, GetPromptRequestParams, ListPromptsResult, ListResourceTemplatesResult,
    ListResourcesResult, ListToolsResult, MessageId, NotificationMessage, PaginatedRequestParams,
    PaginatedResult, PromptCapabilities, RequestMessage, ResourceCapabilities, ResourceRequestParams,
    ServerCapabilities, ToolCapabilities,
};

use crate::server::handlers::{PromptHandler, RequestHandlerFn, ResourceHandler, ToolHandler};
use crate::server::registries::{PromptRegistry, ResourceRegistry, ToolRegistry};
use crate::server::{Server, ServerError, ServerOptions};

/// High-level MCP server with tool/resource/prompt registries.
pub struct McpServer {
    server: Server,
    tools: Arc<Mutex<ToolRegistry>>,
    resources: Arc<Mutex<ResourceRegistry>>,
    prompts: Arc<Mutex<PromptRegistry>>,
    tool_handlers_initialized: bool,
    resource_handlers_initialized: bool,
    prompt_handlers_initialized: bool,
}

impl McpServer {
    pub fn new(server_info: mcp_core::types::Implementation, options: ServerOptions) -> Self {
        Self {
            server: Server::new(server_info, options),
            tools: Arc::new(Mutex::new(ToolRegistry::default())),
            resources: Arc::new(Mutex::new(ResourceRegistry::default())),
            prompts: Arc::new(Mutex::new(PromptRegistry::default())),
            tool_handlers_initialized: false,
            resource_handlers_initialized: false,
            prompt_handlers_initialized: false,
        }
    }

    pub fn server(&self) -> &Server {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut Server {
        &mut self.server
    }

    pub fn register_tool(
        &mut self,
        tool: mcp_core::types::Tool,
        handler: impl ToolHandler,
    ) -> Result<(), ServerError> {
        self.tools
            .lock()
            .expect("tool registry")
            .register_tool(tool, handler);
        self.server.register_capabilities(ServerCapabilities {
            tools: Some(ToolCapabilities {
                list_changed: Some(true),
            }),
            ..Default::default()
        })?;
        self.ensure_tool_handlers()?;
        Ok(())
    }

    pub fn register_resource(
        &mut self,
        resource: mcp_core::types::Resource,
        handler: impl ResourceHandler,
    ) -> Result<(), ServerError> {
        self.resources
            .lock()
            .expect("resource registry")
            .register_resource(resource, handler);
        self.server.register_capabilities(ServerCapabilities {
            resources: Some(ResourceCapabilities {
                subscribe: None,
                list_changed: Some(true),
            }),
            ..Default::default()
        })?;
        self.ensure_resource_handlers()?;
        Ok(())
    }

    pub fn register_resource_template(
        &mut self,
        template: mcp_core::types::ResourceTemplate,
    ) -> Result<(), ServerError> {
        self.resources
            .lock()
            .expect("resource registry")
            .register_template(template);
        self.server.register_capabilities(ServerCapabilities {
            resources: Some(ResourceCapabilities {
                subscribe: None,
                list_changed: Some(true),
            }),
            ..Default::default()
        })?;
        self.ensure_resource_handlers()?;
        Ok(())
    }

    pub fn register_prompt(
        &mut self,
        prompt: mcp_core::types::Prompt,
        handler: impl PromptHandler,
    ) -> Result<(), ServerError> {
        self.prompts
            .lock()
            .expect("prompt registry")
            .register_prompt(prompt, handler);
        self.server.register_capabilities(ServerCapabilities {
            prompts: Some(PromptCapabilities {
                list_changed: Some(true),
            }),
            ..Default::default()
        })?;
        self.ensure_prompt_handlers()?;
        Ok(())
    }

    pub fn tool_list_changed_notification(&self) -> NotificationMessage {
        self.server.tool_list_changed_notification()
    }

    pub fn resource_list_changed_notification(&self) -> NotificationMessage {
        self.server.resource_list_changed_notification()
    }

    pub fn prompt_list_changed_notification(&self) -> NotificationMessage {
        self.server.prompt_list_changed_notification()
    }

    /// Add a resource to the registry after initialization without modifying capabilities.
    /// This is useful when resources are discovered dynamically after the client has initialized.
    pub fn add_resource_after_init(
        &self,
        resource: mcp_core::types::Resource,
        handler: impl ResourceHandler,
    ) {
        self.resources
            .lock()
            .expect("resource registry")
            .register_resource(resource, handler);
    }

    // ==================== Sampling API ====================

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
        self.server.create_message_request(id, params)
    }

    /// Check if the client supports sampling.
    pub fn client_supports_sampling(&self) -> bool {
        self.server.client_supports_sampling()
    }

    /// Check if the client supports sampling with tools.
    pub fn client_supports_sampling_tools(&self) -> bool {
        self.server.client_supports_sampling_tools()
    }

    // ==================== Elicitation API ====================

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
        self.server.elicit_form_request(id, params)
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
        self.server.elicit_url_request(id, params)
    }

    /// Create a notification for URL elicitation completion.
    /// This should be sent after the external URL flow has completed.
    pub fn elicitation_complete_notification(
        &self,
        elicitation_id: impl Into<String>,
    ) -> Result<NotificationMessage, ServerError> {
        self.server.elicitation_complete_notification(elicitation_id)
    }

    /// Check if the client supports form elicitation.
    pub fn client_supports_form_elicitation(&self) -> bool {
        self.server.client_supports_form_elicitation()
    }

    /// Check if the client supports URL elicitation.
    pub fn client_supports_url_elicitation(&self) -> bool {
        self.server.client_supports_url_elicitation()
    }

    fn ensure_tool_handlers(&mut self) -> Result<(), ServerError> {
        if self.tool_handlers_initialized {
            return Ok(());
        }

        let tools = self.tools.clone();
        let list_handler = RequestHandlerFn::new(
            move |_request: &RequestMessage,
                  _context: &RequestContext|
                  -> BoxFuture<'static, Result<Value, ProtocolError>> {
                let tools = tools.clone();
                Box::pin(async move {
                    let tools = tools.lock().expect("tool registry").list_tools();
                    let result = ListToolsResult {
                        pagination: PaginatedResult::default(),
                        tools,
                    };
                    Ok(serde_json::to_value(result)?)
                })
            },
        );

        self.server.register_request_handler(
            "tools/list",
            JsonSchemaValidator::schema_for::<Option<PaginatedRequestParams>>(),
            list_handler,
        );

        let tools = self.tools.clone();
        let call_handler = RequestHandlerFn::new(
            move |request: &RequestMessage,
                  context: &RequestContext|
                  -> BoxFuture<'static, Result<Value, ProtocolError>> {
                let tools = tools.clone();
                let params_value = request.params.clone();
                let context = context.clone();
                Box::pin(async move {
                    let params: CallToolRequestParams = serde_json::from_value(params_value)?;
                    let handler = tools
                        .lock()
                        .expect("tool registry")
                        .handler(&params.name)
                        .ok_or_else(|| ProtocolError::Handler("tool not found".to_string()))?;
                    let result = handler
                        .call(params.arguments, context)
                        .await
                        .map_err(|err| ProtocolError::Handler(err.to_string()))?;
                    Ok(serde_json::to_value(result)?)
                })
            },
        );

        self.server.register_request_handler(
            "tools/call",
            JsonSchemaValidator::schema_for::<CallToolRequestParams>(),
            call_handler,
        );

        self.tool_handlers_initialized = true;
        Ok(())
    }

    fn ensure_resource_handlers(&mut self) -> Result<(), ServerError> {
        if self.resource_handlers_initialized {
            return Ok(());
        }

        let resources = self.resources.clone();
        let list_handler = RequestHandlerFn::new(
            move |_request: &RequestMessage,
                  _context: &RequestContext|
                  -> BoxFuture<'static, Result<Value, ProtocolError>> {
                let resources = resources.clone();
                Box::pin(async move {
                    let resources = resources
                        .lock()
                        .expect("resource registry")
                        .list_resources();
                    let result = ListResourcesResult {
                        pagination: PaginatedResult::default(),
                        resources,
                    };
                    Ok(serde_json::to_value(result)?)
                })
            },
        );

        self.server.register_request_handler(
            "resources/list",
            JsonSchemaValidator::schema_for::<Option<PaginatedRequestParams>>(),
            list_handler,
        );

        let templates = self.resources.clone();
        let template_handler = RequestHandlerFn::new(
            move |_request: &RequestMessage,
                  _context: &RequestContext|
                  -> BoxFuture<'static, Result<Value, ProtocolError>> {
                let templates = templates.clone();
                Box::pin(async move {
                    let templates = templates
                        .lock()
                        .expect("resource registry")
                        .list_templates();
                    let result = ListResourceTemplatesResult {
                        pagination: PaginatedResult::default(),
                        resource_templates: templates,
                    };
                    Ok(serde_json::to_value(result)?)
                })
            },
        );

        self.server.register_request_handler(
            "resources/templates/list",
            JsonSchemaValidator::schema_for::<Option<PaginatedRequestParams>>(),
            template_handler,
        );

        let resources = self.resources.clone();
        let read_handler = RequestHandlerFn::new(
            move |request: &RequestMessage,
                  context: &RequestContext|
                  -> BoxFuture<'static, Result<Value, ProtocolError>> {
                let resources = resources.clone();
                let params_value = request.params.clone();
                let context = context.clone();
                Box::pin(async move {
                    let params: ResourceRequestParams = serde_json::from_value(params_value)?;
                    let handler = resources
                        .lock()
                        .expect("resource registry")
                        .handler(&params.uri)
                        .ok_or_else(|| ProtocolError::Handler("resource not found".to_string()))?;
                    let result = handler
                        .read(params.uri.clone(), context)
                        .await
                        .map_err(|err| ProtocolError::Handler(err.to_string()))?;
                    Ok(serde_json::to_value(result)?)
                })
            },
        );

        self.server.register_request_handler(
            "resources/read",
            JsonSchemaValidator::schema_for::<ResourceRequestParams>(),
            read_handler,
        );

        self.resource_handlers_initialized = true;
        Ok(())
    }

    fn ensure_prompt_handlers(&mut self) -> Result<(), ServerError> {
        if self.prompt_handlers_initialized {
            return Ok(());
        }

        let prompts = self.prompts.clone();
        let list_handler = RequestHandlerFn::new(
            move |_request: &RequestMessage,
                  _context: &RequestContext|
                  -> BoxFuture<'static, Result<Value, ProtocolError>> {
                let prompts = prompts.clone();
                Box::pin(async move {
                    let prompts = prompts.lock().expect("prompt registry").list_prompts();
                    let result = ListPromptsResult {
                        pagination: PaginatedResult::default(),
                        prompts,
                    };
                    Ok(serde_json::to_value(result)?)
                })
            },
        );

        self.server.register_request_handler(
            "prompts/list",
            JsonSchemaValidator::schema_for::<Option<PaginatedRequestParams>>(),
            list_handler,
        );

        let prompts = self.prompts.clone();
        let get_handler = RequestHandlerFn::new(
            move |request: &RequestMessage,
                  context: &RequestContext|
                  -> BoxFuture<'static, Result<Value, ProtocolError>> {
                let prompts = prompts.clone();
                let params_value = request.params.clone();
                let context = context.clone();
                Box::pin(async move {
                    let params: GetPromptRequestParams = serde_json::from_value(params_value)?;
                    let handler = prompts
                        .lock()
                        .expect("prompt registry")
                        .handler(&params.name)
                        .ok_or_else(|| ProtocolError::Handler("prompt not found".to_string()))?;
                    let result = handler
                        .get(params.arguments, context)
                        .await
                        .map_err(|err| ProtocolError::Handler(err.to_string()))?;
                    Ok(serde_json::to_value(result)?)
                })
            },
        );

        self.server.register_request_handler(
            "prompts/get",
            JsonSchemaValidator::schema_for::<GetPromptRequestParams>(),
            get_handler,
        );

        self.prompt_handlers_initialized = true;
        Ok(())
    }
}
