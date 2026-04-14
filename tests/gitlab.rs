use releasekit::Forge;
use releasekit::client::{HeaderMap, HttpClient, Response};
use releasekit::platform::GitLab;

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
            "name": "Version 1.0",
            "tag_name": "v1.0.0",
            "upcoming_release": false,
            "released_at": "2025-01-15T10:30:00Z",
            "description": "First stable release",
            "assets": {
                "count": 2,
                "sources": [],
                "links": [
                    {
                        "id": 1,
                        "name": "app-x86_64.AppImage",
                        "url": "https://gitlab.com/example/app/-/releases/v1.0.0/downloads/app-x86_64.AppImage",
                        "direct_asset_url": "https://gitlab.com/example/app/-/package_files/1/download",
                        "link_type": "other"
                    },
                    {
                        "id": 2,
                        "name": "app-x86_64.tar.gz",
                        "url": "https://gitlab.com/example/app/-/releases/v1.0.0/downloads/app-x86_64.tar.gz",
                        "direct_asset_url": "https://gitlab.com/example/app/-/package_files/2/download",
                        "link_type": "other"
                    }
                ]
            }
        },
        {
            "name": null,
            "tag_name": "v0.9.0",
            "upcoming_release": true,
            "released_at": "2024-12-01T08:00:00Z",
            "description": null,
            "assets": {
                "count": 0,
                "sources": [],
                "links": []
            }
        }
    ]"#
}

fn sample_single_release_json() -> &'static str {
    r#"{
        "name": "Version 2.0",
        "tag_name": "v2.0.0",
        "upcoming_release": false,
        "released_at": "2025-06-01T12:00:00Z",
        "description": "Major update",
        "assets": {
            "count": 1,
            "sources": [],
            "links": [
                {
                    "id": 3,
                    "name": "app.tar.gz",
                    "url": "https://gitlab.com/example/app/-/releases/v2.0.0/downloads/app.tar.gz",
                    "direct_asset_url": "https://gitlab.com/example/app/-/package_files/3/download",
                    "link_type": "other"
                }
            ]
        }
    }"#
}

#[test]
fn parses_multiple_releases() {
    let client = MockClient {
        response: sample_releases_json().to_string(),
    };
    let gl = GitLab::new(client);
    let releases = gl.fetch_releases("test/repo", None).unwrap();

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
    assert_eq!(asset.size(), None);
    assert_eq!(
        asset.url(),
        "https://gitlab.com/example/app/-/package_files/1/download"
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
    let gl = GitLab::new(client);
    let releases = gl.fetch_releases("test/repo", Some("v2.0.0")).unwrap();

    assert_eq!(releases.len(), 1);
    assert_eq!(releases[0].tag(), "v2.0.0");
    assert_eq!(releases[0].assets().len(), 1);
}

#[test]
fn with_token_sends_private_token_header() {
    use std::rc::Rc;

    #[derive(Clone)]
    struct InspectClient {
        response: String,
        got_token: Rc<std::cell::RefCell<Option<String>>>,
    }

    impl HttpClient for InspectClient {
        fn get(&self, _url: &str, headers: &HeaderMap) -> releasekit::error::Result<Response> {
            for (k, v) in headers.iter() {
                if k == "PRIVATE-TOKEN" {
                    *self.got_token.borrow_mut() = Some(v.to_string());
                }
            }
            Ok(Response {
                status: 200,
                body: self.response.clone(),
            })
        }
    }

    let got_token = Rc::new(std::cell::RefCell::new(None));
    let client = InspectClient {
        response: sample_releases_json().to_string(),
        got_token: got_token.clone(),
    };
    let gl = GitLab::new(client).with_token("glpat-test123");
    let _ = gl.fetch_releases("test/repo", None).unwrap();

    assert_eq!(got_token.borrow().as_deref(), Some("glpat-test123"));
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
    let gl = GitLab::new(client);
    let _ = gl.fetch_releases("owner/repo", None).unwrap();

    assert_eq!(
        got_url.borrow().as_deref(),
        Some("https://gitlab.com/api/v4/projects/owner%2Frepo/releases")
    );
}

#[test]
fn custom_base_url() {
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
    let gl = GitLab::new(client).with_base_url("https://gitlab.example.com");
    let _ = gl.fetch_releases("owner/repo", None).unwrap();

    assert_eq!(
        got_url.borrow().as_deref(),
        Some("https://gitlab.example.com/api/v4/projects/owner%2Frepo/releases")
    );
}

#[test]
fn numeric_project_id() {
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
    let gl = GitLab::new(client);
    let _ = gl.fetch_releases("12345", None).unwrap();

    assert_eq!(
        got_url.borrow().as_deref(),
        Some("https://gitlab.com/api/v4/projects/12345/releases")
    );
}
