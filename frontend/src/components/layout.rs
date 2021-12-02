use yew::prelude::{html, Children, Component, ComponentLink, Properties, ShouldRender};

use crate::components::common::title::Title;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

pub struct Layout {
    props: Props,
    _link: ComponentLink<Self>,
}

impl Component for Layout {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: yew::ComponentLink<Self>) -> Self {
        Layout { props, _link }
    }

    fn view(&self) -> yew::Html {
        html! {
            <main>
                <Title>{"TGuard"}</Title>
                { for self.props.children.iter() }
            </main>
        }
    }

    fn update(&mut self, _: <Self as yew::Component>::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }
}
