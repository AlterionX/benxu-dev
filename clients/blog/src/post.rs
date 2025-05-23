use yew::prelude::*;

#[derive(Debug, PartialEq)]
#[derive(Properties)]
pub struct Props {
    pub slug: String,
}

#[function_component]
pub fn Post(props: &Props) -> Html {
    html! {
        <>{"under construction: "}{&props.slug}</>
    }
}
