use yew::prelude::*;

use crate::attributes::EMAIL_ATTRIBUTE_IDENTIFIER;
use crate::components::send_form::attribute_input::AttributeInput;
use crate::types::Recipient;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub update_to: Callback<String>,
    pub update_attribute_value: Callback<(usize, String)>,
    pub delete_to: Callback<()>,
    pub index: usize,
    pub disabled: bool,
    pub multiple: bool,
    pub to: Recipient,
}

pub enum AttributeRowMsg {
    UpdateTo(String),
    UpdateAttrValue(usize, String),
    DeleteTo,
}

pub struct RecipientRow {
    props: Props,
    link: ComponentLink<Self>,
}

impl Component for RecipientRow {
    type Properties = Props;
    type Message = AttributeRowMsg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        RecipientRow { props, link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            AttributeRowMsg::UpdateTo(value) => {
                self.props.update_to.emit(value);
            }
            AttributeRowMsg::UpdateAttrValue(attr_index, value) => {
                self.props.update_attribute_value.emit((attr_index, value));
            }
            AttributeRowMsg::DeleteTo => {
                self.props.delete_to.emit(());
            }
        }

        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        html! {
            <tr>
                <td>
                    <input
                        type="email"
                        name="to"
                        maxlength="512"
                        required=true
                        placeholder="user@example.com"
                        disabled={self.props.disabled}
                        value=self.props.to.to.clone()
                        class="inline"
                        oninput=self.link.callback(move |event: InputData| Self::Message::UpdateTo(event.value))
                    />
                </td>
                { for self.props.to.attributes.iter().enumerate().filter(|(_, attr)| attr.identifier.0 != EMAIL_ATTRIBUTE_IDENTIFIER).map(|(attr_index, attr)| {
                    html!{
                        <td>
                            <AttributeInput
                                attribute=attr.clone()
                                disabled=self.props.disabled
                                update_attribute_value=self.link.callback(move |value: String| Self::Message::UpdateAttrValue(attr_index, value))
                            />
                        </td>
                    }
                })}
                { if self.props.multiple {
                    html!{
                        <td class="actions">
                            <button
                                type="button"
                                class="delete"
                                onclick=self.link.callback(|_| Self::Message::DeleteTo)
                            >
                                {"×"}
                            </button>
                        </td>
                    }
                } else {
                    html! {}
                }}
            </tr>
        }
    }
}
