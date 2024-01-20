use anyhow::{anyhow, Error};
use bcrypt::{hash, DEFAULT_COST};
use secrecy::{ExposeSecret, Secret, SecretString};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Deserialize)]
pub struct Password(SecretString);

#[derive(Debug, serde::Deserialize)]
pub struct AuthParams {
    pub email: String,
    pub password: Secret<String>,
}

impl Password {
    pub fn hash(&self) -> Result<String, Error> {
        hash(self.expose_secret().clone(), DEFAULT_COST)
            .map_err(|_| anyhow!("Could not hash password."))
    }

    pub fn new(secret: String) -> Self {
        Self(SecretString::new(secret))
    }
}

impl Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl Eq for Password {}

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl ExposeSecret<String> for Password {
    fn expose_secret(&self) -> &String {
        &self.0.expose_secret()
    }
}

impl Serialize for Password {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.expose_secret().serialize(serializer)
    }
}
