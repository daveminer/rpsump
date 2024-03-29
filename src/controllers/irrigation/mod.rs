use actix_web::web::ServiceConfig;

pub mod event;
pub mod schedule;

pub fn irrigation_routes(cfg: &mut ServiceConfig) {
    cfg.service(event::irrigation_event);
    cfg.service(schedule::delete_irrigation_schedule);
    cfg.service(schedule::edit_irrigation_schedule);
    cfg.service(schedule::irrigation_schedule);
    cfg.service(schedule::irrigation_schedules);
    cfg.service(schedule::new_irrigation_schedule);
}
