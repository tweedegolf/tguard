use crate::components::layout::Layout;
use crate::components::receive_form::ReceiveForm;
use crate::components::send_form::SendForm;
use crate::components::upload::Upload;
use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew_router::prelude::{Router, Switch};

#[derive(Switch, Debug, Clone)]
pub enum AppRoute {
    #[to = "/upload"]
    Upload,
    #[to = "/download/{id}"]
    Decrypt(String),
    #[to = "/"]
    Encrypt,
}

#[derive(Debug)]
pub struct Index;

impl Component for Index {
    type Properties = ();
    type Message = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <Layout>
                <Router<AppRoute, ()>
                    render = Router::render(|switch: AppRoute| {
                        match switch {
                            AppRoute::Decrypt(id) => html!{<ReceiveForm id = id/>},
                            AppRoute::Encrypt => html!{<SendForm />},
                            AppRoute::Upload => html!{<Upload />},
                        }
                    })
                />
            </Layout>
        }
    }
}
