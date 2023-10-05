use actix_web::web::ServiceConfig;

pub mod event;
pub mod schedule;

pub fn irrigation_routes(cfg: &mut ServiceConfig) {
    //cfg.service(event::verify_email);
    cfg.service(schedule::delete_irrigation_schedule);
    cfg.service(schedule::new_irrigation_schedule);
}
