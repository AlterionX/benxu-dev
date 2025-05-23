//! Generates the landing page.

use axum::response::Html;
use maud::{html, Markup, Render};

use spb::{data::{Script, Css, MenuItem, Menu, Logo, LogoLink, PageMetaData, Favicon}, partials::basic_page};

pub async fn page() -> Html<String> {
    trc::info!("Processing landing page request.");
    let (glue, load) = Script::wasm_bindgen_loader("public/js", "public/wasm", "slideshow");
    let meta = PageMetaData {
        scripts: &[
            Script::External(glue.as_str()),
            Script::Embedded(load.as_str()),
        ],
        css: &[
            Css::Critical { src: "public/css/reset.css" },
            Css::Critical { src: "public/css/typography.css" },
            Css::Critical { src: "public/css/main.css" },
            Css::Critical { src: "public/css/index.css" },
        ],
        menu: Some(&Menu(&[MenuItem {
            text: "Blog",
            link: Some("/blog"),
            children: None,
        }])),
        logo: Some(&Logo {
            src: "public/svg/branding.svg",
            href: Some("/"),
        }),
        favicons: &[Favicon {
            link: "favicon.svg",
            media_type: None,
            sizes: None,
        }],
        ..PageMetaData::default()
    };


    let links = Links([
        LogoLink {
            url: "https://github.com/AlterionX/",
            logo: "public/png/github.png",
            alt_text: "Github",
            text: "AlterionX",
        },
        LogoLink {
            url: "mailto:ben.xu.cs@gmail.com",
            logo: "public/svg/email.svg",
            alt_text: "Email",
            text: "ben.xu.cs@gmail.com",
        },
        LogoLink {
            url: "/resume",
            logo: "public/svg/resume.svg",
            alt_text: "Resume",
            text: "Resume",
        },
    ]);

    let slides = Slides([
        my_intro(),
        my_story(),
        my_work(),
        my_interests(),
        my_passion(),
        my_reading_time(),
        my_gaming_time(),
    ]);

    basic_page(
        html! {
            div.profile {
                h1.tagline { "Ben Xu | Developer" }
                img.propic src="public/jpg/propic.jpg" alt="Profile Picture";
                (links)
                (slides)
            }
        },
        Some(&meta),
    ).into_string().into()
}

pub struct Links<'a, const N: usize>([LogoLink<'a>; N]);
impl<'a, const N: usize> Render for Links<'a, N> {
    fn render(&self) -> Markup {
        html! {
            .link-group {
                @for link in self.0.iter() {
                    (link)
                }
            }
        }
    }
}

pub struct Slides<const N: usize>([Markup; N]);
impl<const N: usize> Render for Slides<N> {
    fn render(&self) -> Markup {
        html! {
            .slides {
                @ for slide in self.0.iter() {
                    (slide)
                }
            }
            .slide-attachments {
                img #slide-prev .slide-attachment src="public/svg/left-simple-arrow.svg";
                .slide-markers .slide-attachment {
                    @for i in 0..self.0.len() {
                        div id = { "slide-marker-"(i) } class={"slide-marker" @if i == 0 { (" active-slide-marker") }}  {}
                    }
                }
                img #slide-next .slide-attachment src="public/svg/right-simple-arrow.svg";
            }
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

fn my_intro() -> Markup {
    slide(
        "Nice to meet you",
        html! {
            p { "My name is Ben. I am a coder, but I am also:" }
            ul {
                li {
                    "a reader; I love to read. But that can get long, so let's save the details for later."
                }
                li {
                    "a worldbuilder; I love thinking about sapience in weird scenarios."
                }
                li {
                    "a gamer; still waiting for " a href="https://robertsspaceindustries.com/" { "Star Citizen." }
                }
            }
            p {"\
                But mostly, I just enjoy watching pretty colors scroll really " span.italic.bold { "really" } " fast down \
                my terminal screen while I run my programs and blabber endlessly about my interests.\
            "}
        },
        Some("intro active-slide"),
    )
}
fn my_interests() -> Markup {
    slide(
        "Fascinations",
        html! {
            p {"\
                C, C++, and Rust are my favorite languages. I have worked in both OpenGl and Vulkan. \
                I've dabbled with Unity, Godot, and Unreal; Amethyst sounds interesting as well. \
                However, I also enjoy gaming and reading in my spare time, as well as learning even more about \
                tech and interesting projects such as WASM, xi, TypeScript, Fuschia, and AR glasses.\
            "}
            p {"\
                As a note, just for fun, this entire website is built with Rust + WASM \
                (Such a fun word. Anyways...). I don't know how many browsers it runs on, \
                but it was definitely fun.\
            "}
        },
        None,
    )
}
fn my_story() -> Markup {
    slide(
        "A bit about my past",
        html! {
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
        },
        None,
    )
}
fn my_work() -> Markup {
    slide(
        "Learning to code",
        html! {
            p {"\
                I've picked up a lot of different skills since that day. I developed a custom Wordpress theme and wrote \
                a chatlog for my English class. In my last year of high school, I learned about automata theory.\
            "}
            p {"\
                When I came to college, I wrote part of an OS in no-std C++ and a Python frontend for connecting to a server and testing. \
                I fell in love with writing tools and performance-critical programming.\
            "}
            p {"\
                I've written (with a friend) a ray tracer, a fluid simulation, and a shattering simulation. I am slowly \
                working on a simulation in Rust that combines a majority of these concepts.\
            "}
        },
        None,
    )
}
fn my_passion() -> Markup {
    slide(
        "Programming and writing",
        html! {
            p {"\
                I focus on systems development, rendering, and physical simulation. I think I've already said \
                enough about that. But I also have a string interest in game development and story writing.\
            "}
            p {"\
                In fact, by virtue of NaNoWriMo, I have the first version of my novel finished!\
            "}
        },
        None,
    )
}
fn my_reading_time() -> Markup {
    slide(
        "Breaktime: Reading!",
        html! {
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
        },
        None,
    )
}
fn my_gaming_time() -> Markup {
    slide(
        "Breaktime: Gaming!",
        html! {
            p {"\
                Games are the other half of my free time. Shooters are good as stress relief but my favorites are RPGs. \
                My favorite, however, is The Last of Us. It is a work of art. NieR: Automata comes in at a close second and
                Clair Obscur: Expedition 33 has soared into my shortlist.\
            "}
            p {"\
                The favorites I'd listed are RPGs, but I find myself more engrossed in Terraria and Stellaris than RPGs since they leave a lot of room to \
                establish a character and role play despite not being an RPG. Dungeons and Dragons (DnD) is pretty fun as well.\
            "}
            p {"\
                I also enjoy various space sims, but Star Citizen has captured my heart and I don't think I could ever play a different \
                space sim without thinking about Star Citizen.\
            "}
        },
        None,
    )
}
