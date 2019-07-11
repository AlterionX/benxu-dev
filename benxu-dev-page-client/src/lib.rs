#![feature(proc_macro_hygiene, decl_macro)]

mod data;
mod partials;

use maud::{Markup, html};

pub fn index(msg: &str) -> Markup {
    partials::basic_page(html! {
        p { (msg) }
    }, None)
}

pub mod resume {
}
pub mod links {
}
pub mod contacts {
}
pub mod projects {
}
