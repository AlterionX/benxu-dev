#![feature(type_ascription, result_map_or_else)]

#[macro_use]
extern crate seed;

use wasm_bindgen::prelude::*;
use seed::prelude::*;

mod model;
use model::Model;
mod messages;
use messages::M;

mod shared;
mod locations;
mod requests;

#[wasm_bindgen]
extern "C" {
    /// Binding to javascript's `console.log()`.
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(a: &str);
}

fn init(url: Url, orders: &mut impl Orders<M, M>) -> Model {
    if let Some(msg) = routes(url) {
        orders.send_msg(msg);
    }
    Model::default()
}
fn update(msg: M, model: &mut Model, orders: &mut impl Orders<M, M>) {
    use locations::*;
    match msg {
        M::ChangePage(loc) => { loc.prep_page_for_render(&model.loc, &model.store, orders); },
        M::RenderPage(loc) => { model.loc = loc; },
        M::StoreOp(op, f) => {
            log("Data fetched. Will now render page.");
            f(model.store.exec(op)).map(|m| orders.send_msg(m));
        },
        // TODO remove boilerplate with macro?
        M::Listing(msg) => {
            log(format!("Handling editor update.").as_str());
            if let Location::Listing(s) = &mut model.loc {
                listing::update(msg, s, &model.store, &mut orders.proxy(M::Listing));
            } else {
                orders.skip();
            }
        },
        M::Viewer(msg) => {
            log(format!("Handling viewer update.").as_str());
            if let Location::Viewer(s) = &mut model.loc {
                viewer::update(msg, s, &model.store, &mut orders.proxy(M::Viewer));
            } else {
                orders.skip();
            }
        },
        M::Login(msg) => {
            log(format!("Handling login update.").as_str());
            if let Location::Login(s) = &mut model.loc {
                login::update(msg, s, &model.store, &mut orders.proxy(M::Login));
            } else {
                orders.skip();
            }
        },
        M::Editor(msg) => {
            log(format!("Handling editor update.").as_str());
            if let Location::Editor(s) = &mut model.loc {
                editor::update(msg, s, &model.store, &mut orders.proxy(M::Editor));
            } else {
                orders.skip();
            }
        },
    }
}
fn routes(url: Url) -> Option<M> {
    log(format!("Routing: {:?}", url).as_str());
    messages::RouteMatch::from(url).into_inner()
}
fn view(m: &Model) -> impl View<M> {
    log(format!("Converting location {:?} to view.", m.loc).as_str());
    m.to_view()
}

#[wasm_bindgen(start)]
pub fn begin_app() {
    log("Beginning render.");

    let app = seed::App::build(init, update, view);
    let body_tag = seed::body();
    let app = match body_tag.query_selector("main") {
        Ok(Some(main_tag)) => app.mount(main_tag),
        _ => app.mount(body_tag),
    };
    let app = app.routes(routes).finish();
    app.run();
}
