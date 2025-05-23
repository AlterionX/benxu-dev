use axum::response::Html;
use maud::html;
use spb::{data::{Css, Favicon, Logo, Menu, MenuItem, PageMetaData, Script}, partials::basic_page};

pub async fn page() -> Html<String> {
    let (glue, load) = Script::wasm_bindgen_loader("public/js", "public/wasm", "blog");
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

    basic_page(
        html! {
            .blog {
                "Under construction, please be patient!"
            }
        },
        Some(&meta),
    ).into_string().into()
}
