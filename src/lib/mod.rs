pub mod data;
pub mod domain;
pub mod service;
pub mod web;

pub use domain::clip::field::ShortCode;
pub use domain::clip::Clip;
pub use domain::clip::ClipError;
pub use domain::time::Time;

pub use data::DataError;

use data::AppDatabase;
use rocket::fs::FileServer;
use rocket::{Build, Rocket};
use web::hit_counter::HitCounter;
use web::http::catcher::catchers;
use web::http::routes;
use web::renderer::Renderer;

pub fn rocket(config: RocketConfig) -> Rocket<Build> {
    rocket::build()
        .manage::<AppDatabase>(config.database)
        .manage::<Renderer>(config.renderer)
        .manage::<HitCounter>(config.hit_counter)
        .mount("/", routes())
        .mount("/static", FileServer::from("static"))
        .register("/", catchers())
}

pub struct RocketConfig {
    pub renderer: Renderer<'static>,
    pub database: AppDatabase,
    pub hit_counter: HitCounter,
}
