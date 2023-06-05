use actix_web::web::Data;
use linkify::{LinkFinder, LinkKind};
use reqwest::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockGuard, ResponseTemplate};

use crate::common::test_app::TestApp;

use rpsump::auth::password::Password;
use rpsump::database::DbPool;
use rpsump::models::user::User;
use serde_json::{Map, Value};

pub mod auth;
pub mod info;
pub mod sump_event;

const TEST_EMAIL: &str = "test_acct@test.local";
const TEST_PASSWORD: &str = "testing87_*Password";

pub async fn create_test_user(db_pool: Data<DbPool>) -> User {
    User::create(
        TEST_EMAIL.into(),
        Password::new(TEST_PASSWORD.into()).hash().unwrap(),
        "127.0.0.1".into(),
        db_pool,
    )
    .await
    .unwrap()
}

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

pub fn user_params() -> Map<String, Value> {
    let mut map = serde_json::Map::new();
    map.insert("email".into(), TEST_EMAIL.into());
    map.insert("password".into(), TEST_PASSWORD.into());

    map
}
