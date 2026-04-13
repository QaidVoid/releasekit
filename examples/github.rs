//! Example: fetch and list releases from a GitHub repository.
//!
//! Usage:
//!   cargo run --example github -- owner/repo
//!   cargo run --example github -- owner/repo v1.0.0
//!   cargo run --example github -- https://github.com/owner/repo

use releasekit::Filter;
use releasekit::Forge;
use releasekit::client::UreqClient;
use releasekit::platform::GitHub;
use releasekit::url::PlatformUrl;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: github <owner/repo | url> [tag]");
        std::process::exit(1);
    }

    let input = &args[1];
    let (project, tag) = parse_input(input, args.get(2).map(|s| s.as_str()));

    let gh = GitHub::new(UreqClient).with_token_from_env(&["GITHUB_TOKEN"]);

    let releases = match gh.fetch_releases(&project, tag.as_deref()) {
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

fn parse_input(input: &str, tag_arg: Option<&str>) -> (String, Option<String>) {
    if let Some(url) = PlatformUrl::parse(input) {
        match url {
            PlatformUrl::GitHub { project, tag } => (project, tag.or(tag_arg.map(String::from))),
            _ => {
                eprintln!("error: only GitHub URLs are supported in this example");
                std::process::exit(1);
            }
        }
    } else {
        (input.to_string(), tag_arg.map(String::from))
    }
}
