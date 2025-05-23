mod world;

mod post_list;
mod post;

mod editor;

use wasm_bindgen::prelude::*;

use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Routable)]
enum Route {
    #[at("/blog")]
    PostList,
    #[at("/blog/post/:slug")]
    Post { slug: String },
    #[at("/blog/editor/:slug")]
    EditExisting { slug: String },
    #[at("/blog/editor")]
    EditNew,
}

fn switch(route: Route) -> Html {
    match route {
        Route::PostList => html! {<post_list::PostList/>},
        Route::Post { slug } => html! {<post::Post slug={slug} />},
        Route::EditNew => html! {<editor::Editor />},
        Route::EditExisting { slug } => html! {<editor::Editor slug={slug} />},
    }
}

#[function_component(Main)]
fn app() -> Html {
    html! {
        <world::World>
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </world::World>
    }
}

#[wasm_bindgen(start)]
pub fn init() -> Result<(), JsValue> {
    yew::Renderer::<Main>::new().render();
    Ok(())
}
