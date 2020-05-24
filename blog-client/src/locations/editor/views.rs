use seed::prelude::*;

use crate::{
    locations::editor::{M, S},
    model::Store as GlobalS,
};

pub fn render(s: &S, _gs: &GlobalS) -> Vec<Node<M>> {
    vec![
        heading(),
        editor(s).unwrap_or_else(crate::shared::views::loading),
    ]
}

pub fn heading() -> Node<M> {
    h1![attrs! { At::Class => "as-h3" }, "Editing"]
}

fn get_title_slug_body(s: &S) -> Option<(&str, Option<&str>, &str)> {
    let (t, slug, b) = match s {
        S::New(post) => (&post.title, post.slug.as_ref(), &post.body),
        S::Old(post, changed) => (
            changed.title.as_ref().unwrap_or(&post.title),
            post.slug.as_ref(),
            changed.body.as_ref().unwrap_or(&post.body),
        ),
        _ => return None,
    };
    let slug = slug.map(String::as_str);
    Some((t, slug, b))
}
fn title_field(title: &str) -> Node<M> {
    div![
        attrs! { At::Class => "editor-title" },
        input![
            {
                let mut attrs = attrs! {
                    At::Placeholder => "Title";
                    At::Type => "text";
                    At::Name => "title",
                    At::Value => title,
                };
                attrs.add_multiple(At::Class, &["single-line-text-entry", "as-h1"]);
                attrs
            },
            input_ev(Ev::Input, M::Title),
        ],
    ]
}
fn slug_field(slug: &str, hint: &str) -> Node<M> {
    div![
        attrs! { At::Class => "editor-slug" },
        label![
            {
                let mut attrs = attrs! {
                    At::For => "slug",
                };
                attrs.add_multiple(At::Class, &["same-line-label", "as-pre"]);
                attrs
            },
            "/blog/posts/",
        ],
        input![
            {
                let mut attrs = attrs! {
                    At::Placeholder => hint;
                    At::Type => "text";
                    At::Name => "slug",
                    At::Value => slug,
                };
                attrs.add_multiple(At::Class, &["single-line-text-entry", "as-pre"]);
                attrs
            },
            input_ev(Ev::Input, M::Slug),
        ],
    ]
}
fn body_field(body: &str) -> Node<M> {
    div![
        attrs! {
            At::Class => "editor-body",
        },
        textarea![
            {
                let mut attrs = attrs! {
                    At::Placeholder => "Write your post here!";
                    At::Type => "text";
                    At::Name => "body",
                };
                attrs.add_multiple(At::Class, &["multi-line-text-entry"]);
                attrs
            },
            body,
            input_ev(Ev::Input, M::Body),
        ],
    ]
}
fn action_buttons(s: &S) -> Node<M> {
    div![
        attrs! {
            At::Class => "editor-actions",
        },
        input![
            attrs! {
                At::Class => "inline-button",
                At::Type => "submit",
                At::Value => "Save",
            },
            ev(Ev::Click, |e| {
                e.prevent_default();
                M::Save
            }),
        ],
        if s.is_publishable() {
            input![
                attrs! {
                    At::Class => "inline-button",
                    At::Type => "submit",
                    At::Value => "Publish",
                },
                ev(Ev::Click, |e| {
                    e.prevent_default();
                    M::Publish
                }),
            ]
        } else {
            empty![]
        },
    ]
}
pub fn editor(s: &S) -> Option<Node<M>> {
    let (title, slug, body) = get_title_slug_body(s)?;
    let slug_hint_mem = slug
        .map(|_| None)
        .unwrap_or_else(|| s.old_ref().map(|p| p.id.to_hyphenated_ref().to_string()));
    let slug_hint = slug_hint_mem
        .as_ref()
        .map(String::as_str)
        .or(slug)
        .unwrap_or("");
    Some(div![
        attrs! { At::Class => "editor" },
        title_field(title),
        slug_field(slug.unwrap_or(""), slug_hint),
        body_field(body),
        action_buttons(s),
    ])
}
