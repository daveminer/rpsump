use linkify::{LinkFinder, LinkKind};
use reqwest::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockGuard, ResponseTemplate};

use crate::common::test_app::TestApp;

use serde_json::{Map, Value};

use self::auth::{NEW_EMAIL, TEST_EMAIL, TEST_PASSWORD};

pub mod auth;
pub mod heater;
pub mod info;
pub mod irrigation;
pub mod pool_pump;
pub mod sump_event;

pub fn link_from_email_text<'a>(text: &str) -> Vec<String> {
    let finder = LinkFinder::new();
    let links: Vec<_> = finder.links(text).collect();

    let mut found_links = vec![];
    for link in links {
        if link.kind() == &LinkKind::Url {
            found_links.push(link.as_str().to_string())
        }
    }

    return found_links;
}

pub fn param_from_email_text<'a>(text: &str, param: &str) -> Vec<String> {
    let finder = LinkFinder::new();
    let links: Vec<_> = finder.links(text).collect();

    let mut found_params = vec![];
    for link in links {
        if link.kind() == &LinkKind::Url {
            let url = Url::parse(link.as_str()).unwrap();
            let query_pairs = url.query_pairs();
            for pair in query_pairs {
                if pair.0 == param {
                    found_params.push(pair.1.into_owned());
                }
            }
        }
    }

    return found_params;
}

async fn email_link_from_mock_server(app: &TestApp) -> String {
    let verification_email = app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    let body = std::str::from_utf8(&verification_email.body).unwrap();

    let link = link_from_email_text(body);

    link[0].clone()
}

async fn mock_email_verification_send(app: &TestApp) -> MockGuard {
    Mock::given(path("/"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Email verification.")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await
}

pub fn new_user_params() -> Map<String, Value> {
    let mut map = serde_json::Map::new();
    map.insert("email".into(), NEW_EMAIL.into());
    map.insert("password".into(), TEST_PASSWORD.into());

    map
}

pub fn user_params() -> Map<String, Value> {
    let mut map = serde_json::Map::new();
    map.insert("email".into(), TEST_EMAIL.into());
    map.insert("password".into(), TEST_PASSWORD.into());

    map
}
