use yew::prelude::{html, Component, ShouldRender};

pub struct Loader;

impl Component for Loader {
    type Properties = ();
    type Message = ();

    fn create(_props: Self::Properties, _link: yew::ComponentLink<Self>) -> Self {
        Self
    }

    fn view(&self) -> yew::Html {
        html! {
            <h2>
                {"Loading..."}
            </h2>
        }
    }

    fn update(&mut self, _: <Self as yew::Component>::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
}
