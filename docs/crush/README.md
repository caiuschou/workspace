# Crush 文档

> Crush 是 OpenCode 的继任项目，由 Charm 团队与原作者维护，终端里的 AI 编程助手。

## 与 OpenCode 的关系

| 项目 | 状态 | 仓库 |
|------|------|------|
| **opencode-ai/opencode** | 已归档，不再维护 | https://github.com/opencode-ai/opencode |
| **charmbracelet/crush** | 活跃开发 | https://github.com/charmbracelet/crush |

- OpenCode 仓库 README 写明：*"This repository has been archived. The project has continued under the name **Crush**."*
- 同一作者（Kujtim Hoxha）将项目交给 Charm 团队，改名为 Crush；配置从 `.opencode.json` 改为 `crush.json`，能力延续并扩展（如 Agent Skills、MCP、LSP 等）。

## 本地源码

本文档基于工作区 `thirdparty/crush` 的代码整理。该目录被 `.gitignore`，需自行 clone：

```bash
cd thirdparty && git clone --depth 1 https://github.com/charmbracelet/crush.git crush
```

## 文档目录

| 文档 | 描述 |
|------|------|
| [Agent Skills](skills.md) | Agent Skills 开放标准在 Crush 中的实现：SKILL.md 解析、发现路径、与 prompt/View 工具的衔接 |

## 参考

- [Crush 官方 README](https://github.com/charmbracelet/crush)
- [Agent Skills 规范](https://agentskills.io)
- [OpenCode 文档](../opencode/README.md)（归档项目，可作历史对照）
