use std::path::PathBuf;

use clipstash::{data::AppDatabase, rocket, web::renderer::Renderer, RocketConfig};
use dotenv::dotenv;
use structopt::StructOpt;
use tokio::runtime::Runtime;

fn main() {
    dotenv().ok();

    let opt = Opt::from_args();
    let rt = Runtime::new().expect("failed to spawn runtime");
    let handle = rt.handle().clone();

    rt.block_on(async move {
        let renderer = Renderer::new(opt.template_dir);
        let database = AppDatabase::new(&opt.connection_string).await;

        let config = RocketConfig { renderer, database };

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
