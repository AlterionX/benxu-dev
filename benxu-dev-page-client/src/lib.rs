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
            (my_work())
            (my_interests())
            (my_passion())
            (my_reading_time())
            (my_gaming_time())
        }
        .slide-markers {
            (slide_markers(7))
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
        p {"\
            But mostly, I just enjoy watching pretty colors scroll really " span.italic.bold { "really" } " fast down \
            my terminal screen while I run my programs and blabber endlessly about my interests.\
        "}
    }, Some("intro active-slide"))
}
fn my_interests() -> Markup {
    slide("Everything is fascinating", html! {
        p {"\
            C, C++, and Rust are my favorite languages. I have worked in both OpenGl and Vulkan. \
            I've dabbled with Unity, Godot, and Unreal; Amethyst sounds interesting as well. \
            However, I also enjoy gaming and reading in my spare time, as well as learning even more about \
            tech and interesting projects such as WASM, xi, TypeScript, Fuschia, and AR glasses.\
        "}
        p {"\
            As a note, just for fun, this entire website is built with Rust + Rust compiled to WASM (WASM. I never get tired of saying that word. Anyways...) \
            + around 5 lines of actual JS to fetch/load the WASM module. I don't know how many browsers it runs on, but it was definitely fun. \
            I'm about to add a blog section that will also be written in Rust.\
        "}
    }, None)
}
fn my_story() -> Markup {
    slide("Improve a little, day by day", html! {
        p {"\
            There was a day in 10th grade, when one of my friends introduced me to Java. I was \
            enamored the moment I touched the keyboard. The actual program was cute little \
            thing, reading and adding two numbers.\
        "}
        p {"\
            It blew my mind.
        "}
        p {"\
            Now that I think about it, it fits; I had been enchanted by the power of words so I wanted to be a novelist,\
            but then I found something even more powerful.\
        "}
        p {"\
            Either way, I had decided then and there that I knew that I wanted to program for \
            a living. And now I'm here, seeking to live a life programming and architecting solutions.\
        "}
    }, None)
}
fn my_work() -> Markup {
    slide("Learning to code", html! {
        p {"\
            I've picked up a lot of different skills since that day. I developed a custom Wordpress theme and wrote \
            a chatlog for my English class. In my last year of high school, I learned about automata theory.\
        "}
        p {"\
            When I came to college, I wrote part of an OS in no-std C++ and a Python frontend for connecting to a server and testing. \
            I fell in love with writing tools and performance-critical programming.\
        "}
        p {"\
            I've written (with a friend) a ray tracer, a fluid simulation, and a shattering simulation. I am in the \
            middle of writing a simulation in Rust that combines a majority of these concepts. I ended up devoting \
            enough time to it that I will make it my thesis project.\
        "}
    }, None)
}
fn my_passion() -> Markup {
    slide("Programming and Writing", html! {
        p {"\
            I focus on systems development, rendering, and physical simulation. I think I've already said \
            enough about that. But I also have a string interest in game development and story writing.\
        "}
        p {"\
            In fact, by virtue of NaNoWriMo, I almost have the first version of my novel finished! I'm \
            about three fourths of the way and it's really heating up.\
        "}
        p{"\
            I am also working on a branching story script that I hope to turn into a simple game. Assets \
            are really hard to manage.\
        "}
    }, None)
}
fn my_reading_time() -> Markup {
    slide("Breaktime: Reading!", html! {
        p {"\
            Speaking of wriing, I love to read as well. " a href="https://brandonsanderson.com/" { "Brandon Sanderson" } "'s my favorite author, \
            but " a href="https://www.patrickrothfuss.com/content/index.asp" { "Patrick Rothfuss" } " is the most \
            inspirational one—still waiting for " span.underline { "The Doors of Stone" } ". (It's alright. We've only waited for a decade-ish.)\
        "}
        p {"\
            Rothfuss is the one who inspired me to write, so I aim to take just as long as him to finish my stories. \
            But, actually, the subtelty and detailed foreshadowing in his work is mind boggling. As I attempt to do \
            the same, I realize this all the more.\
        "}
    }, None)
}
fn my_gaming_time() -> Markup {
    slide("Breaktime: Gaming!", html! {
        p {"\
            Games are the other half of my free time. Shooters are good as stress relief but my favorites are RPGs. \
            My favorites, however, is The Last of Us. It is a work of art. Nier: Automata comes in at a close second; it's only lower \
            due to the PC port -- as a developer, its poor performance was obvious.\
        "}
        p{"\
            It feels like it's been a while since my last good RPG. That, and my faith in CD Projekt Red, \
            makes me optimistic about Cyberpunk 2077.\
        "}
        p {"\
            In fact, I find myself more engrossed in Terraria and Stellaris instead of RPGs since they leave a lot of room to \
            establish a character and role play despite not being an RPG. Dungeons and Dragons (DnD) is pretty fun as well.\
        "}
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
