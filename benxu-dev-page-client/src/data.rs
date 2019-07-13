use maud::{Markup, html, Render};
use chrono::{Utc, Datelike};

pub struct LogoLink<'a> {
    pub url: &'a str,
    pub logo: &'a str,
    pub alt_text: &'a str,
}
impl<'a> Render for LogoLink<'a> {
    fn render(&self) -> Markup {
        html! {
            a href=(self.url) {
                img.link-logo src=(self.logo) alt=(self.alt_text);
            }
        }
    }
}

pub struct MetaData<'a> {
    pub lang: &'a str,
    pub charset: &'a str,
    pub scripts: &'a [Script<'a>],
    pub css: &'a [Css<'a>],
    pub title: &'a str,
    pub description: &'a str,
    pub copyright: Copyright<'a>,
    pub menu: Option<&'a Menu<'a>>,
    pub contact: Option<&'a Contact<'a>>,
    pub logo: Option<&'a Logo<'a>>,
}
impl <'a> MetaData<'a> {
    pub fn new() -> Self {
        let meta = MetaData::default();
        meta
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
            logo: None,
        }
    }
}
pub struct Logo<'a> {
    pub src: &'a str,
}
impl<'a> Render for Logo<'a> {
    fn render(&self) -> Markup {
        html! {
            img.logo src=(self.src);
        }
    }
}
pub struct Script<'a> {
    pub src: &'a str,
}
impl<'a> Render for Script<'a> {
    fn render(&self) -> Markup {
        html! {
            script href={ "/static/js/"(self.src) };
        }
    }
}
pub struct Css<'a> {
    pub src: &'a str,
}
impl<'a> Render for Css<'a> {
    fn render(&self) -> Markup {
        html! {
            link rel="stylesheet" href={ "/public/css/"(self.src)".css" };
        }
    }
}
pub struct Email<'a> {
    pub user: &'a str,
    pub domain: &'a str,
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
    pub email: &'a[Email<'a>],
    pub phone: &'a[PhoneNumber<'a>],
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
    pub first: &'a str,
    pub middle: Option<&'a str>,
    pub last: &'a str,
    pub nicknames: &'a[&'a str],
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
    pub name: &'a Name<'a>,
    pub icon: &'a str,
    pub rights_clause: &'a str,
}
impl<'a> Render for Copyright<'a> {
    fn render(&self) -> Markup {
        let year = Utc::now().year();
        let start_year = year - 1;
        let end_year = year + 1;
        html! {
            p.copyright { (self.icon) " " (start_year) "-" (end_year) " " (self.name) ". " (self.rights_clause) "." }
        }
    }
}
pub struct MenuItem<'a> {
    pub text: &'a str,
    pub link: Option<&'a str>,
    pub children: Option<&'a Menu<'a>>,
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
pub struct Menu<'a>(pub &'a[MenuItem<'a>]);
impl <'a> Render for Menu<'a> {
    fn render(&self) -> Markup {
        html! {
            ul.menu {
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

