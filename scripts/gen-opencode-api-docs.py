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
    readme.append("## 目录（按模块）")
    readme.append("")

    for slug, display_name, group_ops in ordered:
        readme.append(f"### [{display_name}]({slug}.md)")
        readme.append("")
        for _, path, method, _ in group_ops:
            readme.append(f"- `{method.upper()} {path}`")
        readme.append("")

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

    # 保留单文件汇总（便于全文搜索）
    single = []
    single.append("# OpenCode Serve API 接口详细文档（汇总）")
    single.append("")
    single.append("> 本文件由脚本生成。**按模块查找请使用 [opencode-serve-api/README.md](opencode-serve-api/README.md) 目录。**")
    single.append("")
    single.append("---")
    single.append("")
    for slug, display_name, group_ops in ordered:
        single.append(f"## {display_name}")
        single.append("")
        for _, path, method, op in group_ops:
            single.append(gen_operation_md(path, method, op, components))
            single.append("---")
            single.append("")
    single_path = Path(__file__).resolve().parent.parent / "docs" / "opencode-serve-api.md"
    single_path.write_text("\n".join(single), encoding="utf-8")
    print(f"Written: {single_path} (single-file index)")


if __name__ == "__main__":
    main()
