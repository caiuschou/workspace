use std::{collections::HashMap, sync::Arc};

use futures::{FutureExt, select};
use serde_json::Value;

use crate::schema::SchemaValidator;
use crate::types::{
    CreateTaskResult, ErrorCode, ErrorObject, NotificationMessage, RequestMessage, RequestMeta,
    ResultMessage, TaskMetadata,
};

use super::{
    CapabilityChecker, NotificationContext, NotificationHandler, ProtocolError, ProtocolOptions,
    RequestContext, RequestHandler, TaskStore,
};

struct RequestHandlerRegistration<S> {
    handler: Arc<dyn RequestHandler>,
    schema: S,
}

struct NotificationHandlerRegistration<S> {
    handler: Arc<dyn NotificationHandler>,
    schema: S,
}

/// A JSON-RPC protocol runtime that validates incoming payloads.
pub struct Protocol<V: SchemaValidator = crate::schema::JsonSchemaValidator> {
    validator: V,
    options: ProtocolOptions,
    request_handlers: HashMap<String, RequestHandlerRegistration<V::Schema>>,
    notification_handlers: HashMap<String, NotificationHandlerRegistration<V::Schema>>,
}

impl<V: SchemaValidator> Protocol<V> {
    /// Create a new protocol runtime that validates incoming payloads.
    pub fn new(validator: V) -> Self {
        Self::with_options(validator, ProtocolOptions::default())
    }

    /// Create a new protocol runtime with explicit options.
    pub fn with_options(validator: V, options: ProtocolOptions) -> Self {
        Self {
            validator,
            options,
            request_handlers: HashMap::new(),
            notification_handlers: HashMap::new(),
        }
    }

    /// Override the capability checker.
    pub fn set_capability_checker(&mut self, checker: Option<Arc<dyn CapabilityChecker>>) {
        self.options.capability_checker = checker;
    }

    /// Configure a task store for task-augmented requests.
    pub fn set_task_store(&mut self, store: Option<Arc<dyn TaskStore>>) {
        self.options.task_store = store;
    }

    /// Register a handler together with the schema that describes its params.
    pub fn register_handler<H>(&mut self, method: impl Into<String>, schema: V::Schema, handler: H)
    where
        H: RequestHandler,
    {
        self.register_request_handler(method, schema, handler);
    }

    /// Register a handler for requests.
    pub fn register_request_handler<H>(
        &mut self,
        method: impl Into<String>,
        schema: V::Schema,
        handler: H,
    ) where
        H: RequestHandler,
    {
        let method = method.into();
        if let Some(checker) = self.options.capability_checker.as_ref() {
            if let Err(err) = checker.assert_request_handler(&method) {
                panic!("request handler capability check failed: {err}");
            }
        }
        self.request_handlers.insert(
            method,
            RequestHandlerRegistration {
                handler: Arc::new(handler),
                schema,
            },
        );
    }

    /// Register a handler for notifications.
    pub fn register_notification_handler<H>(
        &mut self,
        method: impl Into<String>,
        schema: V::Schema,
        handler: H,
    ) where
        H: NotificationHandler,
    {
        let method = method.into();
        if let Some(checker) = self.options.capability_checker.as_ref() {
            if let Err(err) = checker.assert_notification_handler(&method) {
                panic!("notification handler capability check failed: {err}");
            }
        }
        self.notification_handlers.insert(
            method,
            NotificationHandlerRegistration {
                handler: Arc::new(handler),
                schema,
            },
        );
    }

    /// Handle a request by validating it and invoking the handler.
    pub async fn handle_request(
        &self,
        request: RequestMessage,
    ) -> Result<ResultMessage, ProtocolError> {
        self.handle_request_with_context(request, RequestContext::default())
            .await
    }

    /// Handle a request with explicit context options.
    pub async fn handle_request_with_context(
        &self,
        request: RequestMessage,
        mut context: RequestContext,
    ) -> Result<ResultMessage, ProtocolError> {
        let entry = self
            .request_handlers
            .get(&request.method)
            .ok_or_else(|| ProtocolError::UnknownMethod(request.method.clone()))?;

        if let Some(checker) = self.options.capability_checker.as_ref() {
            checker.assert_request(&request.method)?;
        }

        self.validator.validate(&entry.schema, &request.params)?;

        context.meta = context.meta.or_else(|| extract_meta(&request.params));
        context.task = context.task.or_else(|| extract_task(&request.params));

        if let Some(task) = context.task.clone() {
            let store = self
                .options
                .task_store
                .as_ref()
                .ok_or(ProtocolError::TaskUnsupported)?;
            let task_state = store
                .create_task(task, request.id.clone(), request.clone())
                .await?;
            let result = run_with_options(entry.handler.as_ref(), &request, &context).await;
            let store_result = match result {
                Ok(value) => store.set_task_result(&task_state.task_id, Ok(value)).await,
                Err(err) => {
                    let error =
                        ErrorObject::new(ErrorCode::InternalError as i32, err.to_string(), None);
                    store.set_task_result(&task_state.task_id, Err(error)).await
                }
            };
            store_result?;

            let response = CreateTaskResult {
                task: task_state,
                meta: context.meta.clone(),
            };
            let value = serde_json::to_value(response)?;
            return Ok(ResultMessage::success(request.id.clone(), value));
        }

        let result_value = run_with_options(entry.handler.as_ref(), &request, &context).await?;
        Ok(ResultMessage::success(request.id.clone(), result_value))
    }

    /// Handle a notification by validating it and invoking the handler.
    pub async fn handle_notification(
        &self,
        notification: NotificationMessage,
    ) -> Result<(), ProtocolError> {
        self.handle_notification_with_context(notification, NotificationContext::default())
            .await
    }

    /// Handle a notification with explicit context.
    pub async fn handle_notification_with_context(
        &self,
        notification: NotificationMessage,
        mut context: NotificationContext,
    ) -> Result<(), ProtocolError> {
        let entry = self
            .notification_handlers
            .get(&notification.method)
            .ok_or_else(|| ProtocolError::UnknownMethod(notification.method.clone()))?;

        if let Some(checker) = self.options.capability_checker.as_ref() {
            checker.assert_notification(&notification.method)?;
        }

        let params = notification.params.clone().unwrap_or(Value::Null);
        self.validator.validate(&entry.schema, &params)?;

        context.meta = context.meta.or_else(|| extract_meta(&params));

        entry.handler.handle(&notification, &context).await
    }
}

impl Default for Protocol<crate::schema::JsonSchemaValidator> {
    fn default() -> Self {
        Self::new(crate::schema::JsonSchemaValidator::default())
    }
}

fn extract_meta(params: &Value) -> Option<RequestMeta> {
    let meta = params.get("_meta")?.clone();
    serde_json::from_value(meta).ok()
}

fn extract_task(params: &Value) -> Option<TaskMetadata> {
    let task = params.get("task")?.clone();
    serde_json::from_value(task).ok()
}

async fn run_with_options(
    handler: &dyn RequestHandler,
    request: &RequestMessage,
    context: &RequestContext,
) -> Result<Value, ProtocolError> {
    if let Some(token) = context.options.cancel_token.as_ref() {
        if token.is_cancelled() {
            return Err(ProtocolError::Cancelled);
        }
    }

    let fut = handler.handle(request, context).fuse();
    futures::pin_mut!(fut);

    match (
        context.options.timeout,
        context.options.cancel_token.as_ref(),
    ) {
        (Some(timeout), Some(token)) => {
            let delay = futures_timer::Delay::new(timeout).fuse();
            let cancel = token.cancelled().fuse();
            futures::pin_mut!(delay, cancel);
            select! {
                result = fut => result,
                _ = delay => Err(ProtocolError::Timeout),
                _ = cancel => Err(ProtocolError::Cancelled),
            }
        }
        (Some(timeout), None) => {
            let delay = futures_timer::Delay::new(timeout).fuse();
            futures::pin_mut!(delay);
            select! {
                result = fut => result,
                _ = delay => Err(ProtocolError::Timeout),
            }
        }
        (None, Some(token)) => {
            let cancel = token.cancelled().fuse();
            futures::pin_mut!(cancel);
            select! {
                result = fut => result,
                _ = cancel => Err(ProtocolError::Cancelled),
            }
        }
        (None, None) => fut.await,
    }
}
