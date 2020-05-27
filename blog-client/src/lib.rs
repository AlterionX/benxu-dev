#![feature(drain_filter, try_trait, move_ref_pattern)]

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
        M::UrlChanged(subs::UrlChanged(url)) => {
            log::debug!("Processing url {:?} change...", url.path());
            if let Some(m) = routes(url) {
                orders.skip().send_msg(m);
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
            model.store.exec(op);
            orders.skip();
        }
        M::StoreOpWithAction(op, f) => {
            log::debug!("Store operation with follow up action detected.");
            model.store.exec(op);
            let m = f.into_inner()(&model.store);
            update(m, model, orders);
        }
        M::StoreOpWithMessage(op, m) => {
            log::debug!("Store operation with follow up message detected.");
            model.store.exec(op);
            update(m(), model, orders);
        }
        // TODO remove boilerplate with macro?
        M::Location(msg) => {
            log::debug!("Handling location msg...");
            locations::update(msg, &mut model.loc, &model.store, orders);
            log::trace!("Location msg handled.");
        }
    }
}

fn init(url: Url, orders: &mut impl Orders<M, M>) -> Model {
    log::info!("Running init with url {:?}", url);
    orders
        .subscribe(M::UrlChanged)
        .notify(subs::UrlChanged(url.clone()))
        .subscribe(|login| M::Location(locations::M::Login(login)))
        .subscribe(|listing| M::Location(locations::M::Listing(listing)))
        .subscribe(|viewer| M::Location(locations::M::Viewer(viewer)))
        .subscribe(|editor| M::Location(locations::M::Editor(editor)))
        .perform_cmd(async {
            let user = locations::login::find_current_user().await?;
            Some(M::Grouped(vec![
                M::StoreOp(model::StoreOperations::User(user)),
                M::ChangeMenu(shared::Authorization::LoggedIn),
            ]))
        });
    if let Some(m) = routes(url) {
        orders.send_msg(m);
    }
    Model::default()
}

fn view(m: &Model) -> impl IntoNodes<M> {
    let Model { loc: l, store: s } = m;
    log::info!("Rendering location {:?} with global state {:?}.", l, s);
    locations::view(l, s)
}

fn routes(url: Url) -> Option<M> {
    log::info!("Attempting to route {:?}.", url);
    messages::RouteMatch::from(url).into_inner()
}

fn init_app() {
    log::info!("Starting up application.");

    let tag = {
        let body_tag = seed::body();
        match body_tag.query_selector("main") {
            Ok(Some(main_tag)) => main_tag
                .dyn_into()
                .tap_err(|_| log::error!("Main tag is not an HtmlElement!"))
                .unwrap(),
            _ => body_tag,
        }
    };
    seed::App::start(tag, init, update, view);
    log::info!("App built. Now running.");
}

#[wasm_bindgen(start)]
pub fn begin_app() {
    logging::setup_logger();
    init_app();
}
