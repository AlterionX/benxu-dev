#![feature(type_ascription, result_map_or_else, drain_filter)]

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

#[cfg(not(debug_assertions))]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(debug_assertions)]
mod realtime_log_change {
    #[wasm_bindgen::prelude::wasm_bindgen]
    pub fn set_log_level(level: usize) {
        log::set_max_level([
            log::LevelFilter::Info,
            log::LevelFilter::Trace,
        ][level]);
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

pub fn nav_menu(is_logged_in: bool) -> String {
    if is_logged_in {
        htmlgen::data::Menu(&[
            htmlgen::data::MenuItem {
                text: "Home",
                link: Some("/"),
                children: None,
            },
            htmlgen::data::MenuItem {
                text: "Blog",
                link: Some("/blog"),
                children: None,
            },
            htmlgen::data::MenuItem {
                text: "Create Post",
                link: Some("/blog/editor/new"),
                children: None,
            },
            htmlgen::data::MenuItem {
                text: "Logout",
                link: Some("/blog/logout"),
                children: None,
            },
        ])
    } else {
        htmlgen::data::Menu(&[
            htmlgen::data::MenuItem {
                text: "Home",
                link: Some("/"),
                children: None,
            },
            htmlgen::data::MenuItem {
                text: "Blog",
                link: Some("/blog"),
                children: None,
            },
            htmlgen::data::MenuItem {
                text: "Login",
                link: Some("/blog/login"),
                children: None,
            },
        ])
    }.into_string()
}
fn replace_nav(is_logged_in: bool) {
    let html = nav_menu(is_logged_in);
    let menu_node = seed::body()
        .get_elements_by_tag_name("nav")
        .item(0)
        .unwrap();
    let header_node = menu_node
        .parent_element()
        .unwrap();
    let mock_node = seed::document()
        .create_element("div")
        .unwrap();
    mock_node.set_inner_html(html.as_str());
    let replacement = mock_node.children()
        .item(0)
        .unwrap();
    header_node.replace_child(&replacement, &menu_node).unwrap();
}

fn init(_: Url, orders: &mut impl Orders<M, M>) -> Init<Model> {
    use seed::fetch::Request;
    // Check if logged in.
    const SELF_URL: &'static str = "/api/accounts/me";
    let url = SELF_URL;
    orders.perform_cmd(Request::new(url).fetch_json(move |fo| {
        log::debug!("Detecting if already logged in.");
        M::StoreOpWithAction(
            model::StoreOperations::User(fo),
            |_, res| match res {
                model::StoreOpResult::Success => Some(M::UseLoggedInMenu),
                _ => None,
            },
        )
    }));
    Init::new(Model::default())
}
fn update(msg: M, model: &mut Model, orders: &mut impl Orders<M, M>) {
    log::info!("Processing message {:?}.", msg);
    use locations::*;
    match msg {
        M::NoOp => (),
        M::Grouped(mm) => for m in mm {
            log::debug!("Processing grouped message...");
            update(m, model, orders)
        },
        M::UseLoggedOutMenu => replace_nav(false),
        M::UseLoggedInMenu => replace_nav(true),
        M::ChangePage(loc) => {
            log::debug!("Running page change...");
            loc.prep_page_for_render(&model.loc, &model.store, orders);
            orders.skip();
        },
        M::ChangePageAndUrl(loc) => {
            log::debug!("Running page change (programmatic)...");
            seed::push_route(loc.to_url());
            loc.prep_page_for_render(&model.loc, &model.store, orders);
            orders.skip();
        },
        M::RenderPage(loc) => {
            log::debug!("Running render...");
            match loc.find_redirect(&model.store) {
                Ok(loc) => {
                    orders.skip().send_msg(M::ChangePageAndUrl(loc));
                },
                Err(loc) => {
                    model.loc = loc;
                    orders.force_render_now();
                    if let Some(m) = model.loc.post_load_msgs() {
                        orders.send_msg(m);
                    }
                },
            }
        },
        M::StoreOp(op) => {
            log::debug!("Running store operation...");
            if let Err(e) = model.store.exec(op) {
                log::error!("Store caused error {:?}.", e);
            } else {
                log::trace!("Success.");
            }
            orders.skip();
        },
        M::StoreOpWithAction(op, f) => {
            log::debug!("Store operation with follow up action detected.");
            for m in f(&model.store, model.store.exec(op).into()) {
                orders.send_msg(m);
            }
            orders.skip();
        },
        // TODO remove boilerplate with macro?
        M::Listing(msg) => {
            log::debug!("Handling editor msg...");
            if let Location::Listing(s) = &mut model.loc {
                listing::update(msg, s, &model.store, &mut orders.proxy(M::Listing));
            } else {
                orders.skip();
            }
        },
        M::Viewer(msg) => {
            log::debug!("Handling viewer msg...");
            if let Location::Viewer(s) = &mut model.loc {
                viewer::update(msg, s, &model.store, &mut orders.proxy(M::Viewer));
            } else {
                orders.skip();
            }
        },
        M::Login(msg) => {
            log::debug!("Handling login msg...");
            if let Location::Login(s) = &mut model.loc {
                login::update(msg, s, &model.store, &mut orders.proxy(M::Login));
            } else {
                orders.skip();
            }
        },
        M::Editor(msg) => {
            log::debug!("Handling editor msg...");
            if let Location::Editor(s) = &mut model.loc {
                editor::update(msg, s, &model.store, &mut orders.proxy(M::Editor));
            } else {
                orders.skip();
            }
        },
    }
}
fn routes(url: Url) -> Option<M> {
    log::info!("Attempting to route {:?}.", url);
    messages::RouteMatch::from(url).into_inner()
}
fn view(m: &Model) -> impl View<M> {
    m.to_view()
}

#[wasm_bindgen(start)]
pub fn begin_app() {
    use wasm_bindgen::JsCast;
    setup_logger();
    log::info!("Starting up application.");
    let body_tag = seed::body();
    let tag = match body_tag.query_selector("main") {
        Ok(Some(main_tag)) => main_tag.dyn_into().unwrap(),
        _ => body_tag,
    };

    let app = seed::App::build(init, update, view)
        .sink(update)
        .mount(tag)
        .takeover_mount(true)
        .routes(routes);
    log::info!("App built. Running.");
    app.finish().run();
}
