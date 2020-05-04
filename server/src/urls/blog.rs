//! Marshalls the data between the [`blog_client`](../blog_client) and [`blog_db`](../blog_db).

mod accounts;
mod capabilities;
mod credentials;
mod login;
mod posts;

use crate::util::auth;
use maud::Markup;
use rocket::Route;

/// Handler for serving the primary web app.
#[get("/<_path..>")]
pub fn get(
    _path: Option<rocket::http::uri::Segments>,
    c: Option<auth::UnverifiedCapabilities>,
) -> Markup {
    // TODO set based on capabilities
    htmlgen::index(c.is_some())
}

/// Handler for serving the primary web app for when there is no path.
#[get("/")]
pub fn get_unadorned(c: Option<auth::UnverifiedCapabilities>) -> Markup {
    // TODO set based on capabilities
    htmlgen::index(c.is_some())
}

/// Provides a [`Vec`] of [`Route`]s to be attached with [`rocket::Rocket::mount()`]. Used for the
/// SPA endpoints.
pub fn spa_routes() -> Vec<Route> {
    routes![get, get_unadorned]
}
/// Provides a [`Vec`] of [`Route`]s to be attached with [`rocket::Rocket::mount()`]. Used for the
/// api endpoints.
pub fn api_routes() -> Vec<Route> {
    routes![
        posts::get,
        posts::post,
        posts::post::get,
        posts::post::patch,
        posts::post::delete,
        posts::post::publish,
        posts::post::archive,
        accounts::post,
        accounts::account::get,
        accounts::account::get_self,
        accounts::account::patch,
        accounts::account::delete,
        login::post,
        login::delete,
        credentials::pws::post,
        credentials::pws::pw::patch,
        credentials::pws::pw::delete,
        capabilities::post,
        capabilities::delete,
        capabilities::capability::get,
        capabilities::capability::delete,
    ]
}

/// Functions serving the initial blog page, before it gets taken over by
/// [`blog_client`](blog_client).
mod htmlgen {
    use maud::{html, Markup};
    use page_client::{data, partials};

    /// Create a basic menu.
    pub fn menu() -> Option<data::Menu<'static>> {
        Some(data::Menu(&[
            data::MenuItem {
                text: "Home",
                link: Some("/"),
                children: None,
            },
            data::MenuItem {
                text: "Blog",
                link: Some("/blog"),
                children: None,
            },
        ]))
    }
    /// Create a basic menu for a special user.
    pub fn logged_in_menu() -> Option<data::Menu<'static>> {
        Some(data::Menu(&[
            data::MenuItem {
                text: "Blog",
                link: Some("/blog"),
                children: None,
            },
            data::MenuItem {
                text: "Profile",
                link: Some("/blog/profile"),
                children: None,
            },
        ]))
    }

    /// Returns a list of [`Css`](crate::data::Css) scripts that go in my blog page.
    fn css_scripts<'a>() -> [data::Css<'a>; 4] {
        [
            data::Css::Critical { src: "reset" },
            data::Css::Critical { src: "typography" },
            data::Css::Critical { src: "main" },
            data::Css::NonCritical { src: "blog" },
        ]
    }

    /// Returns a basic page, as everything will be managed by `blog_client`.
    pub fn index(is_logged_in: bool) -> Markup {
        let (glue, load) = data::Script::wasm_bindgen_loader("blog_client");
        let js_scripts = [
            data::Script::External(glue.as_str()),
            data::Script::Embedded(load.as_str()),
        ];
        let css_scripts = css_scripts();
        let menu = if is_logged_in {
            logged_in_menu()
        } else {
            menu()
        };
        let logo = crate::shared_html::logo_markup();
        let meta = data::MetaData::builder()
            .scripts(&js_scripts[..])
            .css(&css_scripts[..])
            .menu(menu.as_ref())
            .logo(logo.as_ref())
            .build();
        partials::basic_page(html! { "Loading. Please wait..." }, Some(&meta))
    }
}
