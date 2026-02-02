#!/usr/bin/env python3
"""
Generate per-endpoint API documentation from OpenCode serve OpenAPI spec.
Usage: python gen-opencode-api-docs.py [openapi.json path or URL]
Default: fetches from http://host.docker.internal:34917/doc
"""

import json
import sys
import urllib.request
from pathlib import Path


def load_spec(source):
    if source.startswith("http://") or source.startswith("https://"):
        with urllib.request.urlopen(source, timeout=10) as r:
            return json.loads(r.read().decode())
    with open(source, "r", encoding="utf-8") as f:
        return json.load(f)


def schema_type_str(schema):
    if not schema:
        return "any"
    if "$ref" in schema:
        return schema["$ref"].split("/")[-1]
    if "type" in schema:
        t = schema["type"]
        if t == "array":
            items = schema.get("items", {})
            if "$ref" in items:
                return f"array<{items['$ref'].split('/')[-1]}>"
            return "array"
        if t == "object":
            return "object"
        return t
    if "anyOf" in schema:
        refs = [x["$ref"].split("/")[-1] for x in schema["anyOf"] if "$ref" in x]
        return " | ".join(refs) if refs else "anyOf"
    return "any"


def describe_param(p):
    name = p.get("name", "?")
    loc = p.get("in", "?")
    req = "**必填**" if p.get("required") else "可选"
    schema = p.get("schema", {})
    typ = schema_type_str(schema)
    desc = (p.get("description") or "").strip()
    return f"| {loc} | `{name}` | {typ} | {req} | {desc} |"


def describe_body_props(schema, components, prefix=""):
    lines = []
    if "$ref" in schema:
        ref = schema["$ref"]
        key = ref.split("/")[-1]
        if key in components.get("schemas", {}):
            schema = components["schemas"][key]
    props = schema.get("properties", {})
    required = set(schema.get("required", []))
    for k, v in props.items():
        req = "**必填**" if k in required else "可选"
        typ = schema_type_str(v)
        desc = (v.get("description") or "").strip()
        pat = v.get("pattern", "")
        if pat:
            desc = f"{desc} (pattern: `{pat}`)" if desc else f"pattern: `{pat}`"
        lines.append(f"| `{prefix}{k}` | {typ} | {req} | {desc} |")
    return lines


def describe_responses(responses, components):
    lines = []
    for code, r in responses.items():
        desc = (r.get("description") or "").strip()
        content = r.get("content", {})
        schema_ref = ""
        for ct, media in content.items():
            sch = media.get("schema", {})
            if "$ref" in sch:
                schema_ref = sch["$ref"].split("/")[-1]
            elif sch.get("type"):
                schema_ref = sch.get("type", "")
            if schema_ref:
                lines.append(f"| {code} | {desc} | `{schema_ref}` |")
                break
        if not schema_ref:
            lines.append(f"| {code} | {desc} | - |")
    return lines


def gen_operation_md(path, method, op, components):
    method = method.upper()
    op_id = op.get("operationId", "")
    summary = (op.get("summary") or "").strip()
    desc = (op.get("description") or "").strip()

    md = []
    md.append(f"### `{method} {path}`")
    md.append("")
    md.append(f"- **OperationId**: `{op_id}`")
    md.append(f"- **摘要**: {summary}")
    if desc:
        md.append(f"- **说明**: {desc}")
    md.append("")

    params = op.get("parameters", [])
    if params:
        md.append("**请求参数**")
        md.append("")
        md.append("| 位置 | 参数名 | 类型 | 必填 | 说明 |")
        md.append("|------|--------|------|------|------|")
        for p in params:
            md.append(describe_param(p))
        md.append("")

    body = op.get("requestBody")
    if body:
        content = body.get("content", {})
        for ct, media in content.items():
            schema = media.get("schema", {})
            md.append(f"**请求体** (`{ct}`)")
            md.append("")
            md.append("| 字段 | 类型 | 必填 | 说明 |")
            md.append("|------|------|------|------|")
            for line in describe_body_props(schema, components):
                md.append(line)
            md.append("")

    responses = op.get("responses", {})
    if responses:
        md.append("**响应**")
        md.append("")
        md.append("| 状态码 | 说明 | 类型/引用 |")
        md.append("|--------|------|-----------|")
        for line in describe_responses(responses, components):
            md.append(line)
        md.append("")

    return "\n".join(md)


# 模块 SDK 实现状态：完成 | 部分 | 未完成（基于 opencode-sdk）
MODULE_SDK_STATUS = {
    "01-global": "部分",    # health 已实现
    "08-session": "部分",   # create, prompt_async, message, diff 已实现
    "12-file": "部分",      # list, status 已实现
    "17-event": "完成",     # subscribe 已实现
}

# 各接口 SDK 实现状态：(method, path) -> 已实现 | 未实现
# path 为 OpenAPI 格式，如 /session/{sessionID}/message
ENDPOINT_SDK_STATUS = {
    ("get", "/global/health"): "已实现",
    ("get", "/event"): "已实现",
    ("post", "/session"): "已实现",
    ("post", "/session/{sessionID}/prompt_async"): "已实现",
    ("get", "/session/{sessionID}/message"): "已实现",
    ("get", "/session/{sessionID}/diff"): "已实现",
    ("get", "/file"): "已实现",
    ("get", "/file/status"): "已实现",
}


# 模块顺序与文件名（用于拆分文档、方便查找）
GROUP_ORDER = [
    ("01-global", "Global 全局"),
    ("02-instance", "Instance 实例"),
    ("03-project", "Project 项目"),
    ("04-path-vcs", "Path & VCS"),
    ("05-config", "Config 配置"),
    ("06-provider", "Provider 模型提供商"),
    ("07-auth", "Auth 认证"),
    ("08-session", "Session 会话 / Message 消息"),
    ("09-permission", "Permission 权限"),
    ("10-question", "Question 问题"),
    ("11-command", "Command 命令"),
    ("12-file", "File 文件"),
    ("13-find", "Find 查找"),
    ("14-lsp-formatter-mcp", "LSP / Formatter / MCP"),
    ("15-agent-skill", "Agent & Skill"),
    ("16-logging", "Logging 日志"),
    ("17-event", "Event 事件"),
    ("18-pty", "PTY 伪终端"),
    ("19-tui", "TUI 界面控制"),
    ("20-experimental", "Experimental 实验性"),
    ("21-docs", "Docs"),
    ("22-other", "Other"),
]

DISPLAY_TO_SLUG = {name: slug for slug, name in GROUP_ORDER}


def group_path(path):
    """Return display name for grouping (must match GROUP_ORDER names)."""
    parts = path.strip("/").split("/")
    if not parts:
        return "Other"
    first = parts[0]
    if first == "global":
        return "Global 全局"
    if first == "instance":
        return "Instance 实例"
    if first == "project":
        return "Project 项目"
    if first in ("path", "vcs"):
        return "Path & VCS"
    if first == "config":
        return "Config 配置"
    if first == "provider":
        return "Provider 模型提供商"
    if first == "auth":
        return "Auth 认证"
    if first == "session":
        return "Session 会话 / Message 消息"
    if first == "permission":
        return "Permission 权限"
    if first == "question":
        return "Question 问题"
    if first == "command":
        return "Command 命令"
    if first == "file":
        return "File 文件"
    if first == "find":
        return "Find 查找"
    if first in ("lsp", "formatter", "mcp"):
        return "LSP / Formatter / MCP"
    if first == "agent":
        return "Agent & Skill"
    if first == "skill":
        return "Agent & Skill"
    if first == "log":
        return "Logging 日志"
    if first == "event":
        return "Event 事件"
    if first == "pty":
        return "PTY 伪终端"
    if first == "tui":
        return "TUI 界面控制"
    if first == "experimental":
        return "Experimental 实验性"
    if first == "doc":
        return "Docs"
    return "Other"


def main():
    source = sys.argv[1] if len(sys.argv) > 1 else "http://host.docker.internal:34917/doc"
    spec = load_spec(source)
    paths = spec.get("paths", {})
    components = spec.get("components", {})

    # Collect (group_display, path, method, op)
    ops = []
    for path, path_item in sorted(paths.items()):
        group = group_path(path)
        for method in ("get", "post", "put", "patch", "delete"):
            op = path_item.get(method)
            if op:
                ops.append((group, path, method, op))

    # Group by display name
    from itertools import groupby
    by_display = {}
    for group, group_ops in groupby(sorted(ops, key=lambda x: (x[0], x[1], x[2])), key=lambda x: x[0]):
        by_display[group] = list(group_ops)

    # Build (slug, display_name, ops) in GROUP_ORDER
    ordered = []
    for slug, display_name in GROUP_ORDER:
        if display_name in by_display:
            ordered.append((slug, display_name, by_display[display_name]))
    for display_name, group_ops in by_display.items():
        if display_name not in DISPLAY_TO_SLUG:
            ordered.append(("22-other", display_name, group_ops))

    out_dir = Path(__file__).resolve().parent.parent / "docs" / "opencode-serve-api"
    out_dir.mkdir(parents=True, exist_ok=True)

    # README: 索引 + 各模块快速查找表
    readme = []
    readme.append("# OpenCode Serve API 文档")
    readme.append("")
    readme.append("按模块拆分的接口文档，便于按功能查找。")
    readme.append("")
    readme.append("## 通用说明")
    readme.append("")
    readme.append("- 多数接口支持 **query 参数 `directory`**，用于指定工作目录；未传时使用当前实例的 cwd。")
    readme.append("- 每个接口文档包含：OperationId、摘要、说明、请求参数表、请求体表（如有）、响应表。")
    readme.append("")
    readme.append("## 重新生成")
    readme.append("")
    readme.append("```bash")
    readme.append("python scripts/gen-opencode-api-docs.py [URL或openapi.json路径]")
    readme.append("```")
    readme.append("")
    readme.append("默认从 `http://host.docker.internal:34917/doc` 拉取 OpenAPI。")
    readme.append("")
    readme.append("---")
    readme.append("")
    readme.append("## 按主题")
    readme.append("")
    readme.append("| 主题 | 说明 |")
    readme.append("|------|------|")
    readme.append("| [实时接口](21-realtime.md) | SSE 事件流、WebSocket、流式 AI 响应等实时能力汇总 |")
    readme.append("")
    readme.append("---")
    readme.append("")
    readme.append("## 目录（按模块）")
    readme.append("")
    readme.append("表格四列：**类别**（链接到该模块文档）、**接口**（METHOD 路径）、**描述**（接口用途）、**状态**（opencode-sdk 是否已实现）。")
    readme.append("")
    readme.append("| 类别 | 接口 | 描述 | 状态 |")
    readme.append("|------|------|------|------|")

    # 描述映射：OpenAPI 摘要 -> 简短中文（可选覆盖）
    DESC_OVERRIDE = {
        ("get", "/global/health"): "获取服务健康与版本",
        ("get", "/global/event"): "订阅全局事件流（SSE）",
        ("post", "/global/dispose"): "销毁所有 OpenCode 实例",
        ("post", "/instance/dispose"): "销毁当前实例，释放资源",
        ("get", "/project"): "列出已打开的项目",
        ("get", "/project/current"): "获取当前活动项目",
        ("patch", "/project/{projectID}"): "更新项目（name、icon、commands）",
        ("get", "/path"): "获取当前工作目录与路径信息",
        ("get", "/vcs"): "获取版本控制信息（如 git 分支）",
        ("get", "/config"): "获取当前配置",
        ("patch", "/config"): "更新配置",
        ("get", "/config/providers"): "列出 providers 与默认模型",
        ("get", "/provider"): "列出所有可用/已连接提供商",
        ("get", "/provider/auth"): "获取各提供商认证方式",
        ("post", "/provider/{providerID}/oauth/authorize"): "发起 OAuth 授权，获取授权 URL",
        ("post", "/provider/{providerID}/oauth/callback"): "处理 OAuth 回调",
        ("put", "/auth/{providerID}"): "设置某提供商的认证凭据",
        ("get", "/session"): "列出会话（可按目录、标题、时间等筛选）",
        ("post", "/session"): "创建新会话",
        ("get", "/session/status"): "获取所有会话状态（active/idle/completed）",
        ("get", "/session/{sessionID}"): "获取会话详情",
        ("delete", "/session/{sessionID}"): "删除会话及全部数据",
        ("patch", "/session/{sessionID}"): "更新会话（如 title）",
        ("get", "/session/{sessionID}/children"): "获取从该会话 fork 出的子会话",
        ("get", "/session/{sessionID}/todo"): "获取会话待办列表",
        ("post", "/session/{sessionID}/init"): "初始化会话（分析项目并生成 AGENTS.md）",
        ("post", "/session/{sessionID}/fork"): "在指定消息处 fork 出新会话",
        ("post", "/session/{sessionID}/abort"): "中止正在运行的会话",
        ("post", "/session/{sessionID}/share"): "创建可分享链接",
        ("delete", "/session/{sessionID}/share"): "取消分享",
        ("get", "/session/{sessionID}/diff"): "获取某条消息导致的文件变更",
        ("post", "/session/{sessionID}/summarize"): "用 AI 总结会话",
        ("get", "/session/{sessionID}/message"): "获取会话内消息列表",
        ("post", "/session/{sessionID}/message"): "发送消息并流式返回 AI 响应",
        ("post", "/session/{sessionID}/prompt_async"): "异步发送消息，立即返回",
        ("get", "/session/{sessionID}/message/{messageID}"): "获取单条消息详情",
        ("delete", "/session/{sessionID}/message/{messageID}/part/{partID}"): "删除消息中的某 part",
        ("patch", "/session/{sessionID}/message/{messageID}/part/{partID}"): "更新消息中的某 part",
        ("post", "/session/{sessionID}/revert"): "回滚某条消息",
        ("post", "/session/{sessionID}/unrevert"): "恢复所有已回滚消息",
        ("post", "/session/{sessionID}/permissions/{permissionID}"): "批准或拒绝权限请求",
        ("post", "/session/{sessionID}/command"): "向会话发送命令由 AI 执行",
        ("post", "/session/{sessionID}/shell"): "在会话上下文中执行 shell 命令",
        ("get", "/permission"): "列出待处理权限请求",
        ("post", "/permission/{requestID}/reply"): "批准或拒绝权限请求",
        ("get", "/question"): "列出待回答问题",
        ("post", "/question/{requestID}/reply"): "回答问题",
        ("post", "/question/{requestID}/reject"): "拒绝问题",
        ("get", "/command"): "列出所有可用命令",
        ("get", "/file"): "列出指定路径下的文件与目录",
        ("get", "/file/content"): "读取文件内容",
        ("get", "/file/status"): "获取项目内文件 git 状态",
        ("get", "/find"): "文本搜索（ripgrep）",
        ("get", "/find/file"): "按名称或模式搜索文件/目录",
        ("get", "/find/symbol"): "符号搜索（LSP）",
        ("get", "/lsp"): "获取 LSP 服务状态",
        ("get", "/formatter"): "获取 Formatter 状态",
        ("get", "/mcp"): "获取 MCP 服务状态",
        ("post", "/mcp"): "动态添加 MCP 服务",
        ("post", "/mcp/{name}/auth"): "启动 MCP OAuth",
        ("delete", "/mcp/{name}/auth"): "移除 MCP OAuth 凭据",
        ("post", "/mcp/{name}/auth/authenticate"): "启动 MCP OAuth 并等待回调",
        ("post", "/mcp/{name}/auth/callback"): "完成 MCP OAuth 回调",
        ("post", "/mcp/{name}/connect"): "连接 MCP 服务",
        ("post", "/mcp/{name}/disconnect"): "断开 MCP 服务",
        ("get", "/agent"): "列出可用 AI 代理",
        ("get", "/skill"): "列出可用技能",
        ("post", "/log"): "向服务端写日志条目",
        ("get", "/event"): "订阅服务端事件流（SSE）",
        ("get", "/pty"): "列出 PTY 会话",
        ("post", "/pty"): "创建 PTY 会话",
        ("get", "/pty/{ptyID}"): "获取 PTY 会话详情",
        ("put", "/pty/{ptyID}"): "更新 PTY 会话",
        ("delete", "/pty/{ptyID}"): "移除并终止 PTY 会话",
        ("get", "/pty/{ptyID}/connect"): "建立 WebSocket 连接与 PTY 交互",
        ("post", "/tui/append-prompt"): "向输入框追加内容",
        ("post", "/tui/clear-prompt"): "清空输入框",
        ("post", "/tui/submit-prompt"): "提交当前输入",
        ("post", "/tui/open-help"): "打开帮助对话框",
        ("post", "/tui/open-sessions"): "打开会话选择对话框",
        ("post", "/tui/open-models"): "打开模型选择对话框",
        ("post", "/tui/open-themes"): "打开主题选择对话框",
        ("post", "/tui/execute-command"): "执行 TUI 命令",
        ("post", "/tui/show-toast"): "显示 toast 通知",
        ("post", "/tui/select-session"): "切换到指定会话",
        ("post", "/tui/publish"): "发布 TUI 事件",
        ("get", "/tui/control/next"): "获取下一个 TUI 控制请求",
        ("post", "/tui/control/response"): "提交对 TUI 控制请求的响应",
        ("get", "/experimental/tool/ids"): "列出所有工具 ID",
        ("get", "/experimental/tool"): "获取某 provider+model 的工具列表",
        ("get", "/experimental/resource"): "获取 MCP 资源",
        ("get", "/experimental/worktree"): "列出 worktree",
        ("post", "/experimental/worktree"): "创建 worktree",
        ("delete", "/experimental/worktree"): "删除 worktree",
        ("post", "/experimental/worktree/reset"): "重置 worktree 分支",
    }

    for slug, display_name, group_ops in ordered:
        for _, path, method, op in group_ops:
            key = (method.lower(), path)
            desc = DESC_OVERRIDE.get(key) or (op.get("summary") or op.get("description") or "-")[:80]
            status = ENDPOINT_SDK_STATUS.get(key, "未实现")
            readme.append(f"| [{display_name}]({slug}.md) | `{method.upper()} {path}` | {desc} | {status} |")

    (out_dir / "README.md").write_text("\n".join(readme), encoding="utf-8")
    print(f"Written: {out_dir / 'README.md'}")

    # 各模块单独文件
    for slug, display_name, group_ops in ordered:
        part = []
        part.append(f"# {display_name}")
        part.append("")
        part.append(f"> [← 返回目录](README.md)")
        part.append("")
        part.append("---")
        part.append("")
        for _, path, method, op in group_ops:
            part.append(gen_operation_md(path, method, op, components))
            part.append("---")
            part.append("")
        path = out_dir / f"{slug}.md"
        path.write_text("\n".join(part), encoding="utf-8")
        print(f"Written: {path}")

    # 生成 docs/opencode-serve-api.md 短索引（指向已拆分模块）
    single_path = Path(__file__).resolve().parent.parent / "docs" / "opencode-serve-api.md"
    single = []
    single.append("# OpenCode Serve API 文档")
    single.append("")
    single.append("> **接口已按模块拆分**，请使用下方链接查找。")
    single.append("")
    single.append("完整目录与接口列表：[opencode-serve-api/README.md](opencode-serve-api/README.md)")
    single.append("")
    single.append("## 模块索引")
    single.append("")
    single.append("> **状态**：基于 opencode-sdk 的实现情况。完成 = 主要接口已实现；部分 = 部分接口已实现；未完成 = 暂未实现。")
    single.append("")
    single.append("| 模块 | 文档 | 状态 |")
    single.append("|------|------|------|")
    for slug, display_name, _ in ordered:
        status = MODULE_SDK_STATUS.get(slug, "未完成")
        single.append(f"| {display_name} | [{slug}.md](opencode-serve-api/{slug}.md) | {status} |")
    single.append("")
    single.append("## 重新生成")
    single.append("")
    single.append("```bash")
    single.append("python scripts/gen-opencode-api-docs.py [URL或openapi.json路径]")
    single.append("```")
    single.append("")
    single_path.write_text("\n".join(single), encoding="utf-8")
    print(f"Written: {single_path} (index)")


if __name__ == "__main__":
    main()
