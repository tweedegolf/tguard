use yew::prelude::{html, Children, Component, ComponentLink, Properties, ShouldRender};

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

pub struct Title {
    props: Props,
    _link: ComponentLink<Self>,
}

impl Component for Title {
    type Properties = Props;
    type Message = ();

    fn create(props: Self::Properties, _link: yew::ComponentLink<Self>) -> Self {
        Title { props, _link }
    }

    fn view(&self) -> yew::Html {
        html! {
            <h1>
            { for self.props.children.iter() }
            </h1>
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
