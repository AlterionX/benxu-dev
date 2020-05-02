use seed::prelude::*;
use crate::shared::LoggedIn;

pub fn loading<M: Clone>() -> seed::virtual_dom::Node<M> {
    p!["Loading!"]
}

fn nav_menu(is_logged_in: LoggedIn) -> String {
    if is_logged_in == LoggedIn::LoggedIn {
        htmlgen::data::Menu(&[
            htmlgen::data::MenuItem {
                text: "Home",
                link: Some("/"),
                children: None,
            },
            htmlgen::data::MenuItem {
                text: "Blog",
                link: Some("/blog"),
                children: None,
            },
            htmlgen::data::MenuItem {
                text: "Create Post",
                link: Some("/blog/editor/new"),
                children: None,
            },
            htmlgen::data::MenuItem {
                text: "Logout",
                link: Some("/blog/logout"),
                children: None,
            },
        ])
    } else {
        htmlgen::data::Menu(&[
            htmlgen::data::MenuItem {
                text: "Home",
                link: Some("/"),
                children: None,
            },
            htmlgen::data::MenuItem {
                text: "Blog",
                link: Some("/blog"),
                children: None,
            },
        ])
    }
    .into_string()
}
pub fn replace_nav(is_logged_in: LoggedIn) {
    let html = nav_menu(is_logged_in);
    let menu_node = seed::body()
        .get_elements_by_tag_name("nav")
        .item(0)
        .expect("Cannot find menu element.");
    let header_node = menu_node.parent_element().unwrap();
    let mock_node = seed::document().create_element("div").unwrap();
    mock_node.set_inner_html(html.as_str());
    let replacement = mock_node.children().item(0).unwrap();
    header_node.replace_child(&replacement, &menu_node).unwrap();
}
