# releasekit

Client-agnostic Rust library for fetching releases from git forges.

## Supported Platforms

| Platform | Status |
|----------|--------|
| GitHub   | Supported |
| GitLab   | Supported |
| Codeberg | Supported (via Gitea) |
| Gitea    | Supported |

## Quick Start

```rust
use releasekit::platform::GitHub;
use releasekit::client::UreqClient;
use releasekit::Forge;

let gh = GitHub::new(UreqClient)
    .with_token_from_env(&["GITHUB_TOKEN", "GH_TOKEN"]);

let releases = gh.fetch_releases("owner/repo", None).unwrap();
for r in &releases {
    println!("{} — {} assets", r.tag(), r.assets().len());
}
```

## Platforms

### GitHub

```rust
use releasekit::platform::GitHub;
use releasekit::client::UreqClient;
use releasekit::Forge;

let gh = GitHub::new(UreqClient)
    .with_token_from_env(&["GITHUB_TOKEN", "GH_TOKEN"]);

// GitHub Enterprise or API proxy
let gh = GitHub::new(UreqClient)
    .with_base_url("https://github.example.com/api/v3");
```

### GitLab

```rust
use releasekit::platform::GitLab;
use releasekit::client::UreqClient;
use releasekit::Forge;

let gl = GitLab::new(UreqClient)
    .with_token_from_env(&["GITLAB_TOKEN", "GL_TOKEN"]);

// Self-hosted GitLab
let gl = GitLab::new(UreqClient)
    .with_base_url("https://gitlab.example.com");

// Numeric project ID
let releases = gl.fetch_releases("12345", None).unwrap();
```

### Gitea / Codeberg

```rust
use releasekit::platform::Gitea;
use releasekit::client::UreqClient;
use releasekit::Forge;

// Codeberg
let cb = Gitea::new(UreqClient, "https://codeberg.org")
    .with_token_from_env(&["CODEBERG_TOKEN"]);

// Self-hosted Gitea
let gt = Gitea::new(UreqClient, "https://gitea.example.com")
    .with_token_from_env(&["GITEA_TOKEN"]);
```

## Asset Filtering

```rust
use releasekit::Filter;

let filter = Filter::new()
    .glob("*.AppImage")
    .include("x86_64")
    .exclude("debug");

for asset in release.assets() {
    if filter.matches(asset.name()) {
        println!("{}", asset.url());
    }
}
```

## Custom HTTP Backend

releasekit uses a trait-based HTTP client. Implement `HttpClient` to use your own backend:

```rust
use releasekit::client::{HttpClient, HeaderMap, Response};

#[derive(Clone)]
struct MyClient;

impl HttpClient for MyClient {
    fn get(&self, url: &str, headers: &HeaderMap) -> releasekit::error::Result<Response> {
        // your implementation
        todo!()
    }
}
```

A built-in `UreqClient` is available with the `ureq` feature (enabled by default).

## Examples

```bash
cargo run --example github -- owner/repo
cargo run --example gitlab -- owner/repo
cargo run --example gitea -- https://codeberg.org/owner/repo
```

## License

MIT
