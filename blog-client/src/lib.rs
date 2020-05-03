#![feature(drain_filter, try_trait)]

#[macro_use]
extern crate seed;

mod locations;
mod messages;
mod model;
mod requests;
mod shared;

use messages::M;
use model::Model;
use seed::prelude::*;

#[cfg(not(debug_assertions))]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(debug_assertions)]
pub mod realtime_log_change {
    #[wasm_bindgen::prelude::wasm_bindgen]
    pub fn set_log_level(level: usize) {
        log::set_max_level([log::LevelFilter::Info, log::LevelFilter::Trace][level]);
    }
}
#[cfg(not(debug_assertions))]
fn setup_logger() {
    crate::log("Is release mode. Logging disabled.");
}
#[cfg(debug_assertions)]
fn setup_logger() {
    fern::Dispatch::new()
        .chain(fern::Output::call(console_log::log))
        .apply()
        .unwrap()
}

fn update(msg: M, model: &mut Model, orders: &mut impl Orders<M, M>) {
    log::info!("Processing message {:?}.", msg);
    use locations::*;
    match msg {
        M::NoOp => (),
        M::Grouped(mm) => {
            for m in mm {
                log::debug!("Processing grouped message...");
                update(m, model, orders)
            }
        }
        M::ChangeMenu(is_logged_in) => shared::views::replace_nav(is_logged_in),
        M::ChangePage(loc) => {
            log::debug!("Running page change...");
            loc.prep_page_for_render(&model.loc, &model.store, orders);
            orders.skip();
        }
        M::ChangePageAndUrl(loc) => {
            log::debug!("Running page change (programmatic)...");
            seed::push_route(loc.to_url());
            loc.prep_page_for_render(&model.loc, &model.store, orders);
            orders.skip();
        }
        M::RenderPage(loc) => {
            log::debug!("Running render...");
            match loc.find_redirect(&model.store) {
                Ok(loc) => {
                    orders.skip().send_msg(M::ChangePageAndUrl(loc));
                }
                Err(loc) => {
                    model.loc = loc;
                    orders.force_render_now();
                    if let Some(m) = model.loc.post_load_msgs() {
                        orders.send_msg(m);
                    }
                }
            }
        }
        M::StoreOp(op) => {
            log::debug!("Running store operation...");
            if let model::StoreOpResult::Failure(e) = model.store.exec(op) {
                log::error!("Store caused error {:?}.", e);
            } else {
                log::trace!("Success.");
            }
            orders.skip();
        }
        M::StoreOpWithAction(op, f) => {
            log::debug!("Store operation with follow up action detected.");
            if let Some(m) = f(&model.store, model.store.exec(op)) {
                update(m, model, orders);
            }
        }
        // TODO remove boilerplate with macro?
        M::Login(msg) => {
            log::debug!("Handling login msg...");
            if let Location::Login(s) = &mut model.loc {
                login::update(msg, s, &model.store, &mut orders.proxy(M::Login));
            } else {
                orders.skip();
            }
        }
        M::Listing(msg) => {
            log::debug!("Handling editor msg...");
            if let Location::Listing(s) = &mut model.loc {
                listing::update(msg, s, &model.store, &mut orders.proxy(M::Listing));
            } else {
                orders.skip();
            }
        }
        M::Viewer(msg) => {
            log::debug!("Handling viewer msg...");
            if let Location::Viewer(s) = &mut model.loc {
                viewer::update(msg, s, &model.store, &mut orders.proxy(M::Viewer));
            } else {
                orders.skip();
            }
        }
        M::Editor(msg) => {
            log::debug!("Handling editor msg...");
            if let Location::Editor(s) = &mut model.loc {
                editor::update(msg, s, &model.store, &mut orders.proxy(M::Editor));
            } else {
                orders.skip();
            }
        }
    }
}
fn routes(url: Url) -> Option<M> {
    log::info!("Attempting to route {:?}.", url);
    messages::RouteMatch::from(url).into_inner()
}
fn view(m: &Model) -> impl View<M> {
    m.to_view()
}

fn after_mount(_: Url, orders: &mut impl Orders<M, M>) -> AfterMount<Model> {
    orders.perform_cmd(async {
        let user = locations::login::find_current_user().await;
        Ok(M::StoreOp(model::StoreOperations::User(user)))
    });
    AfterMount::new(Model::default()).url_handling(UrlHandling::PassToRoutes)
}

fn init_app() {
    use wasm_bindgen::JsCast;
    log::info!("Starting up application.");
    let body_tag = seed::body();
    let tag = match body_tag.query_selector("main") {
        Ok(Some(main_tag)) => main_tag.dyn_into().unwrap(),
        _ => body_tag,
    };

    let app = seed::App::builder(update, view)
        .sink(update)
        .routes(routes)
        .before_mount(move |_| {
            BeforeMount::new()
                .mount_point(tag)
                .mount_type(MountType::Takeover)
        })
        .after_mount(after_mount);
    log::info!("App built. Now running.");
    app.build_and_start();
}

#[wasm_bindgen(start)]
pub fn begin_app() {
    setup_logger();
    init_app();
}
