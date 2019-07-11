use maud::{Markup, html, Render};
use chrono::{Utc, Datelike};

pub struct MetaData<'a> {
    pub lang: &'a str,
    pub charset: &'a str,
    pub scripts: &'a [Script<'a>],
    pub css: &'a [Css<'a>],
    pub title: &'a str,
    pub description: &'a str,
    pub copyright: Copyright<'a>,
    pub menu: Option<&'a Menu<'a>>,
    pub contact: Option<&'a Contact<'a>>
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
pub struct Script<'a> {
    src: &'a str,
}
impl<'a> Render for Script<'a> {
    fn render(&self) -> Markup {
        html! {
            script href={ "/static/js/"(self.src) };
        }
    }
}
pub struct Css<'a> {
    src: &'a str,
}
impl<'a> Render for Css<'a> {
    fn render(&self) -> Markup {
        html! {
            link rel="stylesheet" href={ "/static/css/"(self.src) };
        }
    }
}
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
pub struct Name<'a> {
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
pub struct Copyright<'a> {
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

