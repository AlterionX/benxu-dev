#![feature(type_ascription)]

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

fn init(url: Url, orders: &mut impl Orders<M>) -> Model {
    if let Some(msg) = routes(url) {
        orders.send_msg(msg);
    }
    Model::default()
}

fn update(msg: M, model: &mut Model, orders: &mut impl Orders<M>) {
    match msg {
        M::ChangePage(loc) => match loc.find_redirect(&model) {
            Ok(changed_loc) => {
                log("Attempt to get another page.");
                orders.skip().send_msg(M::ChangePage(changed_loc));
            },
            Err(loc) => match loc.required_data_fetch(&model) {
                Ok(fetch_req) => {
                    log("Attempt to fetch data.");
                    orders.skip().perform_cmd(fetch_req);
                },
                Err(loc) => {
                    log("Attempt to render page directly, since data is already present.");
                    orders.skip().send_msg(M::RenderPage(loc));
                },
            }
        },
        M::RenderPage(loc) => {
            model.loc = loc;
        },
        M::DataFetched(fetched) => {
            log("Data fetched. Will now render page.");
            let loc = fetched.to_loc();
            if let Err(m) = model.update_with(fetched) {
                orders.send_msg(m);
            } else {
                orders.send_msg(M::RenderPage(loc));
            }
        },
        M::Login(login_msg) => {
            log(format!("Handling login update.").as_str());
            if let locations::Location::Login(s) = &mut model.loc {
                locations::login::update(login_msg, s, &mut model.store, orders);
            } else {
                orders.skip();
            }
        },
        _ => (),
    }
}

fn routes(url: seed::Url) -> Option<M> {
    log(format!("Routing: {:?}", url).as_str());
    messages::RouteMatch::from(url).into_inner()
}

fn view(m: &Model) -> impl View<M> {
    log(format!("Converting location {:?} to view.", m.loc).as_str());
    m.to_view()
}

#[wasm_bindgen(start)]
pub fn render() {
    log("Beginning render.");
    let body_tag = seed::body();

    let app = seed::App::build(init, update, view);
    let app = match body_tag.query_selector("main") {
        Ok(Some(main_tag)) => app.mount(main_tag),
        _ => app.mount(body_tag),
    };
    let app = app.routes(routes).finish();
    app.run();
}
