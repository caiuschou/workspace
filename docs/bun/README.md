# Bun 相关文档

本目录存放与 [Bun](https://bun.sh) 运行时/构建器相关的技术文档，侧重于可执行文件编译与格式。

## 文档列表

| 文档 | 描述 |
|------|------|
| [compile-principle.md](compile-principle.md) | `bun build --compile` 的编译原理：整体流程、.bun 节格式（StandaloneModuleGraph）、运行时加载 |
| [decompile-tooling.md](decompile-tooling.md) | 反编译与解包工具：现有工具、解包 vs 反编译边界、解包层实现思路与最小解析步骤、JSC 字节码注意点 |

## 相关

- [Claude Code 逆向分析](../claude-code/reverse-engineering.md)：对基于 Bun 打包的 `claude.exe` 的 PE/.bun 节逆向结论，与本文档中的格式描述一致。
