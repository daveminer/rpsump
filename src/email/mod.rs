use actix_web::web::Data;
use serde::{Deserialize, Serialize};

use crate::config::MailerConfig;
use crate::database::DbPool;
use crate::models::user::User;

pub mod sendinblue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    to: Vec<Contact>,
    sender: Contact,
    subject: String,
    #[serde(rename = "htmlContent")]
    html_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Contact {
    email: String,
    name: Option<String>,
}

impl User {
    #[tracing::instrument(name = "Send email verification", skip(self, db, mailer, server_url), fields(user_id = self.id))]
    pub async fn send_email_verification(
        self,
        db: Data<DbPool>,
        mailer: MailerConfig,
        server_url: &str,
    ) -> Result<(), anyhow::Error> {
        sendinblue::send_email_verification(self, db, mailer, server_url).await
    }

    #[tracing::instrument(name = "Send password reset", skip(self, db, server_url, auth_token), fields(user_id = self.id))]
    pub async fn send_password_reset(
        self,
        db: Data<DbPool>,
        mailer_url: &str,
        server_url: &str,
        auth_token: &str,
    ) -> Result<(), anyhow::Error> {
        sendinblue::send_password_reset(self, db, mailer_url, server_url, auth_token).await
    }
}

#[tracing::instrument(name = "Send error notification", skip(mailer))]
pub async fn send_error_notification(
    mailer: &MailerConfig,
    error_msg: &str,
) -> Result<(), anyhow::Error> {
    sendinblue::send_error_email(mailer, error_msg).await
}
