# Claude Code 可执行文件逆向分析报告

本文档记录对当前环境中 Claude Code 可执行文件（`claude.exe`）的逆向分析结果，仅做结构与格式层面的理解，不涉及破解、脱壳或源码复现。

## 任务与范围

| 任务 | 状态 |
|------|------|
| 定位可执行文件与版本信息 | 已完成 |
| 解析 PE 结构及节表 | 已完成 |
| 识别运行时与嵌入格式（Bun / .bun） | 已完成 |
| 提取入口路径与特征字符串 | 已完成 |

## 1. 目标文件与版本

| 项 | 值 |
|----|-----|
| 路径 | `C:\Users\heycj\.local\bin\claude.exe` |
| 大小 | 约 231 MB（231,449,760 字节） |
| InternalName | `bun` |
| FileDescription / Product | Claude Code |
| ProductVersion / FileVersion | 2.1.7.0 |

结论：该 exe 在内部以「Bun」为运行时标识，实质是「Bun 运行时 + Claude Code 业务逻辑」的单一可执行包。

## 2. PE 结构概要

- **格式**：PE32+，x64（machine 0x8664），控制台子系统（CUI）。
- **入口**：RVA `0x143923F04`（落在 .text）。
- **主要节**（dumpbin 解析）：

| 节名 | 用途 | Raw Size（约） | File Offset（约） |
|------|------|----------------|-------------------|
| .text | 代码 | ~58 MB | 0x400 |
| .rdata | 只读数据 | ~730 MB | 0x398CC00 |
| .data | 可写数据 | ~104 KB | 0x6741000 |
| .pdata | 异常目录 | ~1 MB | 0x675A600 |
| .tls | TLS | ~4.9 MB | 0x68BE600 |
| .rsrc | 资源 | ~200 KB | 0x6D84800 |
| .reloc | 重定位 | ~180 KB | 0x6DB5600 |
| **.bun** | **Bun 嵌入包（字节码/资源）** | **~116 MB** | **0x6DE1E00** |

`.bun` 为最大数据节，从文件偏移 `0x6DE1E00` 开始，是 Bun 用来存放打包后代码与资源的专用段。

## 3. .bun 节与入口信息

### 3.1 节头与魔数

`.bun` 节起始 64 字节（十六进制前若干字节）：

```text
87 5D ED 06 00 00 00 00 C6 3E 97 CC A0 02 00 00 ...
```

属 Bun 自定义格式，非标准 PE 资源或常见压缩/打包魔数；紧跟其后存在大量可读 ASCII，包含路径与注释。

### 3.2 逻辑入口路径

在 `.bun` 内靠前区域可读字符串中可见：

```text
file:///src/entrypoints/cli.js.jsc
```

以及：

```text
// @bun @bytecode @bun-cjs
(function(exports, require, module, __filename, __dirname) {// Claude Code is a Beta product...
```

可得出：

- 逻辑入口为虚拟路径 **`/src/entrypoints/cli.js.jsc`**。
- 内容为 **Bun 字节码**（`.jsc`），形式为 **CommonJS 风格**（`exports, require, module, __filename, __dirname`），并带 `@bun @bytecode @bun-cjs` 标记。

即：`claude.exe` 启动时，由内嵌 Bun 运行时加载并执行该字节码，对应「Claude Code CLI」的入口。

## 4. 嵌入字符串中的其他信息

从 `.bun` 节内可读字符串中可归纳出以下类别（仅为所见内容归纳，不表示完整行为）：

### 4.1 版权与合规

- `Claude Code is a Beta product per Anthropic's Commercial Terms of Service.`
- `(c) Anthropic PBC. All rights reserved. Use is subject to the Legal Agreements outlined here: https://code.claude.com/docs/en/legal-and-compliance.`
- `By using Claude Code, you agree that all code acceptance or rejection decisions you make, ... constitute Feedback under Anthropic's Commercial Terms, and may be used to improve Anthropic's products, including training models.`

说明：产品归属 Anthropic PBC，合规与条款见 `code.claude.com/docs/en/legal-and-compliance`。

### 4.2 配置与错误提示

- `Claude configuration file at ${H} is corrupted: ${D.message}`
- `Claude configuration file not found at: ${H}`

说明：存在「Claude 配置文件」路径变量 `${H}`，损坏或未找到时有对应提示。

### 4.3 API 与运行时

- `new Anthropic({ apiKey, dangerouslyAllowBrowser: true });`  
  说明：使用 Anthropic 官方 SDK，传入 `apiKey`，并存在 `dangerouslyAllowBrowser: true` 的用法（多与浏览器环境或特殊打包场景相关）。
- `no longer support Node.js 16.x on January 6, 2025.` / `updates please upgrade to a supported Node.js LTS version.`  
  说明：对 Node 版本有要求，至少会提示放弃 Node 16 与升级 LTS。

### 4.4 功能与工具描述（面向用户的说明文本）

- `Allows Claude to search the web and use the results to inform responses`
- `This tool allows Claude Code to read images (eg PNG, JPG, etc). When reading an image file the contents are presented visually as Claude Code is a multimodal LLM.`
- `Use this tool for accessing information beyond Claude's knowledge cutoff`
- `All dependencies have been installed according to your package.json file.`
- 以及与「过滤文件 / glob / 语言类型」相关的说明（如 `Filter files with glob parameter (e.g., "*.js", "**/*.tsx")` 等）。

说明：CLI 内包含与「网页搜索」「读图」「知识截止」「依赖安装」「文件过滤」等能力相关的说明或逻辑入口。

## 5. 结构小结

```
claude.exe
├── PE 头与 .text / .rdata / .data / .pdata / .tls / .rsrc / .reloc
│   └── 标准 x64 原生代码与数据（Bun 运行时 + 启动逻辑）
└── .bun 节（file offset 0x6DE1E00，约 116 MB）
    └── Bun 自定义格式包
        └── 逻辑入口: file:///src/entrypoints/cli.js.jsc（Bun 字节码 @bun-cjs）
            └── Claude Code CLI 逻辑、配置、API 调用、工具说明等
```

- **可执行文件**：Bun 运行时 + 单一大包，无外部 Node 解释器依赖。
- **业务入口**：`/src/entrypoints/cli.js.jsc`，以 Bun 字节码形式存在，未以明文 JS 暴露。
- **合规与配置**：通过嵌入字符串可见 Anthropic 条款与本地配置文件相关逻辑的提示。

## 6. 局限与说明

- **未反编译字节码**：`.jsc` 为 Bun 专有格式，本分析未对其指令或数据结构做进一步逆向。
- **未提取或运行嵌入代码**：未尝试从 `.bun` 中抽离、修补或单独执行任何片段。
- **法律与合规**：本报告仅用于理解可执行文件结构与运行方式。任何对 Claude Code 的再分发、仿制或违反其服务条款的行为均与本文档无关，需自行遵守当地法律与 Anthropic 条款。

## 7. 参考与工具

- PE 解析：Microsoft `dumpbin.exe`（VC 自带）的 `/HEADERS`。
- 字符串与节内容：在 Windows 下对 `claude.exe` 按节偏移读取，并对 `.bun` 做 ASCII 字符串扫描。
- 版本与路径：本环境为 `C:\Users\heycj\.local\bin\claude.exe`，版本 2.1.7.0；其他安装路径或版本可能在不同偏移下存在差异。

## 8. 提取 Bun 打包内容的可行性

### 8.1 结论概要

**无法在不动 Bun 实现、不逆向其格式的前提下，从 `claude.exe` 里完整「解包」出文件列表与内容。** 可做的是：从 exe 中按 PE 节表切出 `.bun` 的裸数据，得到独立二进制块，供后续逆向或格式研究使用。

### 8.2 运行时方式（已验证不可行）

- 设置 `BUN_BE_BUN=1` 时，`claude.exe` 会表现为普通 Bun CLI（版本 1.3.5），不执行其内嵌的 Claude 入口。
- 在此模式下执行 `Bun.embeddedFiles`，得到 **length 0**。即：当前进程不会把 `claude.exe` 的嵌入包当作「本进程的嵌入文件」暴露，因此无法通过官方 API 列出或读取其中的嵌入资源。
- 官方文档中的 `Bun.embeddedFiles` / `Bun.file("$bunfs/...")` 仅对「正在执行自身 bundle 的可执行文件」有效；用 `BUN_BE_BUN=1` 跑 claude 时，运行的是 Bun 的默认行为，未加载 Claude 的 bundle，故看不到任何嵌入文件。

### 8.3 静态提取：切出 .bun 节

不解析内容、仅按 PE 节表把 `.bun` 段从 exe 里切出来，在本地是可行的。本环境中：

- **文件偏移**：`0x6DE1E00`
- **大小**：`0x6ED5E00` 字节（约 116 MB）

切出后得到的是一个名为「.bun 节裸数据」的二进制文件，内部是 Bun 自定义格式（含入口路径 `file:///src/entrypoints/cli.js.jsc`、字节码与可能的嵌入资源等），需配合 Bun 源码或逆向才能进一步解析出「文件列表 + 逐文件内容」。

以下为 PowerShell 示例（需将 `$exe` 换成你的 `claude.exe` 路径）：

```powershell
$exe  = "C:\Users\heycj\.local\bin\claude.exe"
$out  = ".\claude-dot-bun.bin"
$offs = 0x6DE1E00
$size = 0x6ED5E00
$fs   = [System.IO.File]::OpenRead($exe)
$fs.Seek($offs, [System.IO.SeekOrigin]::Begin) | Out-Null
$buf  = New-Object byte[] $size
$fs.Read($buf, 0, $size) | Out-Null
$fs.Close()
[System.IO.File]::WriteAllBytes($out, $buf)
Write-Host "Wrote $size bytes to $out"
```

其他版本或别的构建的 `claude.exe`，需先用 `dumpbin /HEADERS ...` 查看其 `.bun` 节的 `file pointer to raw data` 与 `size of raw data`，再替换 `$offs` 与 `$size`。

### 8.4 若要从 .bun 中解出「文件级」内容

需要理解 Bun 写进可执行文件的那段结构的格式，途径包括：

1. **逆向 Bun 源码**：在 [oven-sh/bun](https://github.com/oven-sh/bun) 中查找与 `--compile`、单文件可执行、嵌入资源、PE 节写入相关的实现（如 Windows 下的 linker/compiler 或 embed 相关代码），从中还原「.bun 节」的布局、索引与各 blob 的边界与含义。
2. **对 .bun 做盲解析**：在已有「.bun 节裸数据」的前提下，结合字符串、路径特征（如 `file:///src/...`、`$bunfs/`）、以及已知入口 `cli.js.jsc` 的位置，尝试推断出目录/索引结构；成功率依赖格式复杂度和是否压缩/加密等，一般需要配合 1 的结论。

目前未对 Bun 的该格式做进一步逆向，因此无法提供「从 .bun 解包出具体文件列表与内容」的现成工具或步骤；仅能完成到「按节切出 .bun 并落盘」这一步。
