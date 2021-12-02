use common::AttributeValue;
use yew::prelude::*;

use crate::attributes::{attribute_type, AttributeType};

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub attribute: AttributeValue,
    pub disabled: bool,
    pub update_attribute_value: Callback<String>,
}

pub enum AttributeInputMsg {
    UpdateInput(String),
}

pub struct AttributeInput {
    props: Props,
    link: ComponentLink<Self>,
}

impl Component for AttributeInput {
    type Properties = Props;
    type Message = AttributeInputMsg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        AttributeInput { props, link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            AttributeInputMsg::UpdateInput(value) => {
                self.props.update_attribute_value.emit(value);
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
        match attribute_type(&self.props.attribute.identifier) {
            AttributeType::Boolean => html! {
                <p>{self.props.attribute.value.clone()}</p>
            },
            AttributeType::Number => html! {
                <input
                    type="number"
                    name="attr"
                    maxlength="512"
                    required=true
                    disabled={self.props.disabled}
                    value=self.props.attribute.value.clone()
                    class="inline"
                    oninput=self.link.callback(|event:InputData| Self::Message::UpdateInput(event.value))
                />
            },
            AttributeType::String => html! {
                <input
                    type="text"
                    name="attr"
                    maxlength="512"
                    required=true
                    disabled={self.props.disabled}
                    value=self.props.attribute.value.clone()
                    class="inline"
                    oninput=self.link.callback(|event:InputData| Self::Message::UpdateInput(event.value))
                />
            },
        }
    }
}
