use maud::{Markup, html, DOCTYPE, Render};
use chrono::{Utc, Datelike};

pub struct MetaData<'a> {
    lang: &'a str,
    charset: &'a str,
    scripts: &'a [Script],
    css: &'a [Css],
    title: &'a str,
    description: &'a str,
    copyright: Copyright<'a>,
    menu: Option<&'a Menu<'a>>,
    contact: Option<&'a Contact<'a>>
}
impl <'a> MetaData<'a> {
    pub fn new() -> Self {
        let meta = MetaData::default();
        return meta;
    }
}
impl <'a> Default for MetaData<'a> {
    fn default() -> Self {
        MetaData {
            lang: "en-US",
            charset: "UTF-8",
            scripts: &[],
            css: &[],
            title: "Benjamin Xu",
            description: "Benjamin Xu's personal site.",
            copyright: Copyright {
                name: &Name {
                    first: "Benjamin",
                    middle: Some("Peiyan"),
                    last: "Xu",
                    nicknames: &[],
                },
                icon: "©",
                rights_clause: "All rights reserved",
            },
            menu: None,
            contact: None,
        }
    }
}
pub struct Script;
pub struct Css;
pub struct Email<'a> {
    user: &'a str,
    domain: &'a str,
}
pub enum PhoneNumber<'a> {
    US {
        area_code: u16,
        prefix: u16,
        line_number: u16,
        icon: &'a str,
    }
}
pub struct Contact<'a> {
    email: &'a[Email<'a>],
    phone: &'a[PhoneNumber<'a>],
}
impl<'a> Render for Contact<'a> {
    fn render(&self) -> Markup {
        let year = Utc::now().year();
        html! {
            p { "(self.icon) (year - 1)-(year + 1) (self.name). (self.rights_clause)." }
        }
    }
}
struct Name<'a> {
    first: &'a str,
    middle: Option<&'a str>,
    last: &'a str,
    nicknames: &'a[&'a str],
}
impl<'a> Render for Name<'a> {
    fn render(&self) -> Markup {
        html! {
            (self.first) " " @if let Some(middle) = self.middle {
                @if let Some(initial) = middle.chars().next() {
                    (initial)
                }
            } ". " (self.last)
        }
    }
}
struct Copyright<'a> {
    name: &'a Name<'a>,
    icon: &'a str,
    rights_clause: &'a str,
}
impl<'a> Render for Copyright<'a> {
    fn render(&self) -> Markup {
        let year = Utc::now().year();
        let start_year = year - 1;
        let end_year = year + 1;
        html! {
            p { (self.icon) " " (start_year) "-" (end_year) " " (self.name) ". " (self.rights_clause) "." }
        }
    }
}
pub struct MenuItem<'a> {
    text: &'a str,
    link: Option<&'a str>,
    children: Option<&'a Menu<'a>>,
}
impl<'a> MenuItem<'a> {
    fn render_possible_link(link: Option<&str>, text: &str) -> Markup {
        html! {
            @if let Some(link) = link {
                a href=(link) { (text) }
            } @else {
                (text)
            }
        }
    }
}
impl <'a> Render for MenuItem<'a> {
    fn render(&self) -> Markup {
        html! {
            li {
                (MenuItem::render_possible_link(self.link, self.text))
                @if let Some(children) = self.children {
                    (children)
                }
            }
        }
    }
}
pub struct Menu<'a>([MenuItem<'a>]);
impl <'a> Render for Menu<'a> {
    fn render(&self) -> Markup {
        html! {
            ul {
                @for item in self.0.iter() {
                    (item)
                }
            }
        }
    }
}
impl <'a> Render for &Menu<'a> {
    fn render(&self) -> Markup {
        (*self).render()
    }
}

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
