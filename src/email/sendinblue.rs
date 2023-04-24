use anyhow::Error;
use reqwest;
use serde_json::{json, Map, Value};

use crate::auth::reset_token;
use crate::database::DbPool;
use crate::email::ResetEmail;
use crate::models::user::User;

impl ResetEmail {
    fn new(user: User, link: String) -> Self {
        ResetEmail {
            to: vec![user.email],
            sender: sender(),
            subject: "RPSump Password Reset".to_string(),
            html_content: format!("<!DOCTYPE html> <html> <body> <h1>Reset your password</h1><p>Follow this link to reset your password:\n\n{}\n\n</p>", link),
        }
    }
}

pub async fn send_password_reset_email(
    user: User,
    db: DbPool,
    auth_token: String,
) -> Result<(), Error> {
    let request_url = "https://api.sendinblue.com/v3/smtp/email";

    let (reset_token, reset_token_hash) = reset_token::generate()?;

    User::save_reset_token(user.clone(), reset_token_hash, db);

    let mut email = Map::new();

    email.insert("to".to_string(), json![user.email]);
    email.insert("sender".to_string(), json!("wtrispsn@gmail.com"));
    email.insert("subject".to_string(), json!("RPSump Password Reset"));
    email.insert(
        "htmlContent".to_string(),
        json!(format!(
            "Follow this link to reset your password:\n\n{}\n\n",
            reset_token
        )),
    );

    let client = reqwest::Client::new();

    let response = client
        .post(request_url)
        .header("accept", "application/json")
        .header("api-key", auth_token)
        .header("content-type", "application/json")
        .body("");

    println!("RESP {:?}", response);

    Ok(())
}

fn sender() -> Value {
    // TODO: parameterize this
    json!({
        "name": "RPSump",
        "email": "wtrispsn@gmail.com"
    })
}
