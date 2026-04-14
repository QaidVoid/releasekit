use releasekit::Forge;
use releasekit::client::{HeaderMap, HttpClient, Response};
use releasekit::platform::Gitea;

#[derive(Clone)]
struct MockClient {
    response: String,
}

impl HttpClient for MockClient {
    fn get(&self, _url: &str, _headers: &HeaderMap) -> releasekit::error::Result<Response> {
        Ok(Response {
            status: 200,
            body: self.response.clone(),
        })
    }
}

fn sample_releases_json() -> &'static str {
    r#"[
        {
            "id": 1,
            "tag_name": "v1.0.0",
            "name": "Version 1.0",
            "prerelease": false,
            "published_at": "2025-01-15T10:30:00Z",
            "body": "First stable release",
            "assets": [
                {
                    "id": 1,
                    "name": "app-x86_64.AppImage",
                    "size": 5242880,
                    "browser_download_url": "https://codeberg.org/owner/repo/releases/download/v1.0.0/app-x86_64.AppImage"
                },
                {
                    "id": 2,
                    "name": "app-x86_64.tar.gz",
                    "size": 3145728,
                    "browser_download_url": "https://codeberg.org/owner/repo/releases/download/v1.0.0/app-x86_64.tar.gz"
                }
            ]
        },
        {
            "id": 2,
            "tag_name": "v0.9.0",
            "name": null,
            "prerelease": true,
            "published_at": "2024-12-01T08:00:00Z",
            "body": null,
            "assets": []
        }
    ]"#
}

fn sample_single_release_json() -> &'static str {
    r#"{
        "id": 3,
        "tag_name": "v2.0.0",
        "name": "Version 2.0",
        "prerelease": false,
        "published_at": "2025-06-01T12:00:00Z",
        "body": "Major update",
        "assets": [
            {
                "id": 3,
                "name": "app.tar.gz",
                "size": 1024,
                "browser_download_url": "https://codeberg.org/owner/repo/releases/download/v2.0.0/app.tar.gz"
            }
        ]
    }"#
}

#[test]
fn parses_multiple_releases() {
    let client = MockClient {
        response: sample_releases_json().to_string(),
    };
    let gt = Gitea::new(client, "https://codeberg.org");
    let releases = gt.fetch_releases("owner/repo", None).unwrap();

    assert_eq!(releases.len(), 2);

    let first = &releases[0];
    assert_eq!(first.tag(), "v1.0.0");
    assert_eq!(first.name(), Some("Version 1.0"));
    assert!(!first.is_prerelease());
    assert_eq!(first.published_at(), "2025-01-15T10:30:00Z");
    assert_eq!(first.body(), Some("First stable release"));
    assert_eq!(first.assets().len(), 2);

    let asset = &first.assets()[0];
    assert_eq!(asset.name(), "app-x86_64.AppImage");
    assert_eq!(asset.size(), Some(5_242_880));
    assert_eq!(
        asset.url(),
        "https://codeberg.org/owner/repo/releases/download/v1.0.0/app-x86_64.AppImage"
    );

    let second = &releases[1];
    assert_eq!(second.tag(), "v0.9.0");
    assert_eq!(second.name(), None);
    assert!(second.is_prerelease());
    assert_eq!(second.body(), None);
    assert!(second.assets().is_empty());
}

#[test]
fn parses_single_release() {
    let client = MockClient {
        response: sample_single_release_json().to_string(),
    };
    let gt = Gitea::new(client, "https://codeberg.org");
    let releases = gt.fetch_releases("owner/repo", Some("v2.0.0")).unwrap();

    assert_eq!(releases.len(), 1);
    assert_eq!(releases[0].tag(), "v2.0.0");
    assert_eq!(releases[0].assets().len(), 1);
}

#[test]
fn with_token_sends_auth_header() {
    use std::rc::Rc;

    #[derive(Clone)]
    struct InspectClient {
        response: String,
        got_auth: Rc<std::cell::RefCell<Option<String>>>,
    }

    impl HttpClient for InspectClient {
        fn get(&self, _url: &str, headers: &HeaderMap) -> releasekit::error::Result<Response> {
            for (k, v) in headers.iter() {
                if k == "Authorization" {
                    *self.got_auth.borrow_mut() = Some(v.to_string());
                }
            }
            Ok(Response {
                status: 200,
                body: self.response.clone(),
            })
        }
    }

    let got_auth = Rc::new(std::cell::RefCell::new(None));
    let client = InspectClient {
        response: sample_releases_json().to_string(),
        got_auth: got_auth.clone(),
    };
    let gt = Gitea::new(client, "https://codeberg.org").with_token("test-token-123");
    let _ = gt.fetch_releases("owner/repo", None).unwrap();

    assert_eq!(got_auth.borrow().as_deref(), Some("token test-token-123"));
}

#[test]
fn uses_correct_api_url() {
    use std::rc::Rc;

    #[derive(Clone)]
    struct InspectClient {
        response: String,
        got_url: Rc<std::cell::RefCell<Option<String>>>,
    }

    impl HttpClient for InspectClient {
        fn get(&self, url: &str, _headers: &HeaderMap) -> releasekit::error::Result<Response> {
            *self.got_url.borrow_mut() = Some(url.to_string());
            Ok(Response {
                status: 200,
                body: self.response.clone(),
            })
        }
    }

    let got_url = Rc::new(std::cell::RefCell::new(None));
    let client = InspectClient {
        response: sample_releases_json().to_string(),
        got_url: got_url.clone(),
    };
    let gt = Gitea::new(client, "https://codeberg.org");
    let _ = gt.fetch_releases("owner/repo", None).unwrap();

    assert_eq!(
        got_url.borrow().as_deref(),
        Some("https://codeberg.org/api/v1/repos/owner/repo/releases?limit=50")
    );
}

#[test]
fn custom_gitea_instance() {
    use std::rc::Rc;

    #[derive(Clone)]
    struct InspectClient {
        response: String,
        got_url: Rc<std::cell::RefCell<Option<String>>>,
    }

    impl HttpClient for InspectClient {
        fn get(&self, url: &str, _headers: &HeaderMap) -> releasekit::error::Result<Response> {
            *self.got_url.borrow_mut() = Some(url.to_string());
            Ok(Response {
                status: 200,
                body: self.response.clone(),
            })
        }
    }

    let got_url = Rc::new(std::cell::RefCell::new(None));
    let client = InspectClient {
        response: sample_single_release_json().to_string(),
        got_url: got_url.clone(),
    };
    let gt = Gitea::new(client, "https://gitea.example.com");
    let _ = gt.fetch_releases("owner/repo", Some("v2.0.0")).unwrap();

    assert_eq!(
        got_url.borrow().as_deref(),
        Some("https://gitea.example.com/api/v1/repos/owner/repo/releases/tags/v2.0.0")
    );
}
