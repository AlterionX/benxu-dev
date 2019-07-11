use crate::data::*;
use maud::{Markup, html, DOCTYPE};

fn head(meta: &MetaData) -> Markup {
    html! {
        head {
            meta charset=(meta.charset);
            title { (meta.title) }
            meta name="description" content=(meta.description);
        }
    }
}
fn header(meta: &MetaData) -> Markup {
    html! {
        header {
            @if let Some(menu) = meta.menu {
                (menu)
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
fn footer(meta: &MetaData) -> Markup {
    html! {
        footer {
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
            (header(meta))
            main {
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
