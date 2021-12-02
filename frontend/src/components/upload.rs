use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::{html, ChangeData, Component, ComponentLink, Html, ShouldRender};

use crate::components::common::alert::{Alert, AlertKind};
use crate::decrypt::{decrypt_file, DecryptError};

type FileName = String;

#[derive(PartialEq)]
pub enum UploadFormStatus {
    Initial,
    Error(DecryptError),
    Decrypting,
    Success,
}

pub enum UploadMsg {
    Loaded((FileName, FileData)),
    Decrypted((String, Vec<u8>)),
    DecryptionFailed(DecryptError),
    AddFiles(Vec<File>),
}

pub struct Upload {
    link: ComponentLink<Self>,
    status: UploadFormStatus,
    files: Vec<FileData>,
    tasks: HashMap<FileName, ReaderTask>,
}

impl Component for Upload {
    type Properties = ();
    type Message = UploadMsg;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            status: UploadFormStatus::Initial,
            tasks: HashMap::default(),
            files: vec![],
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Self::Message::DecryptionFailed(e) => {
                self.status = UploadFormStatus::Error(e);
                true
            }
            Self::Message::Decrypted((file_name, content)) => {
                self.files.push(FileData {
                    name: file_name,
                    content,
                });
                self.status = UploadFormStatus::Success;
                true
            }
            Self::Message::Loaded((file_name, file)) => {
                self.status = UploadFormStatus::Decrypting;
                self.tasks.remove(&file_name);
                let link = self.link.clone();

                spawn_local(async move {
                    match decrypt_file(&file).await {
                        Ok(content) => {
                            link.send_message(Self::Message::Decrypted((file_name, content)))
                        }
                        Err(e) => link.send_message(Self::Message::DecryptionFailed(e)),
                    };
                });

                true
            }
            Self::Message::AddFiles(files) => {
                for file in files.into_iter() {
                    let file_name = file.name();
                    let task = {
                        let file_name = file_name.clone();
                        let callback = self
                            .link
                            .callback(move |data| Self::Message::Loaded((file_name.clone(), data)));
                        ReaderService::read_file(file, callback).unwrap()
                    };
                    self.tasks.insert(file_name, task);
                }
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                {
                    match &self.status {
                        UploadFormStatus::Error(e) => html!{
                            <Alert kind=AlertKind::Error>{format!("Error: {}, please try a different file", e)}</Alert>
                        },
                        UploadFormStatus::Success => html!{
                            <Alert kind=AlertKind::Success>{"Message decrypted successfully"}</Alert>
                        },
                        _ => html!{
                            <Alert kind=AlertKind::Empty>
                                {"Select an IRMASeal file to decrypt"}
                            </Alert>
                        }
                    }
                }
                <div>
                    <input type="file" multiple=false onchange=self.link.callback(move |value| {
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
                </div>
                { if !self.files.is_empty() {
                    html!{
                        <label>{"Download decrypted files:"}</label>
                    }
                } else {
                    html!{}
                }}
                <table class="files">
                    { for self.files.iter().map(|f| Self::view_file(f)) }
                </table>
            </>
        }
    }
}

impl Upload {
    fn view_file(data: &FileData) -> Html {
        html! {
            <tr>
                <td>
                    <p class="filename">
                        {(&data.name).to_string()}
                    </p>
                </td>
                <td class="actions">
                    <a class="button outlined" download={data.name.clone()} href={ format!("data:text/plain;base64,{}", base64::encode({ &data.content })) } target="_blank">
                        {"download"}
                    </a>
                </td>
            </tr>
        }
    }
}
