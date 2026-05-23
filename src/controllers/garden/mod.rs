use actix_web::web::ServiceConfig;

pub mod event;
pub mod run;
pub mod schedule;
pub mod status;
pub mod stop;

pub fn garden_routes(cfg: &mut ServiceConfig) {
    cfg.service(schedule::list_schedules);
    cfg.service(schedule::create_schedule);
    cfg.service(schedule::get_schedule);
    cfg.service(schedule::update_schedule);
    cfg.service(schedule::delete_schedule);
    cfg.service(status::status);
    cfg.service(run::run_now);
    cfg.service(stop::stop);
    cfg.service(event::list_events);
}
