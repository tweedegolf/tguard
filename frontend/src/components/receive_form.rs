use wasm_bindgen_futures::spawn_local;
use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use crate::types::{File, ReceivedData};
use crate::{
    actions::download_and_decrypt,
    components::common::{
        alert::{Alert, AlertKind},
        loader::Loader,
    },
};

#[derive(Properties, Clone, PartialEq, Debug)]
pub struct Props {
    pub id: String,
}

pub enum ReceiveFormMsg {
    Initial,
    Error,
    Update(ReceivedData),
}

#[derive(Debug)]
pub struct ReceiveForm {
    props: Props,
    link: ComponentLink<Self>,
    data: ReceivedData,
    error: bool,
}

impl Component for ReceiveForm {
    type Properties = Props;
    type Message = ReceiveFormMsg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        link.send_message(Self::Message::Initial);
        Self {
            props,
            link,
            data: Default::default(),
            error: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Self::Message::Initial => {
                let link = self.link.clone();
                let id = self.props.id.clone();

                spawn_local(async move {
                    if download_and_decrypt(&link, &id).await.is_none() {
                        link.send_message(Self::Message::Error);
                    }
                });
            }
            Self::Message::Error => {
                self.error = true;
            }
            Self::Message::Update(data) => {
                self.data = data;
            }
        };

        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
          <>
            {
                if self.error {
                    html!{
                        <Alert kind=AlertKind::Error>
                            {"An error occured, please "}
                            <a href="." target="_blank">{"try again."}</a>
                        </Alert>
                    }
                } else if !self.data.message.is_empty() {
                    html!{
                        <Alert kind=AlertKind::Success>{"Message decrypted successfully."}</Alert>
                    }
                } else {
                    html!{
                        <Alert kind=AlertKind::Empty>
                            {"Decrypt a secret message using "}
                            <a href="https://irma.app" target="_blank">{"IRMA"}</a>
                        </Alert>
                    }
                }
            }
            {
                if self.data.from.is_empty() {
                    html!{
                        <Loader />
                    }
                } else {
                    html!{
                        <>
                            <dl>
                                <dt>{"Sender:"}</dt>
                                <dd>{&self.data.from}</dd>
                                <dt>{"To:"}</dt>
                                <dd>{&self.data.to}</dd>
                                <dt>{"Subject:"}</dt>
                                <dd>{&self.data.subject}</dd>
                                { if self.data.signed {
                                    html!{
                                      <dt>{"Signed Message:"}</dt>
                                    }
                                } else {
                                    html!{
                                        <dt>{"Message:"}</dt>
                                    }
                                }}
                                <dd>
                                    <pre>
                                        {&self.data.message}
                                    </pre>
                                </dd>
                            </dl>
                            { if !self.data.attachments.is_empty() {
                                html!{
                                    <label>{"Attachments:"}</label>
                                }
                            } else {
                                html!{}
                            }}
                            <table class="files">
                                { for self.data.attachments.iter().map(Self::view_file) }
                            </table>
                        </>
                    }
                }
            }
          </>
        }
    }
}

impl ReceiveForm {
    fn view_file(data: &File) -> Html {
        let content = base64::encode(&data.content);

        html! {
            <tr>
                <td>
                    <p class="filename">
                        {data.filename.clone()}
                    </p>
                </td>
                <td class="actions">
                    <a
                        class="button outlined"
                        download={data.filename.clone()}
                        href={format!("data:{};base64,{}", data.mimetype, content)}
                        target="_blank"
                    >
                            {"download"}
                    </a>
                </td>
            </tr>
        }
    }
}
