# Cursor + Docker 容器开发

本仓库通过 **Dev Containers** 支持在 Docker 容器内用 Cursor 开发，环境包含 Rust、Node.js，与本机隔离、可复现。

## 前置条件

- [Docker Desktop](https://www.docker.com/products/docker-desktop/) 已安装并处于运行状态
- Cursor 已安装 **Dev Containers** 相关扩展，任选其一：
  - **推荐**：在 Cursor 扩展面板搜索 **「Dev Containers」** 或 **「Remote Containers」**，安装发行方为 **Anysphere** 的扩展（扩展 ID：`anysphere.remote-containers`）
  - **备选**：若在 Cursor 内搜不到，可到 [VS Code Marketplace - Dev Containers](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) 下载 `.vsix`，再拖入 Cursor 的扩展面板安装（参见 [Cursor 如何安装扩展](https://www.cursor.com/how-to-install-extension)）
  - **若仍无扩展**：可直接用 **devcontainer CLI** 在本地启动容器开发，见下文「仅用终端时」一节；或在 VS Code 中打开本仓库并 Reopen in Container，再在宿主机用 Cursor 编辑同一份代码。

## 使用步骤

### 1. 在容器中打开项目

1. 用 Cursor 打开本仓库根目录
2. 出现 **「Reopen in Container」** 时点击，或通过命令面板执行：`Dev Containers: Reopen in Container`
3. 等待镜像拉取与容器构建（首次可能数分钟）
4. 重新加载后，终端、扩展、文件访问均在**容器内**进行

### 2. 验证环境

在 Cursor 内置终端中执行：

```bash
pwd        # 应为 /workspace
rustc -V   # 应有 Rust 版本
node -V    # 应为 Node 20.x
cargo -V   # 应有 Cargo 版本
```

### 3. 日常开发

- **Rust**：在 `/workspace` 下执行 `cargo build`、`cargo test` 等，与本地一致
- **Node/Next.js**：在 `page/` 下执行 `npm install`、`npm run dev` 等
- **Git**：在容器内正常使用，提交、推送等与宿主机共享同一仓库

## 配置说明

配置位于 `.devcontainer/devcontainer.json`：

| 项 | 说明 |
|----|------|
| 基础镜像 | `mcr.microsoft.com/devcontainers/base:bookworm` |
| Rust | 通过 `ghcr.io/devcontainers/features/rust:1` 安装 |
| Node.js | 通过 `ghcr.io/devcontainers/features/node:1`，版本 20 |
| 工作目录 | 仓库挂载到容器内 `/workspace` |

可按需在该文件中增删 `features`、扩展或 `postCreateCommand`。

## 仅用终端时（不依赖 Cursor 界面）

若只需要同一套容器环境运行命令（CI、SSH 等），可用 devcontainer CLI：

```bash
# 安装 CLI（一次即可）
npm install -g @devcontainers/cli

# 在项目根目录启动容器
devcontainer up --workspace-folder .

# 在容器内执行命令
devcontainer exec --workspace-folder . cargo build
devcontainer exec --workspace-folder . bash -c "cd page && npm run build"
```

## 注意事项

- 容器内看到的代码即宿主机上的本仓库，修改会直接写回宿主机
- 需访问宿主机服务时，可用 `host.docker.internal`（Docker Desktop 常见用法）
- iOS/Android 模拟器等依赖本机图形或硬件的能力，在 Dev Container 内通常不可用，此类调试建议在宿主机进行
