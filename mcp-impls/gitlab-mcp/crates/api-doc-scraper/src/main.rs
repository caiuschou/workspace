use std::path::PathBuf;
use std::time::Instant;

use api_doc_scraper::{DocScraperClient, HtmlParser, ResourceCategory, get_all_resources, get_resources_by_category};
use clap::Parser;
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

/// GitLab API Documentation Scraper
///
/// Scrape GitLab REST API documentation from docs.gitlab.com
/// and save as Markdown files.
#[derive(Parser, Debug)]
#[command(name = "api-doc-scraper")]
#[command(author = "GitLab MCP Contributors")]
#[command(version = "0.1.0")]
struct Cli {
    /// Output directory for scraped documentation
    #[arg(short, long, default_value = "docs/gitlab-api")]
    output_dir: PathBuf,

    /// Scrape only a specific resource (by name, e.g., "issues")
    #[arg(short, long)]
    resource: Option<String>,

    /// Scrape only resources from a specific category
    #[arg(long)]
    category: Option<String>,

    /// Dry run - show what would be scraped without actually scraping
    #[arg(long)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug)]
struct ScrapeSummary {
    total: usize,
    successful: usize,
    failed: usize,
    skipped: usize,
    duration: std::time::Duration,
    failed_resources: Vec<(String, String)>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_env_filter(EnvFilter::from_default_env().add_directive(log_level.into()))
        .init();

    info!("GitLab API Documentation Scraper starting...");

    // Get resources to scrape
    let resources = filter_resources(cli.resource, cli.category)?;

    if resources.is_empty() {
        warn!("No resources found matching the criteria");
        return Ok(());
    }

    info!("Found {} resources to scrape", resources.len());

    if cli.dry_run {
        println!("\nDry run - would scrape the following resources:\n");
        for category in &[ResourceCategory::Project, ResourceCategory::Group, ResourceCategory::Standalone, ResourceCategory::Templates] {
            let category_resources: Vec<_> = resources.iter()
                .filter(|r| r.category == *category)
                .collect();

            if !category_resources.is_empty() {
                println!("\n{}", category.display_name());
                println!("{}", "=".repeat(category.display_name().len()));
                for r in category_resources {
                    println!("  - {} -> {}", r.name, r.output_path.display());
                }
            }
        }
        println!("\nTotal: {} resources", resources.len());
        return Ok(());
    }

    // Create output directories
    create_output_dirs(&cli.output_dir, &resources)?;

    // Initialize scraper
    let client = DocScraperClient::new()?;

    // Setup progress bar
    let pb = ProgressBar::new(resources.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("##>-"),
    );

    let start = Instant::now();
    let mut summary = ScrapeSummary {
        total: resources.len(),
        successful: 0,
        failed: 0,
        skipped: 0,
        duration: std::time::Duration::ZERO,
        failed_resources: Vec::new(),
    };

    // Scrape each resource
    for resource in &resources {
        pb.set_message(format!("Scraping {}...", resource.name));

        match scrape_resource(&client, &cli.output_dir, resource).await {
            Ok(_) => {
                summary.successful += 1;
            }
            Err(e) => {
                summary.failed += 1;
                summary.failed_resources.push((resource.name.clone(), e.to_string()));
                error!("Failed to scrape {}: {}", resource.name, e);
            }
        }

        pb.inc(1);
    }

    summary.duration = start.elapsed();
    pb.finish_with_message(format!("Completed in {}", HumanDuration(summary.duration)));

    // Print summary
    print_summary(&summary);

    // Generate index
    generate_index(&cli.output_dir, &resources)?;

    Ok(())
}

fn filter_resources(
    resource_filter: Option<String>,
    category_filter: Option<String>,
) -> anyhow::Result<Vec<api_doc_scraper::ApiResource>> {
    let all_resources = get_all_resources();

    let resources = if let Some(ref name) = resource_filter {
        all_resources
            .into_iter()
            .filter(|r| {
                r.name.to_lowercase().contains(&name.to_lowercase())
                    || r.url_slug.to_lowercase().contains(&name.to_lowercase())
            })
            .collect()
    } else if let Some(ref category) = category_filter {
        let cat = match category.to_lowercase().as_str() {
            "project" => ResourceCategory::Project,
            "group" => ResourceCategory::Group,
            "standalone" => ResourceCategory::Standalone,
            "templates" | "template" => ResourceCategory::Templates,
            _ => return Err(anyhow::anyhow!("Unknown category: {}", category)),
        };
        all_resources
            .into_iter()
            .filter(|r| r.category == cat)
            .collect()
    } else {
        all_resources
    };

    Ok(resources)
}

fn create_output_dirs(
    output_dir: &PathBuf,
    resources: &[api_doc_scraper::ApiResource],
) -> anyhow::Result<()> {
    // Create category directories
    std::fs::create_dir_all(output_dir.join("project"))?;
    std::fs::create_dir_all(output_dir.join("group"))?;
    std::fs::create_dir_all(output_dir.join("standalone"))?;
    std::fs::create_dir_all(output_dir.join("templates"))?;

    Ok(())
}

async fn scrape_resource(
    client: &DocScraperClient,
    output_dir: &PathBuf,
    resource: &api_doc_scraper::ApiResource,
) -> anyhow::Result<()> {
    // Fetch the page
    let html = client.fetch_page(&resource.url()).await?;

    // Parse and convert to markdown
    let (title, markdown) = HtmlParser::process_page(&html)?;

    // Write to file
    let output_path = output_dir.join(&resource.output_path);
    std::fs::create_dir_all(output_path.parent().unwrap())?;

    let content = format!("# {}\n\n> Source: [GitLab Documentation](https://docs.gitlab.com/{}.html)\n\n{}\n",
        title, resource.url(), markdown);

    std::fs::write(&output_path, content)?;

    tracing::debug!("Wrote {}", output_path.display());

    Ok(())
}

fn print_summary(summary: &ScrapeSummary) {
    println!("\n{}", "=".repeat(60));
    println!("Scrape Summary");
    println!("{}", "=".repeat(60));
    println!("Total resources:  {}", summary.total);
    println!("Successful:      {}", summary.successful);
    println!("Failed:          {}", summary.failed);
    println!("Duration:        {}", HumanDuration(summary.duration));

    if !summary.failed_resources.is_empty() {
        println!("\nFailed resources:");
        for (name, error) in &summary.failed_resources {
            println!("  - {}: {}", name, error);
        }
    }
    println!("{}", "=".repeat(60));
}

fn generate_index(
    output_dir: &PathBuf,
    resources: &[api_doc_scraper::ApiResource],
) -> anyhow::Result<()> {
    let index_path = output_dir.join("README.md");

    let mut content = String::from("# GitLab REST API Documentation\n\n");
    content.push_str(&format!("> Scraped from [GitLab API Documentation](https://docs.gitlab.com/ee/api/)\n\n"));
    content.push_str(&format!("Total resources: {}\n\n", resources.len()));

    // Group by category
    for category in &[ResourceCategory::Project, ResourceCategory::Group, ResourceCategory::Standalone, ResourceCategory::Templates] {
        let category_resources: Vec<_> = resources.iter()
            .filter(|r| r.category == *category)
            .collect();

        if !category_resources.is_empty() {
            content.push_str(&format!("## {}\n\n", category.display_name()));

            for r in category_resources {
                let rel_path = r.output_path.display().to_string().replace('\\', "/");
                content.push_str(&format!("- [{}]({})\n", r.name, rel_path));
            }

            content.push_str("\n");
        }
    }

    std::fs::write(&index_path, content)?;

    info!("Generated index at {}", index_path.display());

    Ok(())
}
