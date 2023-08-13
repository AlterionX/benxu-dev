use std::net::SocketAddr;

use axum::{Router, routing::get};
use config::Config;
use shared_config::{Cfg, ServersCfg, ServerCfg};

mod landing;
mod resume;
mod blog;
mod favicon;
mod static_file;

mod not_found;
mod internal_error;

#[tokio::main]
async fn main() {
    let raw_cfg = Config::builder()
        .add_source(config::File::with_name("config/active"))
        .add_source(config::Environment::with_prefix("BENXU_DEV"))
        .build().expect("builder works");
    let cfg = Cfg {
        servers: ServersCfg {
            primary: ServerCfg {
                ip: raw_cfg.get_string("addresses.primary.ip").expect("ok").parse().expect("valid ip"),
                port: raw_cfg.get_int("addresses.primary.port").expect("ok").try_into().expect("valid port"),
                domain: raw_cfg.get_string("addresses.primary.port").expect("ok"),
            },
            api: ServerCfg {
                ip: raw_cfg.get_string("addresses.api.ip").expect("ok").parse().expect("valid ip"),
                port: raw_cfg.get_int("addresses.api.port").expect("ok").try_into().expect("valid port"),
                domain: raw_cfg.get_string("addresses.api.domain").expect("ok"),
            },
        },
    };

    // Set up logging.
    tracing_subscriber::fmt().init();
    log::info!("Spinning up! (pwd: {:?})", std::env::current_dir().expect("existing directory").as_os_str());

    let app = Router::new()
        .route("/", get(landing::page))
        .route("/blog/*path", get(blog::page))
        .route(
            "/resume",
            get(resume::file)
        )
        .route("/favicon.svg", get(favicon::svg))
        .route("/favicon.ico", get(favicon::ico))
        .route("/public/css/:file", get(static_file::css))
        .route("/public/js/:file", get(static_file::js))
        .route("/public/wasm/:file", get(static_file::wasm))
        .route("/public/png/:file", get(static_file::png))
        .route("/public/jpg/:file", get(static_file::jpg))
        .route("/public/svg/:file", get(static_file::svg))
        .fallback(not_found::page);


    let server = axum::Server::bind(&SocketAddr::new(cfg.servers.primary.ip, cfg.servers.primary.port))
        .serve(app.into_make_service());

    let handle = tokio::spawn(server);
    log::info!("Spun up!");
    handle.await.expect("no issues joining").expect("no issues from server");
}
