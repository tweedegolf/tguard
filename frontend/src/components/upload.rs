use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use yew::{
    html,
    services::reader::{File, FileData, ReaderService, ReaderTask},
    ChangeData, Component, ComponentLink, Html, ShouldRender,
};

use crate::{
    components::common::alert::{Alert, AlertKind},
    decrypt::{decrypt_file, DecryptError},
    mime::convert_from_mime,
    types::File as FileType,
};

type FileName = String;
type Message = String;

#[derive(PartialEq)]
pub enum UploadFormStatus {
    Initial,
    Error(DecryptError),
    Decrypting,
    Success,
}

pub enum UploadMsg {
    Loaded((FileName, FileData)),
    Decrypted((FileName, String, Vec<FileType>)),
    DecryptionFailed(DecryptError),
    AddFiles(Vec<File>),
}

pub struct Upload {
    link: ComponentLink<Self>,
    status: UploadFormStatus,
    decrypted: Vec<(FileName, Message, Vec<FileType>)>,
    tasks: HashMap<FileName, ReaderTask>,
}

fn convert_and_parse(raw: &[u8]) -> Option<(String, Vec<FileType>)> {
    let plain = String::from_utf8(raw.to_vec()).ok()?;
    let converted = convert_from_mime(&plain)?;
    Some(converted)
}

impl Component for Upload {
    type Properties = ();
    type Message = UploadMsg;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            status: UploadFormStatus::Initial,
            tasks: HashMap::default(),
            decrypted: vec![],
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Self::Message::DecryptionFailed(e) => {
                self.status = UploadFormStatus::Error(e);
                true
            }
            Self::Message::Decrypted(message) => {
                self.decrypted.push(message);
                self.status = UploadFormStatus::Success;
                true
            }
            Self::Message::Loaded((filename, file)) => {
                self.status = UploadFormStatus::Decrypting;
                self.tasks.remove(&filename);
                let link = self.link.clone();

                spawn_local(async move {
                    match decrypt_file(&file).await {
                        Ok(content) => {
                            let (message, attachments) = convert_and_parse(&content).unwrap_or((
                                String::new(),
                                vec![FileType {
                                    filename: filename.clone(),
                                    mimetype: "application/octet-stream".to_owned(),
                                    content,
                                }],
                            ));
                            link.send_message(Self::Message::Decrypted((
                                filename,
                                message,
                                attachments,
                            )))
                        }
                        Err(e) => link.send_message(Self::Message::DecryptionFailed(e)),
                    };
                });

                true
            }
            Self::Message::AddFiles(files) => {
                for file in files.into_iter() {
                    let filename = file.name();
                    let task = {
                        let filename = filename.clone();
                        let callback = self
                            .link
                            .callback(move |data| Self::Message::Loaded((filename.clone(), data)));
                        ReaderService::read_file(file, callback).unwrap()
                    };
                    self.tasks.insert(filename, task);
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
                { if !self.decrypted.is_empty() {
                    html!{
                        { for self.decrypted.iter().map(Self::view_decrypted) }
                    }
                } else {
                    html!{}
                }}
            </>
        }
    }
}

impl Upload {
    fn view_decrypted(decrypted: &(FileName, Message, Vec<FileType>)) -> Html {
        html! {
            <div class="decrypted">
                <dl>
                    <dt>{"File name:"}</dt>
                    <dd>{decrypted.0.clone()}</dd>
                    {if decrypted.1.is_empty() {
                        html!{}
                    } else {
                        html!{
                            <>
                                <dt>{"Message:"}</dt>
                                <dd>
                                    <pre>
                                      {decrypted.1.clone()}
                                    </pre>
                                </dd>
                            </>
                        }
                    }}
                </dl>
                <label>
                    {if decrypted.1.is_empty() {
                        "Attachments:"
                    } else {
                        "Decrypted:"
                    }}
                </label>
                <table class="files">
                    { for decrypted.2.iter().map(Self::view_file) }
                </table>
            </div>
        }
    }

    fn view_file(data: &FileType) -> Html {
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
