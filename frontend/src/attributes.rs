use common::AttributeIdentifier;

#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub enum AttributeType {
    Boolean,
    Number,
    String,
}

pub const EMAIL_ATTRIBUTE_IDENTIFIER: &str = "pbdf.sidn-pbdf.email.email";
const NAME_ATTRIBUTE_IDENTIFIER: &str = "pbdf.gemeente.personalData.fullname";
const IBAN_ATTRIBUTE_IDENTIFIER: &str = "pbdf.pbdf.ideal.iban";
const PHONE_NUMBER_ATTRIBUTE_IDENTIFIER: &str = "pbdf.sidn-pbdf.mobilenumber.mobilenumber";

pub const ALLOWED_ATTRIBUTES: [&str; 4] = [
    EMAIL_ATTRIBUTE_IDENTIFIER,
    NAME_ATTRIBUTE_IDENTIFIER,
    IBAN_ATTRIBUTE_IDENTIFIER,
    PHONE_NUMBER_ATTRIBUTE_IDENTIFIER,
];

pub fn attribute_label(identifier: &AttributeIdentifier) -> &str {
    let ident = identifier.0.as_ref();
    match ident {
        EMAIL_ATTRIBUTE_IDENTIFIER => "Email",
        NAME_ATTRIBUTE_IDENTIFIER => "Full name",
        IBAN_ATTRIBUTE_IDENTIFIER => "IBAN",
        PHONE_NUMBER_ATTRIBUTE_IDENTIFIER => "Phone number",
        s => s,
    }
}

pub fn attribute_type(_identifier: &AttributeIdentifier) -> AttributeType {
    AttributeType::String
}

pub fn chosen_attribute_options(
    chosen: &[AttributeIdentifier],
) -> Vec<(Option<usize>, AttributeIdentifier)> {
    ALLOWED_ATTRIBUTES
        .iter()
        .map(|item| AttributeIdentifier(item.to_string()))
        .map(|item| -> (Option<usize>, AttributeIdentifier) {
            let index = chosen.iter().position(|i| i == &item);

            (index, item)
        })
        .collect()
}
