# Deep Agents 后端协议

## BackendProtocol

**位置**：`libs/deepagents/deepagents/backends/protocol.py`

统一文件操作接口，所有后端实现必须遵循。

### 核心方法

| 方法 | 说明 |
|------|------|
| ls_info(path) | 列出目录 |
| read(file_path, offset, limit) | 读取文件 |
| write(file_path, content) | 写入新文件 |
| edit(file_path, old_string, new_string, replace_all) | 精确替换 |
| grep_raw(pattern, path, glob) | 文本搜索 |
| glob_info(pattern, path) | 模式匹配 |
| upload_files(files) | 批量上传 |
| download_files(paths) | 批量下载 |

### 数据结构

```python
# 文件元信息
FileInfo = TypedDict({
    "path": str,
    "is_dir": NotRequired[bool],
    "size": NotRequired[int],
    "modified_at": NotRequired[str],
})

# 写入结果
WriteResult = dataclass(error, path, files_update)

# 编辑结果
EditResult = dataclass(error, path, files_update, occurrences)
```

## SandboxBackendProtocol

扩展 `BackendProtocol`，增加执行能力：

```python
execute(command: str) -> ExecuteResponse  # output, exit_code, truncated
aexecute(command: str) -> ExecuteResponse
```

## 实现

### StateBackend

**位置**：`libs/deepagents/deepagents/backends/state.py`

- 文件存在 LangGraph state 的 `files` 键
- 会话内持久，checkpoint 后跨步保持
- 通过 `invoke(files={"/path": content})` 传入初始文件
- 不支持 `upload_files`（直接用 invoke 传入）

### FilesystemBackend

- 读写本地磁盘
- 需指定 `root_dir`

### StoreBackend

- 持久化存储（数据库等）
- 需传入 LangGraph `BaseStore`

### CompositeBackend

- 按路径路由到不同后端
- 例如：`/memories/` → StoreBackend，其余 → StateBackend

## BackendFactory

```python
BackendFactory = Callable[[ToolRuntime], BackendProtocol]
```

`create_deep_agent(backend=lambda rt: StateBackend(rt))` 即使用工厂，每个 tool call 时解析 backend。

## 与 OpenCode 的对应

| Deep Agents | OpenCode SDK |
|-------------|--------------|
| BackendProtocol | Workspace / File 抽象 |
| StateBackend + invoke(files={...}) | Session 传入的 workspace 文件 |
| FilesystemBackend | 本地 workspace 目录 |
| SandboxBackendProtocol.execute | PTY / Command 执行 |
