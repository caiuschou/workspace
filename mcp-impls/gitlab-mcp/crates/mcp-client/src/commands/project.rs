use crate::cli::ProjectCommands;
use crate::mcp_transport::McpServerClient;
use crate::output::OutputFormatter;
use crate::Result;

pub async fn execute_project(
    cmd: ProjectCommands,
    mut mcp_client: McpServerClient,
    formatter: OutputFormatter,
) -> Result<(McpServerClient, ())> {
    match cmd {
        ProjectCommands::List { search, per_page, page } => {
            let projects = mcp_client.list_projects(
                search,
                per_page as u64,
                page.map(|p| p as u64),
            )?;

            if formatter.is_table() {
                println!("\nProjects:");
                for p in &projects {
                    let desc = p.description.as_deref().unwrap_or("");
                    println!("{:<8} | {:<30} | {}", p.id, truncate(&p.name, 30), p.path_with_namespace);
                    if !desc.is_empty() {
                        println!("         └─ {}", truncate(desc, 70));
                    }
                    println!();
                }
            } else {
                for p in &projects {
                    println!("# {} ({})", p.path_with_namespace, p.id);
                    if let Some(d) = &p.description {
                        println!("  {}", d);
                    }
                    println!("  {}", p.web_url);
                    println!();
                }
            }

            formatter.success(&format!("Found {} project(s)", projects.len()));
        }

        ProjectCommands::Get { project_id } => {
            let project = mcp_client.get_project(&project_id)?;

            println!("\n{}", project.name);
            println!("{}", "=".repeat(project.name.len()));
            println!("ID:               {}", project.id);
            if let Some(iid) = project.iid {
                println!("IID:              {}", iid);
            }
            println!("Path:             {}", project.path_with_namespace);
            println!(
                "Visibility:       {}",
                project.visibility.as_deref().unwrap_or("Unknown")
            );
            println!(
                "Default Branch:   {}",
                project.default_branch.as_deref().unwrap_or("None")
            );
            println!(
                "Stars:            {}",
                project.star_count.unwrap_or(0)
            );
            println!(
                "Forks:            {}",
                project.forks_count.unwrap_or(0)
            );
            println!("Web URL:          {}", project.web_url);
            println!(
                "SSH URL:          {}",
                project.ssh_url_to_repo.as_deref().unwrap_or("None")
            );
            println!(
                "HTTP URL:         {}",
                project.http_url_to_repo.as_deref().unwrap_or("None")
            );

            if let Some(desc) = &project.description {
                println!("\nDescription:");
                println!("  {}", desc);
            }

            if let Some(topics) = &project.topics {
                if !topics.is_empty() {
                    println!("\nTopics:");
                    for topic in topics {
                        println!("  - {}", topic);
                    }
                }
            }

            if let Some(created) = &project.created_at {
                println!("\nCreated:         {}", created);
            }
            if let Some(activity) = &project.last_activity_at {
                println!("Last Activity:   {}", activity);
            }
        }

        ProjectCommands::Members { project_id } => {
            formatter.error(&format!(
                "Members command not implemented yet for project: {}",
                project_id
            ));
        }
    }

    Ok((mcp_client, ()))
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}
