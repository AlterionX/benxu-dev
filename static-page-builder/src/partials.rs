//! A series of components used across the site.
use crate::data::PageMetaData;
use maud::{html, Markup, DOCTYPE};

/// The `<head>` portion of the webpage.
pub fn head(meta: &PageMetaData) -> Markup {
    html! {
        head {
            meta charset=(meta.charset);
            title { (meta.title) }
            meta name="description" content=(meta.description);
            meta name="viewport" content="width=device-width, initial-scale=1";
            meta name="theme-color" content=(meta.theme_color);
            @for f in meta.favicons {
                (f)
            }
            @for css in meta.css {
                (css)
            }
            @for js in meta.scripts {
                (js)
            }
        }
    }
}

/// The `<header>` portion of the webpage. Displays logos and menus.
pub fn header(meta: &PageMetaData) -> Markup {
    html! {
        header.site-header {
            @if let Some(logo) = meta.logo {
                (logo)
            }
            @if let Some(menu) = meta.menu {
                (menu)
            }
        }
    }
}

/// The `<footer>` portion of the webpage. Displays copyright and contact information.
pub fn footer(meta: &PageMetaData) -> Markup {
    html! {
        footer.site-footer {
            @if let Some(contact) = meta.contact {
                (contact)
            }
            (meta.copyright)
        }
    }
}

/// The `<body>` portion of the webpage. Wraps the main content and offsets it from the header and
/// footer.
pub fn body(m: Markup, meta: &PageMetaData) -> Markup {
    html! {
        body {
            div.bg-img {}
            (header(meta))
            main.site-body {
                (m)
            }
            (footer(meta))
        }
    }
}

/// A template of the page with its `<DOCTYPE>` and `<html>` tags.
pub fn page(m: Markup, meta: &PageMetaData) -> Markup {
    html! {
        (DOCTYPE)
        html lang=(meta.lang) {
            (head(&meta))
            (body(m, &meta))
        }
    }
}

/// A template of a page. Uses default [`MetaData`](crate::data::MetaData) if not provided.
pub fn basic_page(m: Markup, meta_data: Option<&PageMetaData>) -> Markup {
    let store;
    let meta;
    if let Some(meta_ref) = meta_data {
        meta = meta_ref;
    } else {
        store = PageMetaData::default();
        meta = &store;
    }
    page(m, meta)
}
