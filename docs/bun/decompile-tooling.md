# Bun 反编译与解包工具

本文档基于 [Bun 编译原理](compile-principle.md) 与 [Claude Code 逆向分析](../claude-code/reverse-engineering.md)，整理「从 `bun build --compile` 产出的单文件可执行文件中解包 / 反编译」所需的知识与工具现状，并给出实现路线与最小可用解析思路。解包（.bun → 模块列表 + 内容）与反编译（JSC 字节码 → JS 源码）分两层论述。

## 任务与范围

| 任务 | 状态 |
|------|------|
| 现有/相关工具调研 | 已完成 |
| 解包 vs 反编译的目标与边界 | 已完成 |
| 解包层实现思路与最小解析步骤 | 已完成 |
| JSC 字节码反编译现状与注意点 | 已完成 |

## 1. 现有与相关工具

### 1.1 针对「Bun 单文件 exe / .bun 节」的解包工具

- **结论**：未有面向「从 Bun 单文件可执行文件解包 .bun 节、解析 StandaloneModuleGraph」的现成独立工具。
- Bun 官方未提供 `bun unpack` 或类似 CLI；社区可见的「bun unpack」多为通用解压工具，与 .bun 节格式无直接关系。
- 可行做法是自实现：按 PE/Mach-O/ELF 取到 .bun 裸数据后，按 [编译原理](compile-principle.md) 中的格式（StandaloneModuleGraph.toBytes/fromBytes、Offsets、trailer）写解析器，或复用/编译 Bun 源码中的 `StandaloneModuleGraph.fromBytes`。

### 1.2 针对「从 exe 取出 .bun 节裸数据」

- **Windows (PE)**：按 PE 节表找到名为 `.bun` 的节，取其 `PointerToRawData` 与 `SizeOfRawData`，从文件中读出该段即可。可参考 [逆向报告](../claude-code/reverse-engineering.md) 中的节表与偏移，或用任意 PE 解析库/脚本实现。
- **Linux**：可执行文件末尾 8 字节为 module graph 长度（u64, little-endian），其前的若干字节即与 .bun 节同格式的 blob，需整段读入再按同一解析逻辑处理。
- **macOS (Mach-O)**：从对应段（Bun 写入的 section）中读取与 Windows .bun 节相同格式的 blob 即可。

### 1.3 针对「JSC 字节码」的工具

- **`jsc` / JavaScriptCore 相关**：
  - 存在 `jsc -d` 或等价选项，用于**反汇编**字节码（bytecode → 文本形 disassembly），不是「字节码 → 可运行 JS 源码」的反编译。
  - JSC 字节码格式与版本绑定，且随 WebKit/JSC 版本演进，Bun 所嵌 JSC 版本与上游可能略有差异，若要做通用反编译需考虑版本与格式兼容。
- **字节码 → JS 源码**：
  - 未见成熟的、可稳定将 JSC 字节码还原为 JS 源码的通用反编译器；多数工作停留在 disassembly 或部分还原，完整、可维护的「.jsc → .js」工具链尚未普及。
- **含义**：解包 .bun 后，对有 `bytecode` 的模块可：
  - 将字节码按二进制写出（如 `.jsc`），便于后续分析或尝试自研反编译；
  - 若同模块带有 `contents`（未编译为字节码的源码），可直接使用；
  - 若仅有 bytecode，则只能依赖 disassembly 或自研反编译，不能依赖现成「一键还原 JS」工具。

## 2. 解包与反编译的目标与边界

### 2.1 解包（Unpack）——建议实现的范畴

- **输入**：`bun build --compile` 产出的单文件可执行文件（或从其中抽取的 .bun 节/等价格式 blob）。
- **输出**：
  - 模块列表：每个模块的 `name`、`loader`、`module_format`、`side`、是否有 `contents`/`sourcemap`/`bytecode`；
  - 对每个模块按 `name` 写出：
    - `contents` → 源文件（如 .js/.ts/.json 等，按 loader 扩展名）；
    - `sourcemap` → 单独文件（如 `.map`）；
    - `bytecode` → 二进制文件（如 `.jsc`），并标明「需 JSC 反汇编/反编译才可读」。
- **依据**：Bun 的 `StandaloneModuleGraph.toBytes`/`fromBytes`、`CompiledModuleGraphFile`、`Offsets`、trailer 的布局，见 [编译原理 §3](compile-principle.md#3-bun-节二进制格式standalonemodulegraph) 与 [§5](compile-principle.md#5-与-claude-codeclaudeexe-的对应)。

### 2.2 反编译（Decompile）——单独、更高难度的一层

- **狭义反编译**：JSC 字节码（.jsc）→ 可读、可运行的 JS 源码。
- **现状**：无现成通用工具链；若要做，需结合 JSC 版本、字节码 opcode 与语义做逆向或仿写解释逻辑，与「解包 .bun」无直接依赖，可独立规划。
- **建议**：在「反编译工具」文档与实现中，将「解包 .bun → 得到模块 + contents/bytecode/sourcemap」与「JSC 字节码 → JS」拆开：先做好解包与目录导出，再视需要做 disassembly 或自研反编译。

## 3. 解包层实现思路

### 3.1 格式要点（与编译原理一致）

- **trailer**：`"\n---- Bun! ----\n"`（16 字节），用于在 blob 尾部定位。
- **Offsets**：紧接在 trailer 之前，为 `extern struct`，含：
  - `byte_count: usize`：整段有效数据长度；
  - `modules_ptr: bun.StringPointer`：指向 `CompiledModuleGraphFile[]` 的 (offset, length)；
  - `entry_point_id: u32`：入口在模块列表中的下标；
  - `compile_exec_argv_ptr: bun.StringPointer`、`flags` 等。
- **CompiledModuleGraphFile**：每个元素包含：
  - `name`, `contents`, `sourcemap`, `bytecode`：均为在同一 blob 内的 (offset, length)，即 `StringPointer`；
  - `encoding`, `loader`, `module_format`, `side` 等标量。
- **解析顺序**：从 blob **尾部**向前：先定位 trailer，再读 `Offsets`，再用 `modules_ptr` 得到 `CompiledModuleGraphFile[]`，最后用各 `StringPointer` 在 blob 内取片段。所有 (offset, length) 均相对**整段 blob 起始**。

具体字段顺序、对齐与 `StringPointer` 的位宽以 Bun 源码为准：`src/StandaloneModuleGraph.zig` 中的 `Offsets`、`CompiledModuleGraphFile` 及 `bun.StringPointer` / `Schema.StringPointer`。

### 3.2 最小可用解析流程（伪代码级）

1. **取得 .bun 裸数据 `blob`**
   - Windows：读 PE 节表，找到 `.bun` 节，按 `PointerToRawData`、`SizeOfRawData` 读入。
   - Linux：打开 exe， seek 到「文件长度 − 8」，读 8 字节为 `len`（u64 LE）；再 seek 到「文件长度 − 8 − len」，读 `len` 字节为 `blob`。
   - macOS：从 Mach-O 对应段读入与 Windows .bun 节同格式的 blob。

2. **在 blob 尾部定位 Offsets**
   - 在 `blob` 末尾找 16 字节 `"\n---- Bun! ----\n"`；其前的 `sizeof(Offsets)` 字节即 `Offsets`。
   - 按当前 Bun 使用的小端、指针宽度解析出 `byte_count`、`modules_ptr.offset`、`modules_ptr.length`、`entry_point_id` 等。

3. **解析模块列表**
   - `modules_raw = blob[modules_ptr.offset .. modules_ptr.offset + modules_ptr.length]`。
   - 按 `sizeof(CompiledModuleGraphFile)` 将 `modules_raw` 切为数组；若不能整除，说明格式或版本不匹配，需检查 Bun 版本与结构体定义。

4. **逐模块取出 name / contents / sourcemap / bytecode**
   - 对每个 `CompiledModuleGraphFile` 元素，依其 `name`、`contents`、`sourcemap`、`bytecode` 的 (offset, length) 在 `blob` 内取子串：`blob[o..][0..l]`（注意边界）。
   - 将 `name` 转为安全路径（去掉 `file:///`、`B:\~BUN\` 等前缀，或保留为相对路径），建立输出目录并写文件：
     - `contents` → 按 name 的扩展名或 loader 写出；
     - `sourcemap` → 同 name 加 `.map` 或单独目录；
     - `bytecode` → 同 name 加 `.jsc`，并在清单中标为「仅字节码，需反汇编/反编译」。

5. **入口与元数据**
   - `entry_point_id` 对应模块即逻辑入口（如 `file:///src/entrypoints/cli.js.jsc`），可在导出清单中标记。
   - `compile_exec_argv_ptr` 若非空，可按 StringPointer 从 blob 中取出并保存为元数据（例如 `compile_argv.txt`）。

### 3.3 复用 Bun 源码的实现方式

- 若使用 Zig 或能链接 Bun 的 C 接口：可直接调用 `StandaloneModuleGraph.fromBytes(allocator, raw_bytes, offsets)`，得到 `StandaloneModuleGraph` 后遍历 `files`，按 `File.name`、`File.contents`、`File.sourcemap`、`File.bytecode` 写出。此时无需手写 Offsets/CompiledModuleGraphFile 的二进制布局，但需与 Bun 构建与 ABI 保持一致。
- 若使用其他语言：需按上述 3.2 的步骤，根据 Bun 源码中的 `Offsets`、`CompiledModuleGraphFile`、`StringPointer` 定义，用该语言实现等价的二进制解析与目录导出。

### 3.4 本仓库实现

工作区内 [tools/bun-unpack](../../tools/bun-unpack/) 为 Rust 实现的解包工具，支持：

- **输入**：Bun 单文件可执行文件（PE / ELF / Mach-O），或已抽取的 .bun 裸数据（`--blob`）。
- **输出**：按模块虚拟路径写出 `contents`、`sourcemap`（同路径加 `.map`）、`bytecode`（同路径加 `.jsc`），并生成 `manifest.txt` 列出入口与模块列表。

构建与用法见 [tools/bun-unpack/README.md](../../tools/bun-unpack/README.md)。

## 4. JSC 字节码反编译的注意点

- **格式与版本**：Bun 内嵌 JavaScriptCore，其字节码格式随 JSC 版本变化；与「.bun 解包」解耦后，反编译需单独考虑 JSC 的 opcode 集合、常量表、函数元数据等。
- **Disassembly 与 Decompile**：现有能力以「反汇编为文本」为主（如 `jsc -d` 或 WebKit 内工具），「反编译为等价 JS 源码」仍属研究/自研范畴，无法作为解包工具的默认依赖。
- **工程建议**：在「反编译工具」的范围内，优先完成并文档化「解包 .bun → 模块 + contents/bytecode/sourcemap 导出」；对纯 bytecode 模块，在文档中注明需另行处理，并可选提供「导出为 .jsc + 调用现有 disassembler」的示例或链接，便于后续扩展反编译链。

## 5. 小结表

| 层次 | 目标 | 现有工具 | 建议 |
|------|------|----------|------|
| 从 exe 取 .bun | 得到与平台无关的 blob | 无专用工具，需自写或脚本 | 按 PE/Mach-O/ELF 规范读节或文件尾 |
| 解析 StandaloneModuleGraph | 模块列表 + name/contents/sourcemap/bytecode | 无独立工具 | 按 toBytes/fromBytes 布局自实现，或复用 Bun 的 fromBytes |
| 导出到目录 | 按 name 写 contents/sourcemap/bytecode | — | 在解包层实现；bytecode 标为 .jsc 并注明需反汇编/反编译 |
| JSC 字节码 → JS | 可读、可运行源码 | 无成熟通用反编译器 | 与解包分开规划；可先做 disassembly，反编译需自研或单独项目 |

## 6. 参考

- [Bun 编译原理](compile-principle.md)：整体流程、.bun 节格式、StandaloneModuleGraph、运行时 fromExecutable/fromBytes
- [Claude Code 逆向分析](../claude-code/reverse-engineering.md)：claude.exe 的 PE/.bun 节、入口路径、提取 .bun 裸数据的方式
- [oven-sh/bun](https://github.com/oven-sh/bun) 中 `src/StandaloneModuleGraph.zig`：`Offsets`、`CompiledModuleGraphFile`、`fromBytes`、`toBytes`、`trailer`、各平台 `fromExecutable` 取数逻辑
- [tools/bun-unpack](../../tools/bun-unpack/)：本仓库内的 Rust 解包实现，支持 PE/ELF/Mach-O 与 `--blob`
