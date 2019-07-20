#![feature(proc_macro_hygiene, decl_macro)]

mod data;
mod partials;

use maud::{Markup, html, Render};

fn link_group<'a>() -> Markup {
    let links = vec![data::LogoLink {
        url: "https://github.com/AlterionX/",
        logo: "public/img/icon/github.png",
        alt_text: "Github",
        text: "AlterionX",
    }, data::LogoLink {
        url: "mailto:ben.xu.cs@gmail.com",
        logo: "public/img/icon/email.svg",
        alt_text: "Email",
        text: "ben.xu.cs@gmail.com",
    }];
    html! {
        .link-group {
            @for link in links.iter() {
                (link)
            }
        }
    }
}

fn slides() -> Markup {
    html! {
        .slides {
            (my_intro())
            (my_story())
            (my_interests())
            (my_passion())
            (my_free_time())
        }
        .slide-markers {
            (slide_markers(5))
        }
    }
}
fn slide<T: Render, U: Render>(title: T, text: U, cls: Option<&str>) -> Markup {
    html! {
        div class={ "slide" @if let Some(cls) = cls { " " (cls) } } {
            h2.slide-heading { (title) }
            .slide-text { (text) }
        }
    }
}
fn slide_markers(slide_cnt: u8) -> Markup {
    html! {
        @for i in 0..slide_cnt {
            (slide_marker(i))
        }
    }
}
fn slide_marker(idx: u8) -> Markup {
    html! {
        div class={"slide-marker" @if idx == 0 { (" active-slide-marker") }}  {}
    }
}
fn my_intro() -> Markup {
    slide("Nice to meet you", html! {
        p { "My name is Ben. I am a developer, but I am also:" }
        ul {
            li {
                "a reader; I love to read. But that can get long, so let's save the details for later."
            }
            li {
                "a writer; " a href="https://www.nanowrimo.org/participants/alterionx/novels" { "NaNoWriMo" } " \
                (a.k.a. November) is simultaneously my favorite and most hated month of the year."
            }
            li {
                "a gamer; can't wait for " a href="https://www.cyberpunk.net/us/en/" { "Cyberpunk 2077." }
            }
            li {
                "a linguist: I technically know Chinese, and am studying Japanese."
            }
        }
        p {
            "But mostly, I just enjoy watching pretty colors scroll really " span.italic.bold { "really" } " fast down \
            my terminal screen while I run my programs."
        }
    }, Some("intro active-slide"))
}
fn my_interests() -> Markup {
    slide("Everything is fascinating", html! {
        p {
            "C, C++, Rust are my favorite languages."
            "I have work dealt with OpenGl and Vulkan."
            "I've dabbled with Unity, Godot, and Unreal."
            "I would like to get involved with Amethyst as well."
            "At some point, I would like to help out with the emulator development scene as well."
        }
    }, None)
}
fn my_story() -> Markup {
    slide("Improve a little, every day", html! {
        p {
            "My life is built around being a logical thinker and emotional actor."
        }
    }, None)
}
fn my_passion() -> Markup {
    slide("I love my work", html! {
        p {
            "I focus on systems development, rendering, and physical simulation."
            "I have a heavy interest in game development and story writing."
        }
    }, None)
}
fn my_free_time() -> Markup {
    slide("Taking breaks", html! {
        p {
            "Reading is awesome. " a href="https://brandonsanderson.com/" { "Brandon Sanderson" } "'s my favorite author, \
            but " a href="https://www.patrickrothfuss.com/content/index.asp" { "Patrick Rothfuss" } " is the most \
            inspirational one—still waiting for " span.underline { "The Doors of Stone" } ". (It's alright. We've just \
            been waiting for about a decade.) Rothfuss is the one who inspired me to write, so I aim to take just as long \
            as him to finish my stories."
        }
    }, None)
}

fn css_scripts<'a>() -> [data::Css<'a>; 4] {
    [
        data::Css { src: "reset" },
        data::Css { src: "typography" },
        data::Css { src: "main" },
        data::Css { src: "index" },
    ]
}

pub fn index() -> Markup {
    let (glue, load) = data::Script::wasm_bindgen_loader("benxu_dev_page_home_script");
    let js_scripts = [
        data::Script::External(glue.as_str()),
        data::Script::Embedded(load.as_str()),
    ];
    let css_scripts = css_scripts();
    let meta = data::MetaData::builder()
        .scripts(&js_scripts[..])
        .css(&css_scripts[..])
        .build();
    partials::basic_page(html! {
        div.profile {
            h1.tagline { "Ben Xu | Developer" }
            img.propic src="public/img/propic.jpg" alt="Profile Picture";
            (link_group())
            (slides())
        }
    }, Some(&meta))
}

pub mod resume {
}
pub mod links {
}
pub mod contacts {
}
pub mod projects {
}
