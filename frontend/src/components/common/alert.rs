use yew::prelude::{html, Children, Component, ComponentLink, Properties, ShouldRender};

#[derive(Clone, PartialEq)]
pub enum AlertKind {
    Error,
    Success,
    Empty,
}

pub struct Alert {
    props: Props,
    _link: ComponentLink<Self>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub kind: AlertKind,
    #[prop_or_default]
    pub children: Children,
}

impl Component for Alert {
    type Properties = Props;
    type Message = ();

    fn create(props: Self::Properties, _link: yew::ComponentLink<Self>) -> Self {
        Alert { props, _link }
    }

    fn view(&self) -> yew::Html {
        html! {
            <div class=self.classes()>
            { for self.props.children.iter() }
            </div>
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

impl Alert {
    fn classes(&self) -> String {
        let base = "alert".to_owned();
        match self.props.kind {
            AlertKind::Error => base + " error",
            AlertKind::Success => base + " success",
            _ => base + " empty",
        }
    }
}
