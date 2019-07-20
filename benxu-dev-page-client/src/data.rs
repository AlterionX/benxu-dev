use maud::{Markup, html, Render, PreEscaped};
use typed_builder::TypedBuilder;
use chrono::{Utc, Datelike};

pub struct LogoLink<'a> {
    pub url: &'a str,
    pub logo: &'a str,
    pub alt_text: &'a str,
    pub text: &'a str,
}
impl<'a> Render for LogoLink<'a> {
    fn render(&self) -> Markup {
        html! {
            a.link-anchor href=(self.url) {
                img.link-logo src=(self.logo) alt=(self.alt_text); (self.text)
            }
        }
    }
}

#[derive(TypedBuilder)]
pub struct MetaData<'a> {
    #[builder(default="en-US")]
    pub lang: &'a str,
    #[builder(default="UTF-8")]
    pub charset: &'a str,
    #[builder(default_code="&[]")]
    pub scripts: &'a [Script<'a>],
    #[builder(default=&[])]
    pub css: &'a [Css<'a>],
    #[builder(default="Benjamin Xu")]
    pub title: &'a str,
    #[builder(default="Benjamin Xu's personal site.")]
    pub description: &'a str,
    #[builder(default_code=r#"Copyright {
        name: &Name {
            first: "Benjamin",
            middle: Some("Peiyan"),
            last: "Xu",
            nicknames: &[],
        },
        icon: "©",
        rights_clause: "All rights reserved",
    }"#)]
    pub copyright: Copyright<'a>,
    #[builder(default)]
    pub menu: Option<&'a Menu<'a>>,
    #[builder(default)]
    pub contact: Option<&'a Contact<'a>>,
    #[builder(default)]
    pub logo: Option<&'a Logo<'a>>,
}
impl <'a> Default for MetaData<'a> {
    fn default() -> Self {
        Self::builder().build()
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
pub enum Script<'a> {
    External(&'a str),
    Embedded(&'a str),
}
impl<'a> Render for Script<'a> {
    fn render(&self) -> Markup {
        match self {
            Script::External(src) => html! { script defer?[true] src={ "/public/js/"(src) } {} },
            Script::Embedded(src) => html! { script { (PreEscaped(src)) } },
        }
    }
}
impl<'a> Script<'a> {
    pub fn wasm_bindgen_loader(name: &str) -> (String, String) {
        let glue = format!("wasm-bindgen-glue/{}.js", name);
        let load = format!("\
            document.addEventListener(\
                \"DOMContentLoaded\",\
                function(){{\
                    var mod = wasm_bindgen(\"/public/wasm/{}_bg.wasm\");\
                    if (mod.load_listeners) {{\
                        var listeners = mod.load_listeners();\
                    }}\
                }}\
            );\
        ", name);
        (glue, load)
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

