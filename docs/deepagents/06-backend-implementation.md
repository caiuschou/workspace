# 后端实现深度解析

## 1. BackendProtocol 与数据结构

### 1.1 FileData（StateBackend 内部）

```python
FileData = TypedDict({
    "content": list[str],   # 按行
    "created_at": str,      # ISO 8601
    "modified_at": str,
})
```

### 1.2 WriteResult / EditResult

- `WriteResult`：`error`, `path`, `files_update`（StateBackend 需返回 `{path: FileData}`）
- `EditResult`：`error`, `path`, `files_update`, `occurrences`

### 1.3 GrepMatch

```python
GrepMatch = TypedDict({"path": str, "line": int, "text": str})
```

**注意**：BackendProtocol 文档称 grep 为 "literal string"，但 StateBackend/FilesystemBackend 实际用 `re.compile(pattern)` 做 regex。BaseSandbox 的 grep 使用 `grep -F`（fixed-string）。

---

## 2. StateBackend

**位置**：`libs/deepagents/deepagents/backends/state.py`

### 2.1 存储位置

- `runtime.state["files"]`：`dict[path, FileData]`
- 使用 `_file_data_reducer`：`right` 中 value 为 `None` 时表示删除

### 2.2 ls_info 逻辑

- 规范化 path 为 `path/` 形式
- 遍历 `files`，筛选 `k.startswith(normalized_path)` 的项
- 直接子文件：`relative` 不含 `/`
- 子目录：从 `relative` 提取第一段，去重后作为目录项，path 带 `/`

### 2.3 read / write / edit

- `write`：若 path 已存在，返回错误
- `edit`：调用 `perform_string_replacement`，返回 `(new_content, occurrences)` 或错误字符串
- `format_read_response`：分页、加行号、空文件返回 `EMPTY_CONTENT_WARNING`

### 2.4 upload_files / download_files

- `upload_files`：`NotImplementedError`（建议通过 `invoke(files={...})` 传入）
- `download_files`：从 state 读取，返回 `FileDownloadResponse`

---

## 3. FilesystemBackend

**位置**：`libs/deepagents/deepagents/backends/filesystem.py`

### 3.1 virtual_mode

- `virtual_mode=False`：绝对路径直接访问，相对路径基于 `root_dir`，无安全限制
- `virtual_mode=True`：所有路径视为虚拟路径，锚定在 `root_dir`，禁止 `..`、`~`，解析后必须 `relative_to(cwd)`

### 3.2 安全措施

- `os.open(..., O_NOFOLLOW)` 避免跟随符号链接
- `max_file_size_mb`：grep 的 Python fallback 跳过超大门限的文件

### 3.3 grep 实现

- 优先 `_ripgrep_search`：`rg --json` 解析 JSON 输出
- 失败则 `_python_search`：`rglob` 遍历，跳过超大文件，逐行 regex 匹配

### 3.4 glob

使用 `Path.rglob(pattern)`，`pattern` 支持 `**` 等标准 glob。

---

## 4. CompositeBackend

**位置**：`libs/deepagents/deepagents/backends/composite.py`

### 4.1 路由规则

- `routes`：`{"/memories/": StoreBackend, ...}`
- `_get_backend_and_key`：按前缀长度降序匹配，返回 `(backend, stripped_key)`
- `stripped_key`：去掉路由前缀，保留前导 `/`

### 4.2 ls_info("/")

聚合 default 与所有 route 的根目录，route 前缀本身作为目录项展示。

### 4.3 grep_raw / glob_info

- 若 path 匹配某 route：只查该 backend
- 若 path 为 `/` 或 None：查 default + 所有 route，合并结果，路径加回前缀

### 4.4 write/edit 的 files_update 合并

当 routed backend 返回 `files_update` 时，CompositeBackend 尝试合并到 `self.default.runtime.state["files"]`，以便 default 为 StateBackend 时，ls 等能看见 routed 写入（实现上有特殊处理）。

### 4.5 execute

仅委托给 `default`，且 default 须实现 `SandboxBackendProtocol`。

---

## 5. BaseSandbox（SandboxBackendProtocol）

**位置**：`libs/deepagents/deepagents/backends/sandbox.py`

### 5.1 设计思路

只抽象 `execute(command) -> ExecuteResponse`，其余方法通过 shell 命令实现。

### 5.2 ls_info

执行 Python 脚本，`os.scandir` 输出 JSON 行，解析为 `FileInfo`。

### 5.3 read

内联 Python 脚本：按 offset/limit 读取，`print` 行号格式。注意 `file_path` 直接插值，需防范注入（通常由上层保证）。

### 5.4 write / edit

- 使用 heredoc 传 `payload_b64`（base64 JSON），避免 ARG_MAX 和 shell 转义问题
- write：`{"path": "...", "content": "<base64>"}`
- edit：`{"path": "...", "old": "...", "new": "..."}`，exit code 1/2/3/4 表示不同错误

### 5.5 grep_raw

`grep -rHnF`（-F 为 fixed-string），输出 `path:line:text` 解析为 `GrepMatch`。

### 5.6 glob_info

Python 脚本：`glob.glob(pattern, recursive=True)`，输出 JSON 行。

---

## 6. backends/utils.py 工具函数

| 函数 | 作用 |
|------|------|
| `sanitize_tool_call_id` | `.`、`/`、`\` 替换为 `_` |
| `format_content_with_line_numbers` | cat -n 风格，长行拆成 5.1、5.2 |
| `create_file_data` / `update_file_data` | 构造/更新 FileData |
| `perform_string_replacement` | 精确替换，校验唯一性 |
| `truncate_if_too_long` | 按 ~4 chars/token 截断 |
| `grep_matches_from_files` | 内存 files 的 grep，返回 `list[GrepMatch]` |
| `format_grep_matches` | 按 output_mode 格式化 |

---

## 7. Grep：Literal vs Regex

- 工具描述：grep 写的是 "literal text (not regex)"
- StateBackend / FilesystemBackend：实际用 `re.compile(pattern)`
- BaseSandbox：用 `grep -F`，为 literal

若需严格 literal，应对 pattern 做 `re.escape` 或统一使用 BaseSandbox 风格。
