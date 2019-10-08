#![feature(proc_macro_hygiene, decl_macro)]

//! Provides a functions to generate static webpages for benxu.dev at compile time.

pub mod data;
pub mod partials;

pub fn logo() -> Option<data::Logo<'static>> {
    Some(data::Logo {
        src: "/public/img/branding.svg",
        href: Some("/"),
    })
}

/// Functions generating my home page.
pub mod home {
    use crate::{data, partials};
    use maud::{html, Markup, Render};

    /// Create a basic menu.
    pub fn menu() -> Option<data::Menu<'static>> {
        Some(data::Menu(&[
            data::MenuItem {
                text: "Blog",
                link: Some("/blog"),
                children: None,
            },
        ]))
    }

    /// Returns a list of links as [`Markup`].
    fn link_group<'a>() -> Markup {
        let links = vec![
            data::LogoLink {
                url: "https://github.com/AlterionX/",
                logo: "public/img/icon/github.png",
                alt_text: "Github",
                text: "AlterionX",
            },
            data::LogoLink {
                url: "mailto:ben.xu.cs@gmail.com",
                logo: "public/img/icon/email.svg",
                alt_text: "Email",
                text: "ben.xu.cs@gmail.com",
            },
        ];
        html! {
            .link-group {
                @for link in links.iter() {
                    (link)
                }
            }
        }
    }

    /// Returns the slide show as [`Markup`].
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
            .slide-attachments {
                img#slide-prev.slide-attachment src="public/img/left-simple-arrow.svg";
                .slide-markers.slide-attachment {
                    (slide_markers(7))
                }
                img#slide-next.slide-attachment src="public/img/right-simple-arrow.svg";
            }
        }
    }
    /// Returns a slide as [`Markup`].
    fn slide<T: Render, U: Render>(title: T, text: U, cls: Option<&str>) -> Markup {
        html! {
            div class={ "slide" @if let Some(cls) = cls { " " (cls) } } {
                h2.slide-heading { (title) }
                .slide-text { (text) }
            }
        }
    }
    /// Returns the slide_markers as [`Markup`].
    fn slide_markers(slide_cnt: u8) -> Markup {
        html! {
            @for i in 0..slide_cnt {
                (slide_marker(i))
            }
        }
    }
    /// Returns the slide_marker as [`Markup`].
    fn slide_marker(idx: u8) -> Markup {
        html! {
            div id = { "slide-marker-"(idx) } class={"slide-marker" @if idx == 0 { (" active-slide-marker") }}  {}
        }
    }
    /// Returns the first slide as [`Markup`].
    fn my_intro() -> Markup {
        slide(
            "Nice to meet you",
            html! {
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
            },
            Some("intro active-slide"),
        )
    }
    /// Returns the second slide as [`Markup`].
    fn my_interests() -> Markup {
        slide(
            "Everything is fascinating",
            html! {
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
            },
            None,
        )
    }
    /// Returns the third slide as [`Markup`].
    fn my_story() -> Markup {
        slide(
            "Improve a little, day by day",
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
    /// Returns the fourth slide as [`Markup`].
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
                    I've written (with a friend) a ray tracer, a fluid simulation, and a shattering simulation. I am in the \
                    middle of writing a simulation in Rust that combines a majority of these concepts. I ended up devoting \
                    enough time to it that I will make it my thesis project.\
                "}
            },
            None,
        )
    }
    /// Returns the fifth slide as [`Markup`].
    fn my_passion() -> Markup {
        slide(
            "Programming and Writing",
            html! {
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
            },
            None,
        )
    }
    /// Returns the sixth slide as [`Markup`].
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
    /// Returns the seventh slide as [`Markup`].
    fn my_gaming_time() -> Markup {
        slide(
            "Breaktime: Gaming!",
            html! {
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
            },
            None,
        )
    }

    /// Returns a list of [`Css`](crate::data::Css) scripts that go in my home page.
    fn css_scripts<'a>() -> [data::Css<'a>; 4] {
        [
            data::Css::Critical { src: "reset" },
            data::Css::Critical { src: "typography" },
            data::Css::Critical { src: "main" },
            data::Css::Critical { src: "index" },
        ]
    }

    /// Returns the [`Markup`] version of my home page.
    pub fn index() -> Markup {
        let (glue, load) = data::Script::wasm_bindgen_loader("wasm_script");
        let js_scripts = [
            data::Script::External(glue.as_str()),
            data::Script::Embedded(load.as_str()),
        ];
        let css_scripts = css_scripts();
        let menu = menu();
        let logo = crate::logo();
        let meta = data::MetaData::builder()
            .scripts(&js_scripts[..])
            .css(&css_scripts[..])
            .menu(menu.as_ref())
            .logo(logo.as_ref())
            .build();
        partials::basic_page(
            html! {
                div.profile {
                    h1.tagline { "Ben Xu | Developer" }
                    img.propic src="public/img/propic.jpg" alt="Profile Picture";
                    (link_group())
                    (slides())
                }
            },
            Some(&meta),
        )
    }
}
/// Functions generating a complete version of my resume. Not yet implemented.
pub mod resume {}
/// Functions generating a summary of my online presence. Not yet implemented.
pub mod links {}
/// Functions generating a complete list of my projects. Not yet implemented.
pub mod projects {}
/// Functions serving the initial blog page, before it gets taken over by
/// [`blog_client`](blog_client).
pub mod blog {
    use crate::{data, partials};
    use maud::{Markup, html};

    /// Create a basic menu.
    pub fn menu() -> Option<data::Menu<'static>> {
        Some(data::Menu(&[
            data::MenuItem {
                text: "Blog",
                link: Some("/blog"),
                children: None,
            },
            data::MenuItem {
                text: "Login",
                link: Some("/blog/login"),
                children: None,
            },
        ]))
    }
    /// Create a basic menu for a special user.
    pub fn logged_in_menu() -> Option<data::Menu<'static>> {
        Some(data::Menu(&[
            data::MenuItem {
                text: "Blog",
                link: Some("/blog"),
                children: None,
            },
            data::MenuItem {
                text: "Profile",
                link: Some("/blog/profile"),
                children: None,
            },
        ]))
    }

    /// Returns a list of [`Css`](crate::data::Css) scripts that go in my blog page.
    fn css_scripts<'a>() -> [data::Css<'a>; 4] {
        [
            data::Css::Critical { src: "reset" },
            data::Css::Critical { src: "typography" },
            data::Css::Critical { src: "main" },
            data::Css::NonCritical { src: "blog" },
        ]
    }

    /// Returns a basic page, as everything will be managed by `blog_client`.
    pub fn index(is_logged_in: bool) -> Markup {
        let (glue, load) = data::Script::wasm_bindgen_loader("blog_client");
        let js_scripts = [
            data::Script::External(glue.as_str()),
            data::Script::Embedded(load.as_str()),
        ];
        let css_scripts = css_scripts();
        let menu = if is_logged_in {
            logged_in_menu()
        } else {
            menu()
        };
        let logo = crate::logo();
        let meta = data::MetaData::builder()
            .scripts(&js_scripts[..])
            .css(&css_scripts[..])
            .menu(menu.as_ref())
            .logo(logo.as_ref())
            .build();
        partials::basic_page(html!{ "Loadinggggggggggggg" }, Some(&meta))
    }
}
