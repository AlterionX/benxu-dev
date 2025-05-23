use yew::prelude::*;

#[derive(Debug, PartialEq)]
#[derive(Properties)]
pub struct Props {
    pub children: Html,
}

#[function_component]
pub fn World(props: &Props) -> Html {
    html! {
        <>{props.children.clone()}</>
    }
}

