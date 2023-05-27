pub mod rfc3339;
pub mod sump_event;
pub mod user;
pub mod user_event;

/// Convenience functions for serializing and deserializing times in RFC 3339 format.
/// Used for returning time values in JSON API responses.
/// Example: `2012-02-22T14:53:18+00:00`.
use chrono::{DateTime, Utc};
use serde::{self, Deserialize, Deserializer, Serializer};

pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    //let s = dt.to_rfc3339(); //DateTime::<Utc>::from_utc(*dt, Utc).to_rfc3339();
    serializer.serialize_str(&dt.to_rfc3339())
}
pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let dt = DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)?;
    Ok(dt.into())
}

/// Wrapper for dealing with Option<DateTime<Utc>>
pub mod option {
    use chrono::{DateTime, Utc};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(dt: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *dt {
            Some(dt) => super::serialize(&dt, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match super::deserialize(deserializer) {
            Ok(dt) => Ok(Some(dt)),
            Err(_) => Ok(None),
        }
    }
}
