#![feature(drain_filter, try_trait)]

#[macro_use]
extern crate seed;

mod logging;
#[cfg(debug_assertions)]
pub use logging::realtime_log_change;

mod locations;
mod messages;
mod model;
mod requests;
mod shared;

use messages::M;
use model::Model;
use seed::prelude::*;
use tap::*;

fn update(msg: M, model: &mut Model, orders: &mut impl Orders<M, M>) {
    log::info!("Processing message {:?}.", msg);
    match msg {
        M::NoOp => (),
        M::Grouped(mm) => {
            for m in mm {
                log::debug!("Processing grouped message...");
                update(m, model, orders)
            }
        }
        M::ChangeMenu(is_logged_in) => shared::views::replace_nav(is_logged_in),
        M::ChangePageAndUrl(loc) => {
            log::debug!("Running page change (programmatic)...");
            seed::push_route(loc.to_url());
            orders.skip().send_msg(M::ChangePage(loc));
        }
        M::ChangePage(loc) => {
            log::debug!("Running page change...");
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
        M::Location(msg) => {
            log::debug!("Handling location msg...");
            locations::update(msg, &mut model.loc, &model.store, &mut orders.proxy(M::Location));
            log::trace!("Location msg handled.");
        }
    }
}

fn view(m: &Model) -> impl View<M> {
    let Model { loc: l, store: s } = m;
    log::info!("Rendering location {:?} with global state {:?}.", l, s);
    locations::view(l, s)
}

fn routes(url: Url) -> Option<M> {
    log::info!("Attempting to route {:?}.", url);
    messages::RouteMatch::from(url).into_inner()
}

fn before_mount(_: Url) -> BeforeMount {
    use wasm_bindgen::JsCast;
    let body_tag = seed::body();
    let tag = match body_tag.query_selector("main") {
        Ok(Some(main_tag)) => main_tag
            .dyn_into()
            .tap_err(|_| log::error!("Main tag is not an HtmlElement!"))
            .unwrap(),
        _ => body_tag,
    };

    BeforeMount::new()
        .mount_point(tag)
        .mount_type(MountType::Takeover)
}

fn after_mount(_: Url, orders: &mut impl Orders<M, M>) -> AfterMount<Model> {
    orders.perform_cmd(async {
        let user = locations::login::find_current_user().await;
        Ok(M::StoreOp(model::StoreOperations::User(user)))
    });
    AfterMount::new(Model::default()).url_handling(UrlHandling::PassToRoutes)
}

fn init_app() {
    log::info!("Starting up application.");

    let app = seed::App::builder(update, view)
        .sink(update)
        .routes(routes)
        .before_mount(before_mount)
        .after_mount(after_mount);
    log::info!("App built. Now running.");
    app.build_and_start();
}

#[wasm_bindgen(start)]
pub fn begin_app() {
    logging::setup_logger();
    init_app();
}
