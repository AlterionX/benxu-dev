#![feature(proc_macro_hygiene, decl_macro)]

mod data;
mod partials;

use maud::{Markup, html, Render, PreEscaped};

fn slide<T: Render, U: Render>(title: T, text: U) -> Markup {
    html! {
        .slide {
            h2.slide-title { (title) }
            .slide-text { (text) }
        }
    }
}
fn my_intro() -> Markup {
    slide(
        "Nice to meet you,",
        html! {
            p { "I'm Ben, a developer. but I've said that already, haven't I? Well, in that case, I am:" }

            ul {
                li {
                    "an architect; I dream of clean code structures and consistency."
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
                li {
                    "a reader; I love to read. But that can get long, so let's save the details for later."
                }
                li {
                    "a programmer; I love to code. But that can also get long, so let's save the details for later, too."
                }
            }

            p {
                "But mostly, I just enjoy watching pretty colors scroll really " span { "really" } " fast on \
                my terminal screen when I'm running my programs."
            }
        },
    )
}
fn my_interests() -> Markup {
    slide("", html! {
        p {
            "C, C++, Rust are my favorite languages."
            "I have work dealt with OpenGl and Vulkan."
            "I've dabbled with Unity, Godot, and Unreal."
            ""
        }
    })
}
fn my_story() -> Markup {
    slide("", html! {
        p {
            "My life is built around being a logical thinker and emotional actor."
        }
    })
}
fn my_focus() -> Markup {
    slide("", html! {
        p {
            "I focus on systems development, rendering, and physical simulation."
            "I have a heavy interest in game development and story writing."
        }
    })
}
fn my_favorite_media() -> Markup {
    slide("", html! {
        p {
            a href="https://brandonsanderson.com/" { "Brandon Sanderson" } "'s my favorite author, \
            but " a href="https://www.patrickrothfuss.com/content/index.asp" { "Patrick Rothfuss" } " is the most \
            inspirational one—still waiting for " span.underline { "The Doors of Stone" } ". (It's alright. We've just \
            been waiting for about a decade.) Rothfuss is the one who inspired me to write, so I aim to take just as long \
            as him to finish my stories."
        }
    })
}

pub fn index() -> Markup {
    let mut meta = data::MetaData::default();
    meta.css = &[
        data::Css { src: "reset" },
        data::Css { src: "typography" },
        data::Css { src: "main" },
        data::Css { src: "index" },
    ];
    let links = vec![data::LogoLink {
        url: "https://github.com/AlterionX/",
        logo: "public/img/icon/github.png",
        alt_text: "Github",
    }];
    partials::basic_page(html! {
        div.profile {
            p.blurb { "I'm a." }
            h1.tagline { "Developer. Reader. Gamer. Writer." }
            img.propic src="public/img/propic.jpg" alt="Profile Picture";
            .slides {
                (my_intro())
                (my_story())
                (my_interests())
                (my_focus())
                (my_favorite_media())
            }
            .link-group {
                @for link in links.iter() {
                    (link)
                }
            }
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
