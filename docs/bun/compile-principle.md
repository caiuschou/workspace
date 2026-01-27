# Bun 编译原理

本文档基于 [oven-sh/bun](https://github.com/oven-sh/bun) 源码与官方文档，整理 `bun build --compile` 生成单文件可执行文件的原理，包括整体流程、.bun 节格式与运行时加载方式。与 [Claude Code 逆向分析](../claude-code/reverse-engineering.md) 中的 `.bun` 节结论可互相印证。

## 任务与范围

| 任务 | 状态 |
|------|------|
| 编译入口与整体流程 | 已完成 |
| Bundler → 可执行文件 的链路 | 已完成 |
| .bun 节二进制格式（StandaloneModuleGraph） | 已完成 |
| 运行时加载（fromExecutable） | 已完成 |

## 1. 编译入口与整体流程

### 1.1 用户侧入口

- **CLI**：`bun build ./cli.ts --compile --outfile mycli`
- **API**：`await Bun.build({ entrypoints: ['./app.js'], compile: true, outfile: './my-app' })`

`compile` 为 true 或传入 `compile: { target, outfile, ... }` 时，走「单文件可执行」路径；否则为普通打包。

### 1.2 编译链路概览

```
用户入口 (bun build --compile / Bun.build)
    │
    ▼
Bundler/Transpiler
    │ 解析、依赖解析、转译、可选的字节码生成
    │ 输出：output_files[]（entry、chunk、bytecode、sourcemap、embed 等）
    ▼
StandaloneModuleGraph.toBytes(allocator, prefix, output_files, format, exec_argv, flags)
    │ 将 output_files 序列化为一段自定义二进制（见下文「.bun 节格式」）
    ▼
StandaloneModuleGraph.toExecutable(...)
    │ 1) 取得「模板 exe」：本机 bun 或按 target 下载的跨平台 bun
    │ 2) inject(bytes, self_exe, windows_options, target)
    │    - Windows：复制模板 → PE 解析 → pe_file.addBunSection(bytes) → 写回
    │    - macOS：复制模板 → Mach-O 解析 → macho_file.writeSection(bytes) → 写回并签名
    │    - Linux：复制模板 → 在文件末尾追加 bytes + 8 字节长度
    │ 3) 将得到的临时 exe 移动到用户指定的 outfile
    ▼
单文件可执行文件（内含 .bun 节或尾部 module graph）
```

核心实现位于：

- **序列化与注入**：`src/StandaloneModuleGraph.zig`（`toBytes`、`inject`、`toExecutable`）
- **Windows .bun 节写入**：`src/pe.zig` 的 `PEFile.addBunSection(bytes, .strip_always)`
- **跨平台模板**：`src/compile_target.zig`、`StandaloneModuleGraph.download()` 等

## 2. Bundler 与 output_files

- **Bundler/Transpiler**：负责解析 TS/JS、解析 import、转译、tree-shaking、可选的 minify 与 **bytecode 生成**。
- **output_files**：每一维对应一个「产出」，例如：
  - `output_kind == .@"entry-point"` 的入口；
  - 其它 chunk（含 side: server/client）；
  - `output_kind == .bytecode` 的字节码块；
  - `output_kind == .sourcemap` 的 sourcemap；
  - 以及 `import … with { type: "file" }` 等产生的资源。

只有「会进入 standalone 的」文件会参与 `toBytes`（`output_kind.isFileInStandaloneMode()`）。字节码在写入时需按 128 字节对齐（offset % 128 == 120），以兼容 PE/Mach-O 节前 8 字节头导致的起始偏移。

## 3. .bun 节二进制格式（StandaloneModuleGraph）

`.bun` 节（或 Linux 尾部追加的 blob）的布局由 `StandaloneModuleGraph.toBytes` / `fromBytes` 定义，大致如下。

### 3.1 逻辑结构

- **前缀**：可选 `module_prefix`（如 `file:///src/`），与每个文件的 `dest_path` 拼成 `name`。
- **模块列表**：`CompiledModuleGraphFile[]`，每个元素包含：
  - `name`：StringPointer（offset + length 指向同一 blob 内字符串）
  - `contents`：源码或资源内容
  - `sourcemap`：可选，StringPointer
  - `bytecode`：可选，StringPointer（JSC 字节码）
  - `encoding`：latin1 / binary / utf8
  - `loader`：js/ts/json/file 等
  - `module_format`：none / esm / cjs
  - `side`：server / client
- **尾部**：
  - `Offsets`（见下）
  - 魔数 trailer：`"\n---- Bun! ----\n"`

### 3.2 Offsets 与 trailer

```zig
// 来自 StandaloneModuleGraph.zig
pub const Offsets = extern struct {
    byte_count: usize,
    modules_ptr: bun.StringPointer,   // 指向 CompiledModuleGraphFile[]
    entry_point_id: u32,
    compile_exec_argv_ptr: bun.StringPointer,
    flags: Flags,
};
const trailer = "\n---- Bun! ----\n";
```

- 整段数据的有效长度为 `Offsets.byte_count`。
- `fromBytes` / 运行时加载时，从**尾部**向前解析：先找 `trailer`，再读 `Offsets`，再用 `modules_ptr` 与 `byte_count` 确定模块列表与各 StringPointer 的基址。

### 3.3 字节码对齐

字节码在 toBytes 里按「(current_offset + padding) % 128 == 120」做对齐，以适配「节基址 + 8 字节头」后仍满足 JSC 对 128 字节对齐的要求（见源码注释中的 PLATFORM-SPECIFIC ALIGNMENT）。

### 3.4 Windows 下 .bun 节的写入

- `StandaloneModuleGraph.inject()` 在 Windows 分支中调用 `bun.pe.PEFile.addBunSection(bytes, .strip_always)`。
- `addBunSection` 会在当前 PE 中新增一节名为 **`.bun`**，将 `bytes` 作为该节的 raw data 写入；`strip_always` 表示重写/剥离原有签名以便注入。
- 因此，你看到的 `claude.exe` 中「.bun 节」的内容，就是上述 `toBytes` 产出的同一格式；节内靠前仍可能是自定义头或对齐填充，再往后才是模块列表与各 StringPointer 指向的字符串/字节码等。

## 4. 运行时加载（fromExecutable）

### 4.1 何时加载

- 若进程由「自带 bundle 的」可执行文件启动（且未设 `BUN_BE_BUN=1`），运行时会尝试把**当前 exe** 当作 standalone，并调用 `StandaloneModuleGraph.fromExecutable(allocator)`。

### 4.2 按平台取数

- **Windows**：通过导出 `Bun__getStandaloneModuleGraphPELength` / `Bun__getStandaloneModuleGraphPEData`，从 PE 的 `.bun` 节取回整块数据（通常由加载器在进程启动时映射）。
- **macOS**：通过 `Bun__getStandaloneModuleGraphMachoLength` 等从 Mach-O 对应段取回。
- **Linux**：从 `/proc/self/exe` 或等价路径打开当前 exe，在**文件末尾**读取「最后 8 字节」得到 module graph 长度，再向前读对应字节数，最后按 trailer + Offsets 解析。

### 4.3 解析与挂载

- 得到 `raw_bytes` 后，用尾部的 `Offsets` 和 `fromBytes(allocator, raw_bytes, offsets)` 反序列化为 `StandaloneModuleGraph`，并 `set()` 为全局实例。
- 之后，`require`/`import` 在解析到 `/$bunfs/...`（或 Windows 上 `B:\~BUN\...`）这类路径时，会从 `StandaloneModuleGraph` 中按 name 查文件，不再走磁盘。

### 4.4 与 BUN_BE_BUN=1 的关系

- 设置 `BUN_BE_BUN=1` 时，逻辑上「不把自己当 standalone」：不加载当前 exe 的 bundle，因此 `StandaloneModuleGraph.fromExecutable` 不会挂上 Claude 的模块图，`Bun.embeddedFiles` 也为空。这与 [逆向报告 8.2](../claude-code/reverse-engineering.md#82-运行时方式已验证不可行) 的结论一致。

## 5. 与 Claude Code（claude.exe）的对应

- **claude.exe** 是将「Claude Code CLI」用同一套 `bun build --compile`（或等价的 Bun.build compile 流程）打出来的单文件可执行文件。
- 其 PE 中的 **.bun 节** 即为上述格式：由 `StandaloneModuleGraph.toBytes` 产出，再经 `pe_file.addBunSection` 写入。
- 节内逻辑入口由 `Offsets.entry_point_id` 指出，对应到 `output_files` 里 `output_kind == .@"entry-point"` 的那一项；从字符串可见虚拟路径为 `file:///src/entrypoints/cli.js.jsc`，即字节码形式的 CLI 入口。
- 若要对 .bun 做「解包」或解析，需要按照 `StandaloneModuleGraph` 的 `toBytes`/`fromBytes` 与 `CompiledModuleGraphFile`、`Offsets`、`trailer` 的布局自写解析器，或直接复用/编译 Bun 源码中的 `StandaloneModuleGraph.fromBytes`。

## 6. 小结表

| 阶段 | 位置 | 作用 |
|------|------|------|
| 入口 | CLI / Bun.build | `compile: true` 或 `--compile` |
| 打包 | Bundler/Transpiler | 产出 output_files（含 entry、bytecode、sourcemap、embed） |
| 序列化 | StandaloneModuleGraph.toBytes | 生成 .bun 节内存映像（Offsets + 模块列表 + trailer） |
| 注入 | inject → pe.addBunSection / macho.writeSection / 尾部追加 | 写入模板 exe，得到最终可执行文件 |
| 运行时 | fromExecutable → fromBytes | 从 .bun 节或文件尾读回，挂到 StandaloneModuleGraph，供 require/import |

## 7. 参考

- [oven-sh/bun](https://github.com/oven-sh/bun)（main）
- `src/StandaloneModuleGraph.zig`：toBytes、inject、toExecutable、fromExecutable、fromBytes
- `src/pe.zig`：PEFile、addBunSection
- `src/compile_target.zig`：目标平台与模板 exe 路径
- [Bun 官方文档 - Single-file executable](https://bun.sh/docs/bundler/executables)
- [Claude Code 逆向分析](../claude-code/reverse-engineering.md)
