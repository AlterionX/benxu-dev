use axum::{http::StatusCode, response::Html};
use maud::html;
use spb::{data::{PageMetaData, Css, Menu, MenuItem, Logo, Favicon}, partials::basic_page};

pub async fn page() -> (StatusCode, Html<String>) {
    let meta = PageMetaData {
        scripts: &[],
        css: &[
            Css::Critical { src: "public/css/reset.css" },
            Css::Critical { src: "public/css/typography.css" },
            Css::Critical { src: "public/css/main.css" },
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
            link: "public/svg/favicon.svg",
            media_type: None,
            sizes: None,
        }],
        ..PageMetaData::default()
    };

    let text = html! { p { "You're probably looking to head back to " a href="/" { "the home page" } "..." } };

    (StatusCode::NOT_FOUND, Html(basic_page(text, Some(&meta)).into_string()))
}
