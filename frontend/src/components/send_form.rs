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

use crate::actions::{encrypt_and_submit, sign, SendError};
use crate::attributes::{
    attribute_label, attribute_type, chosen_attribute_options, AttributeType,
    EMAIL_ATTRIBUTE_IDENTIFIER,
};
use crate::components::{
    common::alert::{Alert, AlertKind},
    send_form::{attribute_header_row::AttributeHeaderRow, recipient_row::RecipientRow},
};
use crate::mime::convert_to_mime;
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
    SignAndSubmit,
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
                if index < self.attributes.len() {
                    self.attributes.remove(index);
                    self.form.to.iter_mut().for_each(|el| {
                        el.attributes.remove(index);
                    });
                }
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
            Self::Message::SignAndSubmit => {
                let link = self.link.clone();
                let form = self.form.clone();

                spawn_local(async move {
                    let message = convert_to_mime(&form);
                    match sign(&link, &message).await {
                        Err(e) => {
                            link.send_message(Self::Message::UpdateStatus(SendFormStatus::Error(
                                e,
                            )));
                        }
                        Ok(signature) => {
                            if let Err(e) =
                                encrypt_and_submit(&link, form, message, Some(signature)).await
                            {
                                link.send_message(Self::Message::UpdateStatus(
                                    SendFormStatus::Error(e),
                                ));
                            }
                        }
                    };
                });
            }
            Self::Message::Submit => {
                let link = self.link.clone();
                let form = self.form.clone();

                spawn_local(async move {
                    let message = convert_to_mime(&form);
                    if let Err(e) = encrypt_and_submit(&link, form, message, None).await {
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
        let chosen = chosen_attribute_options(&self.attributes);
        let email_chosen = self
            .attributes
            .iter()
            .find(|&attr| attr == &AttributeIdentifier(EMAIL_ATTRIBUTE_IDENTIFIER.to_owned()));

        html! {
            <form
                class={self.status.class_name()}
                onsubmit=self.link.callback(|e: FocusEvent| { e.prevent_default(); SendFormMsg::Submit })
            >
                {
                    match &self.status {
                        SendFormStatus::Error(e) => html!{
                            <Alert kind=AlertKind::Error>{format!("Error: {}", e)}</Alert>
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

                        { if chosen.len() != self.attributes.len() + if email_chosen.is_some() { 0 } else { 1 } {
                            html!{
                                <div>
                                    <label class="light">
                                        {"Add attribute for encryption:"}
                                    </label>
                                    { for chosen.iter().map(|(chosen_index, attr)| {
                                        let attr_clone = attr.clone();
                                        let label = attribute_label(attr);

                                        if attr.0 == EMAIL_ATTRIBUTE_IDENTIFIER {
                                            return html!{};
                                        }

                                        if chosen_index.is_none() {
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
                                        } else {
                                            html! {}
                                        }
                                    })}
                                </div>
                            }
                        } else {
                            html!{}
                        }}

                        { if self.attributes.len() - if email_chosen.is_some() { 1 } else { 0 } > 0 {
                            html!{
                                <div class="delete">
                                    <label class="light">
                                        {"Delete attribute for encryption:"}
                                    </label>
                                    { for chosen.into_iter().map(|(chosen_index, attr)| {
                                        let label = attribute_label(&attr);

                                        if attr.0 == EMAIL_ATTRIBUTE_IDENTIFIER {
                                            return html!{};
                                        }

                                        if let Some(index) = chosen_index {
                                            html! {
                                                <button
                                                    type="button"
                                                    class="outlined delete"
                                                    onclick=self.link.callback(move |e: MouseEvent| { e.prevent_default(); Self::Message::DeleteAttr(index) })
                                                >
                                                    {"- "}
                                                    {index}
                                                    {": "}
                                                    {label}
                                                </button>
                                            }
                                        } else {
                                            html! {
                                            }
                                        }
                                    })}
                                </div>
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
                        <input
                            type="file"
                            multiple=true
                            onchange=self.link.callback(move |value| {
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
                                                <td>
                                                    {format!("{:.2} MB", file.content.len() as f32 / 1_000_000.0)}
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
                                <div>
                                    <button
                                        type="submit"
                                        disabled={disabled}
                                    >
                                        <svg role="img" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">
                                            <path fill="currentColor" d="M464 4.3L16 262.7C-7 276-4.7 309.9 19.8 320L160 378v102c0 30.2 37.8 43.3 56.7 20.3l60.7-73.8 126.4 52.2c19.1 7.9 40.7-4.2 43.8-24.7l64-417.1C515.7 10.2 487-9 464 4.3zM192 480v-88.8l54.5 22.5L192 480zm224-30.9l-206.2-85.2 199.5-235.8c4.8-5.6-2.9-13.2-8.5-8.4L145.5 337.3 32 290.5 480 32l-64 417.1z"></path>
                                        </svg>
                                        {"Send"}
                                    </button>
                                    {" or "}
                                    <button
                                        type="submit"
                                        disabled={disabled}
                                        onclick=self.link.callback(|e: MouseEvent| { e.prevent_default(); Self::Message::SignAndSubmit })
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 512">
                                            <path fill="currentColor" d="M630.1 206.8c-32.7 8.4-77.1 30.8-108.7 49.6-14.7 8.8-29.9 17.9-41.5 23.4-31.3 15.2-58.3 28.2-84.1 28.2-12.9 0-23-4.3-29.9-12.7-11.3-13.8-11.3-35.3-7.9-63.3 3.3-26.9.6-47.8-7.6-57.3-4.4-5-10-6.9-18.7-6.3C307 170 257 210.8 169.6 300.9l-54.4 56 70-187.6c11.8-31.6 3.6-66.1-20.9-87.8C145 64.3 112 54.3 77.3 78L3.5 127.8c-3.5 2.3-4.5 7-2.5 10.6l9.2 16.2c2.3 4.1 7.6 5.3 11.5 2.7L97.5 106c6.6-4.5 14-6.8 21.4-6.8 9.1 0 17.6 3.4 24.8 9.8 13.2 11.7 17.6 30.2 11.3 47.1L55.2 423.7c-1.9 5.2-1 12.6 2.2 17.7 2.4 3.7 6.6 6.1 11.3 6.6 4.9.3 9.7-1.4 13-4.9C125 396.8 239.5 278.4 298 228.4l20.4-17.4c3.4-2.9 8.5-.3 8.2 4.1l-2.1 27.9c-2 27.3-2.4 55.9 16.8 78.6 12.4 14.5 30.7 21.9 54.6 21.9 32.7 0 64.1-15.1 97.3-31.1 10.2-4.9 24.9-14.1 39.2-23 30.9-19.3 72.3-40.5 101.8-47.7 3.5-.9 5.9-4 5.9-7.6v-17.3c-.1-7.4-5-11.2-10-10z"></path>
                                        </svg>
                                        {"Sign and send"}
                                    </button>
                                </div>
                            }
                        }
                    }
                    </div>
                </div>
            </form>
        }
    }
}
