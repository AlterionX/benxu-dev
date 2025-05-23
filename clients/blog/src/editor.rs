use yew::prelude::*;

#[derive(Debug, PartialEq)]
#[derive(Properties)]
pub struct Props {
    #[prop_or_default]
    pub slug: Option<String>,
}

#[function_component]
pub fn Editor(props: &Props) -> Html {
    html! {
        <>{"under construction: "}{props.slug.as_ref().map(String::as_ref).unwrap_or("")}</>
    }
}

