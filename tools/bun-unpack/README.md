# bun-unpack

从 Bun 单文件可执行文件中解包 `.bun` 节，按 [StandaloneModuleGraph](https://github.com/oven-sh/bun/blob/main/src/StandaloneModuleGraph.zig) 格式解析，将各模块的 `name` / `contents` / `sourcemap` / `bytecode` 写出到目录。

格式与流程见 [docs/bun/compile-principle.md](../../docs/bun/compile-principle.md) 与 [docs/bun/decompile-tooling.md](../../docs/bun/decompile-tooling.md)。

## 用法

```bash
# 从可执行文件解包（PE / ELF / Mach-O）
cargo run -p bun-unpack -- path/to/claude.exe
cargo run -p bun-unpack -- path/to/claude.exe -o ./out

# 从已抽取的 .bun 裸数据解包
cargo run -p bun-unpack -- --blob section.bin -o ./out
```

- **输入**：Bun `bun build --compile` 产出的可执行文件，或从其中抽出的 `.bun` 裸数据（`--blob`）。
- **输出**：默认 `./unpacked`，可用 `-o/--output` 指定。每个模块按虚拟路径写出 `contents`，`sourcemap` 为同路径加 `.map`，`bytecode` 为同路径加 `.jsc`；入口与模块列表见 `manifest.txt`。

## 构建

需已安装 Rust 与（Windows 上）能用的 MSVC 或 GNU 工具链。若出现 `kernel32.lib` 等链接错误，请确认已安装对应 Windows SDK 或使用 `x86_64-pc-windows-gnu` 等目标。

```bash
cargo build -p bun-unpack -r
# 或
cargo install --path tools/bun-unpack
```

## 支持格式

- **Windows (PE)**：在节表中查找名为 `.bun` 的节，读取其 raw data。
- **Linux (ELF)**：文件末 8 字节为 module graph 长度（u64 LE），其前若干字节为与 .bun 节同格式的 blob。
- **macOS (Mach-O)**：在末尾附近查找 trailer `"\n---- Bun! ----\n"`，据此截取 blob。
- **裸 blob**：`--blob <file>` 时直接按同一 StandaloneModuleGraph 布局解析，不区分平台。

## 布局说明

- Trailer：`\n---- Bun! ----\n`（16 字节）。
- Offsets：紧接 trailer 前，含 `byte_count`、`modules_ptr`(offset+length)、`entry_point_id` 等。
- 每个模块 36 字节（4×StringPointer + encoding/loader/module_format/side），StringPointer 为 u32 offset + u32 length，均相对于整段 blob。

若 Bun 升级导致布局变更，需对照 [StandaloneModuleGraph.zig](https://github.com/oven-sh/bun/blob/main/src/StandaloneModuleGraph.zig) 调整 `MODULE_STRUCT_SIZE` 与 `Offsets` 解析。
