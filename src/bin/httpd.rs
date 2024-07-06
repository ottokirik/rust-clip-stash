use std::path::PathBuf;

use clipstash::{
    data::AppDatabase,
    domain::maintenance::Maintenance,
    rocket,
    web::{hit_counter::HitCounter, renderer::Renderer},
    RocketConfig,
};
use dotenv::dotenv;
use structopt::StructOpt;
use tokio::runtime::Runtime;

fn main() {
    dotenv().ok();

    let opt = Opt::from_args();
    let rt = Runtime::new().expect("failed to spawn runtime");
    let handle = rt.handle().clone();
    let renderer = Renderer::new(opt.template_dir.clone());
    let database = rt.block_on(async move { AppDatabase::new(&opt.connection_string).await });
    let hit_counter = HitCounter::new(database.get_pool().clone(), handle.clone());
    let maintenance = Maintenance::spawn(database.get_pool().clone(), handle.clone());
    let config = RocketConfig {
        renderer,
        database,
        hit_counter,
        maintenance,
    };

    rt.block_on(async move {
        rocket(config)
            .launch()
            .await
            .expect("failed to launch server");
    });
}

#[derive(StructOpt, Debug)]
#[structopt(name = "httpd")]
struct Opt {
    #[structopt(default_value = "sqlite:data.db")]
    connection_string: String,
    #[structopt(short, long, parse(from_os_str), default_value = "templates/")]
    template_dir: PathBuf,
}
