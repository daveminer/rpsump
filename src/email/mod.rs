use serde::{Deserialize, Serialize};

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
    pub async fn send_email_verification(
        self,
        db: actix_web::web::Data<DbPool>,
        server_url: String,
        auth_token: String,
    ) -> Result<(), anyhow::Error> {
        sendinblue::send_email_verification(self, db, server_url, auth_token).await
    }

    pub async fn send_password_reset(
        self,
        db: actix_web::web::Data<DbPool>,
        server_url: String,
        auth_token: String,
    ) -> Result<(), anyhow::Error> {
        sendinblue::send_password_reset(self, db, server_url, auth_token).await
    }
}
