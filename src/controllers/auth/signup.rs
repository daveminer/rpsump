use actix_web::{post, web, web::Data, HttpResponse, Responder, Result};
use bcrypt::{hash, DEFAULT_COST};
use diesel::RunQueryDsl;

use crate::controllers::auth::AuthParams;
use crate::database::DbPool;
use crate::models::user::NewUser;
use crate::schema::user;

#[post("/signup")]
pub async fn signup(user_data: web::Json<AuthParams>, db: Data<DbPool>) -> Result<impl Responder> {
    let new_user = user_data.into_inner();
    let password_hash = hash(&new_user.password, DEFAULT_COST).expect("Could not hash password.");

    let mut conn = db.get().expect("Could not get db connection.");

    let _user = diesel::insert_into(user::table)
        .values(&NewUser {
            email: new_user.email,
            password_hash,
        })
        .execute(&mut conn)
        .expect("Could not insert new user");

    Ok(HttpResponse::Ok().finish())
}
