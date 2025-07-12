//! A collection of metadata used during site generation.

use chrono::{Datelike, Utc};
use maud::{html, Markup, PreEscaped, Render};
use std::fs;

/// Represents a logo.
pub struct LogoLink<'a> {
    /// The link the logo will resolve to when clicked.
    pub url: &'a str,
    /// The url of the logo picture.
    pub logo: &'a str,
    /// Alternative text if the logo cannot be loaded.
    pub alt_text: &'a str,
    /// Text accompanying the logo.
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

/// Data used during site generation for things like css, scripts, contact info and menus. Most are
/// for meta tags.
pub struct PageMetaData<'a> {
    /// Language of the website.
    pub lang: &'a str,
    /// Encoding of the website.
    pub charset: &'a str,
    /// Scripts to include in the website.
    pub scripts: &'a [Script<'a>],
    /// CSS to include in the website.
    pub css: &'a [Css<'a>],
    /// The title of the website.
    pub title: &'a str,
    /// The description of the website.
    pub description: &'a str,
    /// The copyright data of the website.
    pub copyright: Copyright<'a>,
    /// The favicon
    pub favicons: &'a [Favicon<'a>],
    /// The menu of the website.
    pub menu: Option<&'a Menu<'a>>,
    /// The points of contact for the owner of the website.
    pub contact: Option<&'a Contact<'a>>,
    /// The logo of the website.
    pub logo: Option<&'a Logo<'a>>,
    /// The theme color of the website. Affects mobile address name bars.
    pub theme_color: &'a str,
}
impl<'a> Default for PageMetaData<'a> {
    fn default() -> Self {
        Self {
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
            favicons: &[],
            menu: None,
            contact: None,
            logo: None,
            theme_color: "#00003f",
        }
    }
}

pub struct Favicon<'a> {
    pub link: &'a str,
    pub media_type: Option<&'a str>,
    pub sizes: Option<&'a str>
}
impl<'a> Render for Favicon<'a> {
    fn render(&self) -> Markup {
        match (self.media_type, self.sizes) {
            (Some(mt), Some(sz)) => {
                html! { link rel="icon" href={(self.link)} type={(mt)} sizes={(sz)} {} }
            },
            (None, Some(sz)) => {
                html! { link rel="icon" href={(self.link)} sizes={(sz)} {} }
            },
            (Some(mt), None) => {
                html! { link rel="icon" href={(self.link)} type={(mt)} {} }
            },
            (None, None) => {
                html! { link rel="icon" href={(self.link)} {} }
            },
        }
    }
}

/// Information regarding the logo. (This is very simple).
pub struct Logo<'a> {
    /// The url to the actual image.
    pub src: &'a str,
    pub href: Option<&'a str>,
}
impl<'a> Render for Logo<'a> {
    fn render(&self) -> Markup {
        html! {
            div.logo {
                @match self.href {
                    Some(link) => a.logo-wrapper href=(link) {
                        img.logo-img src=(self.src);
                    },
                    None => img.logo-img src=(self.src);,
                }
            }
        }
    }
}

/// Information regarding the `<script>` tags to include.
pub enum Script<'a> {
    /// Represents a script externally linked (in the `public/js` directory).
    External(&'a str),
    /// Represents a script copy and pasted into the website.
    Embedded(&'a str),
}
impl<'a> Render for Script<'a> {
    fn render(&self) -> Markup {
        match self {
            Script::External(src) => html! { script defer?[true] src={ (src) } {} },
            Script::Embedded(src) => html! { script { (PreEscaped(src)) } },
        }
    }
}
impl<'a> Script<'a> {
    /// A script for hooking in the WASM loading script
    pub fn wasm_bindgen_loader(js_path: &str, wasm_path: &str, name: &str) -> (String, String) {
        let glue = format!("/{js_path}/{name}.js");
        let load = format!(
            "\
             document.addEventListener(\
                \"DOMContentLoaded\",\
                function(){{\
                    var mod = wasm_bindgen(\"/{wasm_path}/{name}_bg.wasm\")\
                        .catch(function(e) {{\
                            console.log(\"Promise received from wasm load.\");\
                            console.log(e);\
                            e.catch(function(e) {{\
                                console.log(e);\
                            }});
                        }});\
                    if (mod.load_listeners) {{\
                        var listeners = mod.load_listeners();\
                    }}\
                }}\
             );\
            ",
        );
        (glue, load)
    }
}

/// Information regarding the `<style>` tags to include.
pub enum Css<'a> {
    /// Above the fold CSS. This get linked in from the resources directory, `/public`.
    Critical { src: &'a str },
    /// Under the fold CSS. This get linked in from the resources directory, `/public`.
    NonCritical { src: &'a str },
}
impl<'a> Render for Css<'a> {
    fn render(&self) -> Markup {
        match self {
            Css::NonCritical { src } => html! { link rel="stylesheet" href={
                (src)
            }{} },
            Css::Critical { src } => {
                let style = fs::read_to_string(src)
                    .unwrap_or_else(|e| panic!("{:?} is missing ({:?})", src, e));
                html! { style { (PreEscaped(style)) } }
            }
        }
    }
}

/// A email address.
pub struct Email<'a> {
    /// The username portion of the email.
    pub user: &'a str,
    /// The domain portion of the email.
    pub domain: &'a str,
}
impl<'a> Render for Email<'a> {
    fn render(&self) -> Markup {
        html! {
            (self.user)"@"(self.domain)
        }
    }
}

/// A phone number. This is an enum for globalization.
pub enum PhoneNumber<'a> {
    /// A phone number in the US.
    US {
        /// The area code.
        area_code: u16,
        /// The prefix (the three numbers after the area code).
        prefix: u16,
        /// The line number (the four numbers after the area code).
        line_number: u16,
        /// A link to the icon for this number. (Work, Mobile, etc.)
        icon: &'a str,
    },
}
impl<'a> Render for PhoneNumber<'a> {
    fn render(&self) -> Markup {
        match self {
            PhoneNumber::US {
                icon,
                area_code,
                prefix,
                line_number,
            } => html! {
                (icon)": ("(area_code)") "(prefix)"-"(line_number)
            },
        }
    }
}

/// A contact card. Comprised of emails and phone numbers.
pub struct Contact<'a> {
    /// Emails for this contact.
    pub email: &'a [Email<'a>],
    /// Phone numbers for this contact.
    pub phone: &'a [PhoneNumber<'a>],
}
impl<'a> Render for Contact<'a> {
    fn render(&self) -> Markup {
        html! {
            @for email in self.email {
                p.contact-email { "Email: " (email) }
            }
            @for phone in self.phone {
                p.contact-phone-number { "Phone: " (phone) }
            }
        }
    }
}

/// A struct representing names.
pub struct Name<'a> {
    /// First name.
    pub first: &'a str,
    /// Middle name.
    pub middle: Option<&'a str>,
    /// Last name.
    pub last: &'a str,
    /// A list of nicknames.
    pub nicknames: &'a [&'a str],
}
impl<'a> Render for Name<'a> {
    fn render(&self) -> Markup {
        html! {
            (self.first) " " @if let Some(middle) = self.middle {
                @if let Some(initial) = middle.chars().next() {
                    (initial) ". " 
                }
            } (self.last)
        }
    }
}

/// Copyright data.
pub struct Copyright<'a> {
    /// Person copyrighting the website.
    pub name: &'a Name<'a>,
    /// The copyright icon to be used.
    pub icon: &'a str,
    /// What rights to grant/refuse.
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

/// An entry in the menu.
pub struct MenuItem<'a> {
    /// Text to display.
    pub text: &'a str,
    /// Where the entry links to, if it links to one.
    pub link: Option<&'a str>,
    /// A child menu, if one exists.
    pub children: Option<&'a Menu<'a>>,
}
impl<'a> MenuItem<'a> {
    /// Render a link to [`Markup`] if present.
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
impl<'a> Render for MenuItem<'a> {
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

/// A newtype for a list of [`MenuItem`](crate::data::MenuItem)s.
pub struct Menu<'a>(pub &'a [MenuItem<'a>]);
impl<'a> Render for Menu<'a> {
    fn render(&self) -> Markup {
        html! {
            nav.menu {
                ul {
                    @for item in self.0.iter() {
                        (item)
                    }
                }
            }
        }
    }
}
impl<'a> Menu<'a> {
    pub fn into_string(self) -> String {
        self.render().into_string()
    }
}
