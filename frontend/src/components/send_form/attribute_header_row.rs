use crate::attributes::{attribute_label, EMAIL_ATTRIBUTE_IDENTIFIER};
use common::AttributeIdentifier;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub count: usize,
    pub attributes: Vec<AttributeIdentifier>,
    pub add_attribute: Callback<AttributeIdentifier>,
    pub delete_attribute: Callback<usize>,
}

pub enum AttributeHeaderRowMsg {
    ToggleEmail(Option<usize>),
    DeleteAttr(usize),
}

pub struct AttributeHeaderRow {
    props: Props,
    link: ComponentLink<Self>,
}

impl Component for AttributeHeaderRow {
    type Properties = Props;
    type Message = AttributeHeaderRowMsg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        AttributeHeaderRow { props, link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            AttributeHeaderRowMsg::ToggleEmail(email_index) => {
                if let Some(index) = email_index {
                    self.props.delete_attribute.emit(index);
                } else {
                    self.props
                        .add_attribute
                        .emit(AttributeIdentifier(EMAIL_ATTRIBUTE_IDENTIFIER.to_owned()));
                }
            }
            AttributeHeaderRowMsg::DeleteAttr(index) => {
                self.props.delete_attribute.emit(index);

                if self.props.attributes.is_empty() {
                    self.props
                        .add_attribute
                        .emit(AttributeIdentifier(EMAIL_ATTRIBUTE_IDENTIFIER.to_owned()));
                }
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
        let email_index = self
            .props
            .attributes
            .iter()
            .position(|attr| attr.0 == EMAIL_ATTRIBUTE_IDENTIFIER);
        let render_attributes = self
            .props
            .attributes
            .iter()
            .enumerate()
            .filter(|&(_, attr)| attr.0 != EMAIL_ATTRIBUTE_IDENTIFIER);

        html! {
            <>
                <th>
                    <div>
                        <label>{ if self.props.count > 1 { "Recipients:" } else { "Recipient:" }}</label>
                        {if render_attributes.clone().count() > 0  {
                            html! {
                                <label class="checkbox-label">
                                    <input
                                        type="checkbox"
                                        id="encrypt-for-email"
                                        checked={email_index != None}
                                        onclick=self.link.callback(move |_| Self::Message::ToggleEmail(email_index))
                                    />
                                    {"Encrypt for e-mail address"}
                                </label>
                            }
                        } else {
                            html!{}
                        }}
                    </div>
                </th>
                { for render_attributes.map(|(index, attribute): (usize, &AttributeIdentifier)| {
                    html! {
                        <th>
                            <div>
                                <label>
                                    {attribute_label(attribute)}
                                </label>
                                <button
                                    class="delete small"
                                    onclick=self.link.callback(move |e: MouseEvent| { e.prevent_default(); Self::Message::DeleteAttr(index) })
                                >
                                    {"×"}
                                </button>
                            </div>
                        </th>
                    }
                })}
                { if self.props.count > 1 {
                    html! { <th></th> }
                } else {
                    html!{}
                }}
            </>
        }
    }
}
