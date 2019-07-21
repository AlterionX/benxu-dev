use crate::data::*;
use maud::{Markup, html, DOCTYPE};

fn head(meta: &MetaData) -> Markup {
    html! {
        head {
            meta charset=(meta.charset);
            title { (meta.title) }
            meta name="description" content=(meta.description);
            meta name="viewport" content="width=device-width, initial-scale=1";
            meta name="theme-color" content=(meta.theme_color);
            @for css in meta.css {
                (css)
            }
            @for js in meta.scripts {
                (js)
            }
        }
    }
}
fn header(meta: &MetaData) -> Markup {
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
fn footer(meta: &MetaData) -> Markup {
    html! {
        footer.site-footer {
            @if let Some(contact) = meta.contact {
                (contact)
            }
            (meta.copyright)
        }
    }
}
fn body(m: Markup, meta: &MetaData) -> Markup {
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
fn page(m: Markup, meta: &MetaData) -> Markup {
    html! {
        (DOCTYPE)
        html lang=(meta.lang) {
            (head(&meta))
            (body(m, &meta))
        }
    }
}
pub fn basic_page(m: Markup, meta_data: Option<&MetaData>) -> Markup {
    let store;
    let meta;
    if let Some(meta_ref) = meta_data {
        meta = meta_ref;
    } else {
        store = MetaData::default();
        meta = &store;
    }
    page(m, meta)
}
