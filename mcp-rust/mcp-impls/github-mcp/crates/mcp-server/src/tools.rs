pub mod repositories;
pub mod issues;
pub mod pulls;
pub mod files;
pub mod branches;
pub mod commits;
pub mod tokens;

use mcp_server::{McpServer, ServerError};
use std::sync::Arc;
use crate::client::GithubState;

/// 注册所有 GitHub 工具
pub fn register_all_tools(
    server: &mut McpServer,
    state: Arc<GithubState>,
) -> Result<(), ServerError> {
    // 仓库相关工具
    repositories::register_tools(server, state.clone())?;

    // Issue 相关工具
    issues::register_tools(server, state.clone())?;

    // Pull Request 相关工具
    pulls::register_tools(server, state.clone())?;

    // 文件操作工具
    files::register_tools(server, state.clone())?;

    // 分支操作工具
    branches::register_tools(server, state.clone())?;

    // 提交操作工具
    commits::register_tools(server, state.clone())?;

    // Token 管理工具
    tokens::register_tools(server, state.clone())?;

    Ok(())
}
