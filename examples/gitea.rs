//! Example: fetch and list releases from a Gitea or Codeberg repository.
//!
//! Usage:
//!   cargo run --example gitea -- https://codeberg.org/owner/repo
//!   cargo run --example gitea -- https://codeberg.org/owner/repo v1.0.0
//!   cargo run --example gitea -- https://gitea.example.com/owner/repo

use releasekit::Filter;
use releasekit::Forge;
use releasekit::client::UreqClient;
use releasekit::platform::Gitea;
use releasekit::url::PlatformUrl;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: gitea <url> [tag]");
        eprintln!();
        eprintln!("examples:");
        eprintln!("  gitea https://codeberg.org/owner/repo");
        eprintln!("  gitea https://gitea.example.com/owner/repo v1.0");
        std::process::exit(1);
    }

    let input = &args[1];
    let (base_url, project, tag) = parse_input(input, args.get(2).map(|s| s.as_str()));

    let token_vars: &[&str] = if base_url.contains("codeberg.org") {
        &["CODEBERG_TOKEN"]
    } else {
        &["GITEA_TOKEN"]
    };

    let gt = Gitea::new(UreqClient, &base_url).with_token_from_env(token_vars);

    let releases = match gt.fetch_releases(&project, tag.as_deref()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    if releases.is_empty() {
        println!("No releases found.");
        return;
    }

    let filter = Filter::new()
        .glob("*.tar.gz")
        .glob("*.AppImage")
        .exclude("debug");

    for release in &releases {
        println!(
            "{} {} ({} assets)",
            release.tag(),
            release.name().unwrap_or(""),
            release.assets().len()
        );

        for asset in release.assets() {
            let marker = if filter.matches(asset.name()) {
                "✓"
            } else {
                " "
            };
            let size = asset.size().map(|s| format!(" ({s})")).unwrap_or_default();
            println!("  {marker} {}{}", asset.name(), size);
        }

        println!();
    }
}

fn parse_input(input: &str, tag_arg: Option<&str>) -> (String, String, Option<String>) {
    if let Some(url) = PlatformUrl::parse(input) {
        match url {
            PlatformUrl::Codeberg { project, tag } => (
                "https://codeberg.org".to_string(),
                project,
                tag.or(tag_arg.map(String::from)),
            ),
            PlatformUrl::Gitea {
                instance,
                project,
                tag,
            } => (
                format!("https://{instance}"),
                project,
                tag.or(tag_arg.map(String::from)),
            ),
            _ => {
                eprintln!("error: only Gitea/Codeberg URLs are supported in this example");
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("error: please provide a full URL (e.g. https://codeberg.org/owner/repo)");
        std::process::exit(1);
    }
}
