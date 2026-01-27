# 工具调用系统

类型安全的工具调用和注册系统。

## 工具注册表

```rust
use std::collections::HashMap;
use std::sync::Arc;

/// 工具注册表
pub struct ToolRegistry {
    tools: HashMap<Box<str>, Arc<dyn DynTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// 注册工具
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let name: Box<str> = tool.name().into();
        self.tools.insert(name, Arc::new(tool));
    }

    /// 获取工具
    pub fn get(&self, name: &str) -> Option<&Arc<dyn DynTool>> {
        self.tools.get(name)
    }

    /// 列出所有工具
    pub fn list(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_ref()).collect()
    }

    /// 调用工具
    pub async fn invoke(
        &self,
        call: &ToolCall,
    ) -> Result<serde_json::Value, ToolError> {
        let tool = self.get(&call.name)
            .ok_or_else(|| ToolError::NotFound(call.name.clone()))?;

        tool.invoke(call.args.clone()).await
    }

    /// 批量调用
    pub async fn invoke_batch(
        &self,
        calls: Vec<ToolCall>,
    ) -> Vec<Result<serde_json::Value, ToolError>> {
        let futures: Vec<_> = calls
            .iter()
            .map(|call| self.invoke(call))
            .collect();

        futures::future::join_all(futures).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

## 动态工具 Trait

```rust
/// 动态工具 - 类型擦除
#[async_trait]
pub trait DynTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> &serde_json::Value;
    async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, ToolError>;
}

/// 为实现 Tool 的类型自动实现 DynTool
#[async_trait]
impl<T: Tool> DynTool for T {
    fn name(&self) -> &str {
        Tool::name(self)
    }

    fn description(&self) -> &str {
        Tool::description(self)
    }

    fn schema(&self) -> &serde_json::Value {
        Tool::schema(self)
    }

    async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, ToolError> {
        let typed_input: T::Input = serde_json::from_value(input)
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;

        let output = self.execute(typed_input).await?;

        serde_json::to_value(output)
            .map_err(|e| ToolError::SerializationError(e.to_string()))
    }
}
```

## 工具调用

```rust
/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub args: serde_json::Value,
    pub id: String,
}

impl ToolCall {
    pub fn new(name: String, args: serde_json::Value) -> Self {
        Self {
            name,
            args,
            id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// 工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub result: serde_json::Value,
    pub error: Option<String>,
}

/// LLM 函数调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,  // JSON 字符串
}

/// 从 LLM 响应解析工具调用
pub fn parse_tool_calls(
    response: &str,
) -> Vec<ToolCall> {
    // OpenAI 格式
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
        if let Some(calls) = parsed["tool_calls"].as_array() {
            return calls.iter()
                .filter_map(|call| {
                    let function = call.get("function")?;
                    Some(ToolCall::new(
                        function["name"].as_str()?.to_string(),
                        serde_json::from_str(function["arguments"].as_str()?).ok()?,
                    ))
                })
                .collect();
        }
    }

    // 文本格式解析
    // ...
    Vec::new()
}
```

## 内置工具

```rust
/// HTTP 请求工具
pub struct HttpRequest {
    client: reqwest::Client,
}

#[async_trait]
impl Tool for HttpRequest {
    type Input = RequestInput;
    type Output = ResponseOutput;

    fn name(&self) -> &str {
        "http_request"
    }

    fn description(&self) -> &str {
        "发送 HTTP 请求"
    }

    fn schema(&self) -> &serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {"type": "string"},
                "method": {"enum": ["GET", "POST", "PUT", "DELETE"]},
                "body": {"type": "string"},
            },
            "required": ["url", "method"]
        })
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ToolError> {
        let method = match input.method.as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            _ => return Err(ToolError::InvalidInput("Invalid method".into())),
        };

        let request = self.client
            .request(method, &input.url);

        let request = if let Some(body) = input.body {
            request.body(body)
        } else {
            request
        };

        let response = request.send().await?;
        let status = response.status();
        let body = response.text().await?;

        Ok(ResponseOutput {
            status: status.as_u16(),
            body,
        })
    }
}

#[derive(Deserialize)]
pub struct RequestInput {
    pub url: String,
    pub method: String,
    pub body: Option<String>,
}

#[derive(Serialize)]
pub struct ResponseOutput {
    pub status: u16,
    pub body: String,
}

/// 文件操作工具
pub struct FileOps {
    base_dir: PathBuf,
}

#[async_trait]
impl Tool for FileOps {
    type Input = FileOpInput;
    type Output = FileOpOutput;

    fn name(&self) -> &str {
        "file_ops"
    }

    fn description(&self) -> &str {
        "读写文件"
    }

    fn schema(&self) -> &serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "op": {"enum": ["read", "write", "list"]},
                "path": {"type": "string"},
                "content": {"type": "string"},
            },
            "required": ["op", "path"]
        })
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ToolError> {
        let path = self.base_dir.join(&input.path);

        match input.op.as_str() {
            "read" => {
                let content = tokio::fs::read_to_string(&path).await?;
                Ok(FileOpOutput::Read { content })
            }
            "write" => {
                tokio::fs::write(&path, input.content.unwrap_or_default()).await?;
                Ok(FileOpOutput::Write)
            }
            "list" => {
                let mut entries = Vec::new();
                let mut dir = tokio::fs::read_dir(&path).await?;
                while let Some(entry) = dir.next_entry().await? {
                    entries.push(entry.file_name().to_string_lossy().into_owned());
                }
                Ok(FileOpOutput::List { entries })
            }
            _ => Err(ToolError::InvalidInput("Invalid operation".into())),
        }
    }
}

#[derive(Deserialize)]
pub struct FileOpInput {
    pub op: String,
    pub path: String,
    pub content: Option<String>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum FileOpOutput {
    Read { content: String },
    Write,
    List { entries: Vec<String> },
}
```

## 工具验证

```rust
/// 工具验证器
pub struct ToolValidator {
    schema: serde_json::Value,
}

impl ToolValidator {
    pub fn new(schema: serde_json::Value) -> Self {
        Self { schema }
    }

    pub fn validate(&self, input: &serde_json::Value) -> Result<(), ValidationError> {
        // 使用 JSON Schema 验证
        // 这里简化实现
        if let Some(obj) = self.schema.as_object() {
            if let Some(required) = obj.get("required").and_then(|v| v.as_array()) {
                for field in required {
                    if let Some(s) = field.as_str() {
                        if input.get(s).is_none() {
                            return Err(ValidationError::MissingField(s.to_string()));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// 验证工具装饰器
pub struct ValidatedTool<T: Tool> {
    inner: T,
    validator: ToolValidator,
}

impl<T: Tool> ValidatedTool<T> {
    pub fn new(inner: T) -> Self {
        let validator = ToolValidator::new(inner.schema().clone());
        Self { inner, validator }
    }
}

#[async_trait]
impl<T: Tool + Send + Sync> Tool for ValidatedTool<T> {
    type Input = T::Input;
    type Output = T::Output;

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn schema(&self) -> &serde_json::Value {
        self.inner.schema()
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ToolError> {
        // 验证输入
        let input_value = serde_json::to_value(&input)
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;

        self.validator.validate(&input_value)?;

        // 执行工具
        self.inner.execute(input).await
    }
}
```

## 并发工具执行

```rust
/// 并发执行工具
pub async fn execute_tools_concurrent(
    registry: &ToolRegistry,
    calls: Vec<ToolCall>,
    max_concurrent: usize,
) -> Vec<ToolResult> {
    use futures::stream::{self, StreamExt};

    stream::iter(calls)
        .map(|call| {
            let registry = registry.clone();
            async move {
                let result = registry.invoke(&call).await;
                ToolResult {
                    call_id: call.id,
                    result: result.unwrap_or_default(),
                    error: result.err().map(|e| e.to_string()),
                }
            }
        })
        .buffer_unordered(max_concurrent)
        .collect()
        .await
}
```
