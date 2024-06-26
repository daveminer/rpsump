use chrono::Utc;
use jsonwebtoken::Algorithm;
use reqwest::header::{HeaderName, HeaderValue};
use std::{env, str::FromStr};

use rpsump::auth::claim::Claim;
use rpsump::repository::models::user::User;

pub fn create_auth_header(token: &str) -> (HeaderName, HeaderValue) {
    (
        HeaderName::from_str("Authorization").unwrap(),
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    )
}

fn create_expired_token(user: User) -> String {
    let now = Utc::now().timestamp();
    let expiration_time = now - 3600 * 24 * 8; // Set the expiration time to 8 days ago
    let claim = Claim {
        sub: user.id.to_string(),
        iat: now as u64,
        exp: expiration_time as u64,
    };

    let jwt_secret =
        env::var("JWT_SECRET").expect(&format!("{} environment variable not found.", "JWT_SECRET"));

    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(Algorithm::HS256),
        &claim,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap()
}

fn create_valid_token(user: User) -> String {
    let now = Utc::now().timestamp();
    let expiration_time = now + 3600; // Set the expiration time to 1 hour from now
    let claim = Claim {
        sub: user.id.to_string(),
        iat: now as u64,
        exp: expiration_time as u64,
    };

    let jwt_secret =
        env::var("JWT_SECRET").expect(&format!("{} environment variable not found.", "JWT_SECRET"));

    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(Algorithm::HS256),
        &claim,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .unwrap()
}

mod tests {
    use reqwest::StatusCode;
    use rpsump::{repository::models::user::UserFilter, test_fixtures::gpio::build_mock_gpio};

    use crate::{
        auth::authenticated_user::{create_auth_header, create_expired_token, create_valid_token},
        common::test_app::spawn_app,
        controllers::auth::TEST_EMAIL,
    };

    #[tokio::test]
    async fn protected_request_valid_token() {
        let app = spawn_app(&build_mock_gpio()).await;

        let user_filter = UserFilter {
            email: Some(TEST_EMAIL.into()),
            ..Default::default()
        };
        let users = app.repo.users(user_filter).await.unwrap();
        let token = create_valid_token(users[0].clone());
        let (header_name, header_value) = create_auth_header(&token);

        let result = app
            .api_client
            .get(&format!("{}/info", &app.address))
            .header(header_name, header_value)
            .send()
            .await
            .expect("Failed to execute request.");

        assert!(result.status() == StatusCode::OK);
    }

    #[tokio::test]
    async fn protected_request_failed_no_token() {
        let app = spawn_app(&build_mock_gpio()).await;

        let result = app
            .api_client
            .get(&format!("{}/info", &app.address))
            .send()
            .await
            .expect("Failed to execute request.");

        assert!(result.status() == StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn protected_request_failed_expired_token() {
        let app = spawn_app(&build_mock_gpio()).await;

        let user_filter = UserFilter {
            email: Some(TEST_EMAIL.into()),
            ..Default::default()
        };

        let user = &app.repo.users(user_filter).await.unwrap()[0];

        let token = create_expired_token(user.clone());
        let (header_name, header_value) = create_auth_header(&token);

        let result = app
            .api_client
            .get(&format!("{}/info", &app.address))
            .header(header_name, header_value)
            .send()
            .await
            .expect("Failed to execute request.");

        assert!(result.status() == StatusCode::UNAUTHORIZED);
    }
}
