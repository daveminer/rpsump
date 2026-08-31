use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::invite;

#[derive(Clone, Debug, Identifiable, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = invite)]
pub struct Invite {
    pub id: i32,
    pub email: String,
    pub token: String,
    pub invited_by_user_id: i32,
    pub expires_at: NaiveDateTime,
    pub accepted_at: Option<NaiveDateTime>,
    pub accepted_by_user_id: Option<i32>,
    pub created_at: NaiveDateTime,
}

impl Invite {
    /// An invite is usable only while it is unexpired and unaccepted. Callers
    /// must also confirm the signup address matches `email`; a token alone is
    /// not sufficient, so that a forwarded link cannot be redeemed by someone
    /// other than the intended recipient.
    pub fn is_redeemable(&self, now: NaiveDateTime) -> bool {
        self.accepted_at.is_none() && self.expires_at > now
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, NaiveDate};

    fn at(offset_secs: i64) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 8, 31)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            + Duration::seconds(offset_secs)
    }

    fn invite(expires_at: NaiveDateTime, accepted_at: Option<NaiveDateTime>) -> Invite {
        Invite {
            id: 1,
            email: "invitee@test.local".into(),
            token: "token".into(),
            invited_by_user_id: 1,
            expires_at,
            accepted_at,
            accepted_by_user_id: accepted_at.map(|_| 2),
            created_at: at(0),
        }
    }

    #[test]
    fn unexpired_and_unaccepted_is_redeemable() {
        assert!(invite(at(60), None).is_redeemable(at(0)));
    }

    #[test]
    fn an_expired_invite_is_not_redeemable() {
        assert!(!invite(at(-1), None).is_redeemable(at(0)));
    }

    #[test]
    fn expiry_is_exclusive_at_the_boundary() {
        assert!(!invite(at(0), None).is_redeemable(at(0)));
    }

    #[test]
    fn an_accepted_invite_is_not_redeemable_even_if_unexpired() {
        assert!(!invite(at(60), Some(at(1))).is_redeemable(at(0)));
    }
}
