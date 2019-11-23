#![feature(type_ascription, result_map_or_else, drain_filter)]

#[macro_use]
extern crate seed;

use seed::prelude::*;

mod model;
use model::Model;
mod messages;
use messages::M;

mod locations;
mod requests;
mod shared;

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
        ])
    }
    .into_string()
}
fn replace_nav(is_logged_in: bool) {
    let html = nav_menu(is_logged_in);
    let menu_node = seed::body()
        .get_elements_by_tag_name("nav")
        .item(0)
        .unwrap();
    let header_node = menu_node.parent_element().unwrap();
    let mock_node = seed::document().create_element("div").unwrap();
    mock_node.set_inner_html(html.as_str());
    let replacement = mock_node.children().item(0).unwrap();
    header_node.replace_child(&replacement, &menu_node).unwrap();
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
        M::UseLoggedOutMenu => replace_nav(false),
        M::UseLoggedInMenu => replace_nav(true),
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
            if let Err(e) = model.store.exec(op) {
                log::error!("Store caused error {:?}.", e);
            } else {
                log::trace!("Success.");
            }
            orders.skip();
        }
        M::StoreOpWithAction(op, f) => {
            log::debug!("Store operation with follow up action detected.");
            if let Some(m) = f(&model.store, model.store.exec(op).into()) {
                update(m, model, orders);
            }
        }
        // TODO remove boilerplate with macro?
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
        M::Login(msg) => {
            log::debug!("Handling login msg...");
            if let Location::Login(s) = &mut model.loc {
                login::update(msg, s, &model.store, &mut orders.proxy(M::Login));
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

fn init_app(fo: seed::fetch::FetchObject<db_models::users::DataNoMeta>) {
    use wasm_bindgen::JsCast;
    log::info!("Starting up application.");
    let body_tag = seed::body();
    let tag = match body_tag.query_selector("main") {
        Ok(Some(main_tag)) => main_tag.dyn_into().unwrap(),
        _ => body_tag,
    };

    let model = fo
        .result
        .ok()
        .and_then(|fd| fd.data.ok())
        .map(Model::with_user)
        .unwrap_or_else(Model::default);
    let app = seed::App::build(
        move |_, orders| {
            if model.store.user.is_some() {
                orders.send_msg(M::UseLoggedInMenu);
            }
            Init {
                mount_type: MountType::Takeover,
                ..Init::new(model)
            }
        },
        update,
        view,
    )
    .sink(update)
    .mount(tag)
    .routes(routes);
    log::info!("App built. Running.");
    app.build_and_start();
}
async fn init_with_current_user() -> Result<JsValue, JsValue> {
    const SELF_URL: &str = "/api/accounts/me";
    log::info!("Detecting if already logged in...");
    let req = seed::Request::new(SELF_URL)
        .fetch_json(|f: seed::fetch::FetchObject<db_models::users::DataNoMeta>| f);
    let fo = future_futures::compat::Compat01As03::new(req)
        .await
        .unwrap_or_else(|e| e);
    init_app(fo);
    Ok(JsValue::UNDEFINED)
}

#[wasm_bindgen(start)]
pub fn begin_app() -> Result<(), JsValue> {
    setup_logger();
    // TODO change this to something more... direct. And sensible.
    Err(wasm_bindgen_futures::future_to_promise(init_with_current_user()).into())
}
