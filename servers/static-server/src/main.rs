use std::{env::args, net::SocketAddr};

use axum::{Router, routing::get};
use shared_config::Cfg;

mod landing;
mod resume;
mod blog;
mod favicon;
mod static_file;

mod not_found;
mod internal_error;

#[tokio::main]
async fn main() {
    let root_config = {
        let mut args = args();
        let _ = args.next().expect("first value to be present");
        args.next().expect("config path to be present as first argument")
    };
    let cfg = {
        let temp: Cfg = config::Config::builder()
            .add_source(config::File::with_name(root_config.as_str()))
            .add_source(config::Environment::with_prefix("BENXU_DEV"))
            .build().expect("Configuration parses correctly")
            .try_deserialize().expect("Configuration parses correctly");
        Box::leak(Box::new(temp))
    };

    // Set up logging.
    tracing_subscriber::fmt().init();
    trc::info!("Spinning up! (pwd: {:?})", std::env::current_dir().expect("existing directory").as_os_str());

    let app = Router::new()
        // Starting points
        .route("/", get(landing::page))
        .route("/blog/*path", get(blog::page))
        // Static files
        .route("/favicon.svg", get(favicon::svg))
        .route("/favicon.ico", get(favicon::ico))
        .route("/resume", get(resume::file))
        .route("/public/css/:file", get(static_file::css))
        .route("/public/js/:file", get(static_file::js))
        .route("/public/wasm/:file", get(static_file::wasm))
        .route("/public/png/:file", get(static_file::png))
        .route("/public/jpg/:file", get(static_file::jpg))
        .route("/public/svg/:file", get(static_file::svg))
        // Error paths
        .fallback(not_found::page);


    let server = axum::Server::bind(&SocketAddr::new(cfg.servers.primary.address.ip, cfg.servers.primary.address.port))
        .serve(app.into_make_service());

    let handle = tokio::spawn(server);
    trc::info!("Spun up!");
    handle.await.expect("no issues joining").expect("no issues from server");
}
