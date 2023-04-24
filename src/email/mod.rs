use anyhow::Error;
use serde_json::Value;

use crate::database::DbPool;
use crate::models::user::User;

pub mod sendinblue;

struct ResetEmail {
    to: Vec<String>,
    sender: Value,
    subject: String,
    html_content: String,
}

struct Contact {
    email: String,
    name: String,
}

// pub fn send_email_verification_email(user: User) -> Result<(), Error> {
//     let mut mailer =
//         SmtpClient::new_simple("smtp.gmail.com")?.credentials(Credentials::new("").unwrap());

//     Ok(())
// }

pub async fn send_password_reset_email(
    user: User,
    db: DbPool,
    auth_token: String,
) -> Result<(), Error> {
    let result = sendinblue::send_password_reset_email(user, db, auth_token).await?;

    println!("RESULT: {:?}", result);

    Ok(())
}

// fn send(user: User) {
//     let link = format!("https://domain.tld/reset/{}", user.reset_token);

//     let email = Message::builder()
//         .from("RPSump <system@domain.tld>".parse().unwrap())
//         .to(user.email)
//         .subject("RPSump Password Reset")
//         .header(ContentType::TEXT_PLAIN)
//         .body(format!(
//             "Follow this link to reset your password:\n\n{}\n\n",
//             "link"
//         ))
//         .unwrap();
// }
