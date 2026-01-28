# Zoekt JSON API 接口文档

本文档基于 `thirdparty/zoekt` 中与 JSON 接口相关的代码整理，描述 zoekt-webserver 在开启 RPC 时提供的 HTTP JSON API。

## 1. 代码位置与入口

### 1.1 实现位置

| 职责 | 文件路径 | 说明 |
|------|----------|------|
| JSON HTTP 路由与请求体处理 | `internal/json/json.go` | `JSONServer(searcher)` 返回 `/search`、`/list` 的 Handler |
| 挂载到 Web 服务 | `web/server.go` | `Server.RPC == true` 时，将 `zjson.JSONServer(...)` 挂到 `/api/` 前缀下 |
| 模板用数据结构（非 JSON API 契约） | `web/api.go` | `ResultInput`、`FileMatch`、`RepoListInput` 等供 HTML 模板使用 |
| 核心类型定义（请求/响应契约） | `api.go`（仓库根） | `SearchOptions`、`SearchResult`、`RepoList`、`ListOptions`、`FileMatch` 等 |
| 官方简要说明 | `doc/json-api.md` | 仓库自带的简短 API 说明 |

### 1.2 挂载方式

在 `web/server.go` 的 `NewMux` 中，当 `Server.RPC == true` 时：

```go
if s.RPC {
    mux.Handle("/api/", http.StripPrefix("/api", zjson.JSONServer(traceAwareSearcher{s.Searcher})))
}
```

因此实际暴露的路径为：

- **搜索**：`POST /api/search`
- **仓库列表**：`POST /api/list`

请求与响应均为 JSON，`Content-Type: application/json`。仅支持 POST。

---

## 2. 搜索接口：POST /api/search

### 2.1 请求体

在 `internal/json/json.go` 中，请求体被解码为 `jsonSearchArgs`：

```go
type jsonSearchArgs struct {
    Q       string             // 必填，查询字符串，语法见 query 包
    RepoIDs *[]uint32          // 可选，限定在哪些仓库 ID 内搜索
    Opts    *zoekt.SearchOptions  // 可选，搜索选项；为 nil 时使用默认
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `Q` | string | 是 | 查询字符串，由 `query.Parse` 解析，见 [query 语法](https://github.com/sourcegraph/zoekt/blob/main/doc/query_syntax.md) |
| `RepoIDs` | []uint32 | 否 | 若索引时带有 `repoid`，可在此限定只在这些仓库 ID 中搜索 |
| `Opts` | object | 否 | 搜索选项，见下表；为 null/省略时使用零值，并在服务端做部分默认与启发式限制 |

**SearchOptions 常用字段（与 JSON 一一对应）：**

| 字段 | 类型 | 说明 |
|------|------|------|
| `EstimateDocCount` | bool | 为 true 时仅做文档数估计（如填入 `ShardFilesConsidered`），不返回匹配内容 |
| `Whole` | bool | 为 true 时返回整文件内容 |
| `ShardMaxMatchCount` | int | 单个 shard 内最多保留的匹配数 |
| `TotalMaxMatchCount` | int | 全局最多匹配数 |
| `ShardRepoMaxMatchCount` | int | 单 shard 内单仓库最多匹配数（多仓库复合 shard 时常用） |
| `MaxWallTime` | duration | 搜索最大耗时；为 0 时 JSON 服务会用默认 20s |
| `MaxDocDisplayCount` | int | 汇总排序后最多返回的文档（文件）数 |
| `MaxMatchDisplayCount` | int | 汇总排序后最多返回的匹配条数 |
| `NumContextLines` | int | 每条匹配前后各带多少行上下文 |
| `ChunkMatches` | bool | 为 true 时用 ChunkMatches 而非 LineMatches（实验性） |
| `UseBM25Scoring` | bool | 使用 BM25 风格打分（实验性） |
| `Trace` | bool | 是否开启 tracing |
| `DebugScore` | bool | 结果中是否带打分调试信息 |

超时逻辑：若 `Opts.MaxWallTime == 0`，`internal/json` 会对请求应用 `defaultTimeout = 20s`。此外，若设置了 `MaxDocDisplayCount` 且未设置 `ShardMaxMatchCount`，会通过 `CalculateDefaultSearchLimits` 根据 `ShardFilesConsidered` 等做启发式上限计算。

### 2.2 响应体

成功时 HTTP 200，body 为：

```json
{
  "Result": <zoekt.SearchResult>
}
```

`SearchResult`（`api.go`）结构概要：

| 字段 | 类型 | 说明 |
|------|------|------|
| `Stats` | object | 内嵌；见下方 Stats |
| `Files` | array | `FileMatch` 列表 |
| `RepoURLs` | map string→string | 仓库 → URL 模板 |
| `LineFragments` | map string→string | 仓库 → 行号片段模板 |

**Stats（搜索统计）：**

包含但不限于：`ContentBytesLoaded`、`IndexBytesLoaded`、`Crashes`、`Duration`、`FileCount`、`ShardFilesConsidered`、`FilesConsidered`、`FilesLoaded`、`FilesSkipped`、`ShardsScanned`、`ShardsSkipped`、`ShardsSkippedFilter`、`MatchCount`、`NgramMatches`、`NgramLookups`、`Wait`、`MatchTreeConstruction`、`MatchTreeSearch`、`RegexpsConsidered`、`FlushReason` 等。

**FileMatch：**

| 字段 | 类型 | 说明 |
|------|------|------|
| `FileName` | string | 文件名 |
| `Repository` | string | 仓库名 |
| `SubRepositoryName` | string | 子仓库名（可选） |
| `SubRepositoryPath` | string | 子仓库挂载路径（可选） |
| `Version` | string | 提交 SHA（可选） |
| `Language` | string | 检测到的语言 |
| `Debug` | string | 调试信息（需 Opts.DebugScore） |
| `Branches` | []string | 命中的分支 |
| `LineMatches` | []LineMatch | 行级匹配（默认） |
| `ChunkMatches` | []ChunkMatch | 块级匹配（Opts.ChunkMatches 时） |
| `Content` | []byte | 仅在请求整文件等时填充 |
| `Checksum` | []byte | 内容校验和 |
| `Score` | float64 | 相关性分数 |
| `RepositoryPriority` | float64 | 仓库优先级（Sourcegraph 扩展） |
| `RepositoryID` | uint32 | 仓库 ID（Sourcegraph 扩展） |

**LineMatch：**

包含 `Line`、`LineStart`、`LineEnd`、`LineNumber`、`Before`、`After`、`FileName`、`Score`、`DebugScore`、`LineFragments` 等。`Before`/`After` 在 `NumContextLines > 0` 时被填充。

**ChunkMatch（块匹配）：**

包含 `Content`、`Ranges`、`SymbolInfo`、`FileName`、`ContentStart`、`Score`、`BestLineMatch` 等。

注意：`SearchResult.Progress` 在 JSON 中不序列化（`json:"-"`），因含 -Inf 无法安全编码。

### 2.3 错误响应

请求体解析失败、缺少 `Q`、query 解析错误、或搜索执行错误时，返回 4xx/5xx，body 形如：

```json
{ "Error": "错误信息字符串" }
```

方法非 POST 时返回 405，body 同上。

### 2.4 示例

```bash
# 基本搜索
curl -X POST -H "Content-Type: application/json" -d '{"Q":"needle"}' 'http://127.0.0.1:6070/api/search'

# 按仓库 ID 过滤
curl -X POST -H "Content-Type: application/json" -d '{"Q":"needle","RepoIDs":[1234,4567]}' 'http://127.0.0.1:6070/api/search'

# 带选项：上下文行数、文档数上限
curl -X POST -H "Content-Type: application/json" \
  -d '{"Q":"needle","Opts":{"NumContextLines":5,"MaxDocDisplayCount":100}}' \
  'http://127.0.0.1:6070/api/search'
```

---

## 3. 仓库列表接口：POST /api/list

### 3.1 请求体

解码为 `jsonListArgs`：

```go
type jsonListArgs struct {
    Q    string              // 必填，仅可包含 query.Repo 类 atom
    Opts *zoekt.ListOptions  // 可选
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `Q` | string | 是 | 由 `query.Parse` 解析，且应仅包含“按仓库”的约束（如 repo: 等） |
| `Opts` | object | 否 | `ListOptions`，目前主要含 `Field`，用于决定返回 `Repos` 还是 `ReposMap` |

**ListOptions：**

- `Field`：`RepoListField`，0 = RepoListFieldRepos（填充 `Repos`），2 = RepoListFieldReposMap（填充 `ReposMap`）。为 nil 时按 Repos 处理。

### 3.2 响应体

成功时 HTTP 200：

```json
{
  "List": <zoekt.RepoList>
}
```

**RepoList：**

| 字段 | 类型 | 说明 |
|------|------|------|
| `Repos` | []*RepoListEntry | `Field == RepoListFieldRepos` 时填充 |
| `ReposMap` | ReposMap | `Field == RepoListFieldReposMap` 时填充 |
| `Crashes` | int | 崩溃的 shard 数 |
| `Stats` | RepoStats | 聚合统计 |

**RepoListEntry** 包含 `Repository`、`IndexMetadata`、`Stats`。**RepoStats** 含 `Shards`、`IndexBytes`、`Documents`、`ContentBytes`、`NewLinesCount` 等。

### 3.3 错误响应

与 search 相同：非 POST 返回 405；body 解析失败或 query 解析失败返回 4xx；`List` 执行失败返回 5xx。格式均为 `{"Error":"..."}`。

### 3.4 示例

```bash
# 列出所有仓库（Q 用 "true" 或恒真表达式）
curl -X POST -H "Content-Type: application/json" -d '{"Q":""}' 'http://127.0.0.1:6070/api/list'
```

注意：`Q` 为空时，`query.Parse("")` 的行为需视 zoekt/query 实现而定；测试代码中常见用 `{"Q":""}` 表示“列出全部”，实际语义以 `query.Parse` 及 zoekt 版本为准。

---

## 4. 与 HTML 搜索的「format=json」区别

`web/server.go` 中 `/search` 还提供 HTML 搜索：`serveSearch` 根据 URL 参数决定返回 HTML 还是 JSON：

- 当 `?format=json` 时，直接对“模板用”的 `ApiSearchResult` 做 `json.NewEncoder(w).Encode(result)`，即返回的是 **ResultInput / RepoListInput** 那套结构（见 `web/api.go`），用于页面渲染，和 `/api/search` 的 **zoekt.SearchResult** 不是同一套 schema。
- `/api/search` 由 `internal/json` 提供，请求为 POST + JSON body，响应为标准的 `{ "Result": *SearchResult }`。

因此：

- **机器调用、集成、与 zoekt 核心类型一致**：应使用 **POST /api/search** 与 **POST /api/list**。
- **仅希望网页版搜索接口直接出 JSON**：使用 GET/POST `/search?format=json`（仍走同一套 HTML 查询逻辑，只是响应格式为 JSON）。

---

## 5. 小结

| 端点 | 方法 | 请求体 | 响应体 | 说明 |
|------|------|--------|--------|------|
| `/api/search` | POST | `{ "Q", "RepoIDs?", "Opts?" }` | `{ "Result": SearchResult }` | 代码搜索，Q 必填，超时默认 20s |
| `/api/list` | POST | `{ "Q", "Opts?" }` | `{ "List": RepoList }` | 仓库列表，Q 为 repo 约束 |

实现集中在 `internal/json/json.go`，类型定义在根目录 `api.go`，RPC 在 `Server.RPC == true` 时通过 `http.StripPrefix("/api", zjson.JSONServer(...))` 挂载在 `/api/` 下。
