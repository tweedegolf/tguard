pub mod attribute_header_row;
pub mod attribute_input;
pub mod recipient_row;

use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::{
    html, ChangeData, Component, ComponentLink, FocusEvent, Html, InputData, MouseEvent,
    ShouldRender,
};
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};

use common::{AttributeIdentifier, AttributeValue};

use crate::actions::{encrypt_and_submit, SendError};
use crate::attributes::{
    attribute_label, attribute_type, remaining_attribute_options, AttributeType,
    EMAIL_ATTRIBUTE_IDENTIFIER,
};
use crate::components::{
    common::alert::{Alert, AlertKind},
    send_form::{attribute_header_row::AttributeHeaderRow, recipient_row::RecipientRow},
};
use crate::types::{FormData, Recipient};

type FileName = String;

#[derive(PartialEq)]
pub enum SendFormStatus {
    Initial,
    Error(SendError),
    Encrypting,
    Sent,
}

impl SendFormStatus {
    fn class_name(&self) -> String {
        match self {
            Self::Initial => "initial",
            Self::Encrypting => "encrypting",
            Self::Sent => "sent",
            Self::Error(_) => "error",
        }
        .into()
    }
}

pub enum SendFormMsg {
    Submit,
    UpdateStatus(SendFormStatus),
    AddTo,
    UpdateTo(usize, String),
    DeleteTo(usize),
    AddAttr(AttributeIdentifier),
    DeleteAttr(usize),
    UpdateAttrValue(usize, usize, String),
    UpdateFrom(String),
    UpdateSubject(String),
    UpdateMessage(String),
    AddFiles(Vec<File>),
    LoadedFile((FileName, FileData)),
    DeleteFile(usize),
}

pub struct SendForm {
    link: ComponentLink<Self>,
    status: SendFormStatus,
    form: FormData,
    attributes: Vec<AttributeIdentifier>,
    tasks: HashMap<FileName, ReaderTask>,
}

impl Component for SendForm {
    type Properties = ();
    type Message = SendFormMsg;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            status: SendFormStatus::Initial,
            form: Default::default(),
            attributes: vec![AttributeIdentifier(EMAIL_ATTRIBUTE_IDENTIFIER.to_owned()); 1],
            tasks: HashMap::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Self::Message::AddTo => {
                let mut attributes = self.form.to[0].attributes.clone();
                attributes.iter_mut().for_each(|attr| {
                    attr.value = String::new();
                });

                self.form.to.push(Recipient {
                    to: String::new(),
                    attributes,
                });
            }
            Self::Message::UpdateTo(index, to) => {
                self.form.to[index].to = to.clone();

                if let Some(email_index) = self.form.to[index]
                    .attributes
                    .iter()
                    .position(|attr| attr.identifier.0 == EMAIL_ATTRIBUTE_IDENTIFIER)
                {
                    self.form.to[index].attributes[email_index].value = to;
                }
            }
            Self::Message::DeleteTo(index) => {
                if self.form.to.len() > 1 {
                    self.form.to.remove(index);
                }
            }
            Self::Message::AddAttr(identifier) => {
                self.attributes.push(identifier.clone());
                self.form.to.iter_mut().for_each(|el| {
                    el.attributes.push(AttributeValue {
                        identifier: identifier.clone(),
                        value: if identifier.0 == EMAIL_ATTRIBUTE_IDENTIFIER {
                            el.to.clone()
                        } else if attribute_type(&identifier) == AttributeType::Boolean {
                            "Yes".to_owned()
                        } else {
                            String::new()
                        },
                    });
                });
            }
            Self::Message::UpdateAttrValue(to_index, attribute_index, attribute_value) => {
                self.form.to[to_index].attributes[attribute_index].value = attribute_value;
            }
            Self::Message::DeleteAttr(index) => {
                self.attributes.remove(index);
                self.form.to.iter_mut().for_each(|el| {
                    el.attributes.remove(index);
                });
            }
            Self::Message::UpdateFrom(from) => self.form.from = from,
            Self::Message::UpdateSubject(subject) => self.form.subject = subject,
            Self::Message::UpdateMessage(message) => self.form.message = message,
            Self::Message::UpdateStatus(status) => {
                if status == SendFormStatus::Initial {
                    self.form = Default::default();
                    self.attributes =
                        vec![AttributeIdentifier(EMAIL_ATTRIBUTE_IDENTIFIER.to_owned()); 1];
                    self.tasks = HashMap::default();
                }

                self.status = status;
            }
            Self::Message::Submit => {
                let link = self.link.clone();
                let form = self.form.clone();

                spawn_local(async move {
                    if let Err(e) = encrypt_and_submit(&link, form).await {
                        link.send_message(Self::Message::UpdateStatus(SendFormStatus::Error(e)));
                    }
                });
            }
            Self::Message::AddFiles(files) => {
                for file in files.into_iter() {
                    let file_name = file.name();
                    let task = {
                        let file_name = file_name.clone();
                        let callback = self.link.callback(move |data| {
                            Self::Message::LoadedFile((file_name.clone(), data))
                        });
                        ReaderService::read_file(file, callback).unwrap()
                    };
                    self.tasks.insert(file_name, task);
                }
            }
            Self::Message::LoadedFile((file_name, file)) => {
                self.tasks.remove(&file_name);
                self.form.attachments.push(file);
            }
            Self::Message::DeleteFile(index) => {
                self.form.attachments.remove(index);
            }
        };

        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let disabled = match self.status {
            SendFormStatus::Initial | SendFormStatus::Error(_) => false,
            SendFormStatus::Encrypting | SendFormStatus::Sent => true,
        };
        let multiple = self.form.to.len() > 1;
        let remaining = remaining_attribute_options(&self.attributes);

        html! {
            <form
                class={self.status.class_name()}
                onsubmit=self.link.callback(|e: FocusEvent| { e.prevent_default(); SendFormMsg::Submit })
            >
                {
                    match &self.status {
                        SendFormStatus::Error(e) => html!{
                            <Alert kind=AlertKind::Error>{format!("Error: {}, please try again.", e)}</Alert>
                        },
                        SendFormStatus::Sent => html!{
                            <Alert kind=AlertKind::Success>{"Message encrypted and sent successfully."}</Alert>
                        },
                        _ => html!{
                            <Alert kind=AlertKind::Empty>
                                {"Send encrypted messages with "}
                                <a href="https://irma.app" target="_blank" rel="noopener noreferrer">{"IRMA"}</a>
                            </Alert>
                        }
                    }
                }
                <div>
                    <div>
                        <label>{"Your email:"}</label>
                        <input
                            type="email"
                            name="from"
                            maxlength="512"
                            required=true
                            disabled={disabled}
                            value=self.form.from.clone()
                            oninput=self.link.callback(|event: InputData| Self::Message::UpdateFrom(event.value))
                        />
                    </div>
                    <table class="attributes">
                        <tr>
                            <AttributeHeaderRow
                                attributes=self.attributes.clone()
                                count=self.form.to.len()
                                add_attribute=self.link.callback(Self::Message::AddAttr)
                                delete_attribute=self.link.callback(Self::Message::DeleteAttr)
                            />
                        </tr>
                        { for self.form.to.iter().enumerate().map(|(index, to): (usize, &Recipient)| {
                            html! {
                                <RecipientRow
                                    index=index
                                    to=to.clone()
                                    disabled=disabled
                                    multiple=multiple
                                    update_to=self.link.callback(move |value: String| Self::Message::UpdateTo(index, value))
                                    update_attribute_value=self.link.callback(move |(attr_index, value): (usize, String)| Self::Message::UpdateAttrValue(index, attr_index, value))
                                    delete_to=self.link.callback(move |_| Self::Message::DeleteTo(index))
                                />
                            }
                        }) }
                    </table>
                    <div class="attribute-actions">
                        <button
                            type="button"
                            class="outlined"
                            onclick=self.link.callback(|e: MouseEvent| { e.prevent_default(); Self::Message::AddTo })
                        >
                            {"+ "}
                            {"Add recipient"}
                        </button>

                        { if !remaining.is_empty() {
                            html!{
                                <label class="light">
                                    {"Add attribute for encryption:"}
                                    { for remaining.into_iter().map(|attr| {
                                        let attr_clone = attr.clone();
                                        let label = attribute_label(&attr);

                                        html! {
                                            <button
                                                type="button"
                                                class="outlined"
                                                onclick=self.link.callback(move |e: MouseEvent| { e.prevent_default(); Self::Message::AddAttr(attr_clone.clone()) })
                                            >
                                                {"+ "}
                                                {label}
                                            </button>
                                        }
                                    })}
                                </label>
                            }
                        } else {
                            html!{}
                        }}
                    </div>
                    <div>
                        <label>{"Subject:"}</label>
                        <input
                            name="subject"
                            maxlength="256"
                            required=true
                            disabled={disabled}
                            value=self.form.subject.clone()
                            oninput=self.link.callback(|event: InputData| Self::Message::UpdateSubject(event.value))
                        />
                    </div>
                    <div>
                        <label>{"Message:"}</label>
                        <textarea
                            name="content"
                            maxlength="16384"
                            required=true
                            disabled={disabled}
                            value=self.form.message.clone()
                            oninput=self.link.callback(|event: InputData| Self::Message::UpdateMessage(event.value))
                        />
                    </div>
                    <div>
                        <label>{"Attachments:"}</label>
                        <input type="file" multiple=true onchange=self.link.callback(move |value| {
                            let mut result = Vec::new();
                            if let ChangeData::Files(files) = value {
                                let files = js_sys::try_iter(&files)
                                    .unwrap()
                                    .unwrap()
                                    .map(|v| File::from(v.unwrap()));
                                result.extend(files);
                            }
                            Self::Message::AddFiles(result)
                        })
                        />
                        { if !self.form.attachments.is_empty() {
                            html!{
                                <table class="files">
                                    { for self.form.attachments.iter().enumerate().map(|(index, file)|
                                        html!{
                                            <tr>
                                                <td>
                                                    <span class="filename">
                                                        {file.name.clone()}
                                                    </span>
                                                </td>
                                                <td class="actions">
                                                    <button
                                                        type="button"
                                                        class="delete"
                                                        onclick=self.link.callback(move |_| Self::Message::DeleteFile(index))
                                                    >
                                                        {"×"}
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    )}
                                </table>
                            }
                        } else {
                            html!{}
                        }}
                    </div>
                    <div>
                    {
                        if self.status == SendFormStatus::Sent {
                            html!{
                                <button
                                    type="button"
                                    onclick=self.link.callback(|e: MouseEvent| { e.prevent_default(); Self::Message::UpdateStatus(SendFormStatus::Initial) })
                                >
                                    {"Reset"}
                                </button>
                            }
                        } else {
                            html! {
                                <button
                                    type="submit"
                                    disabled={disabled}
                                >
                                    {"Send"}
                                </button>
                            }
                        }
                    }
                    </div>
                </div>
            </form>
        }
    }
}
