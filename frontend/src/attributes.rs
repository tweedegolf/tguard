use common::AttributeIdentifier;

#[derive(Clone, PartialEq)]
pub enum AttributeType {
    Boolean,
    Number,
    String,
}

pub const EMAIL_ATTRIBUTE_IDENTIFIER: &str = "pbdf.sidn-pbdf.email.email";
const CITY_ATTRIBUTE_IDENTIFIER: &str = "pbdf.gemeente.address.city";
const OVER18_ATTRIBUTE_IDENTIFIER: &str = "pbdf.gemeente.personalData.over18";
const HOUSENUMBER_ATTRIBUTE_IDENTIFIER: &str = "pbdf.gemeente.address.houseNumber";

pub const ALLOWED_ATTRIBUTES: [&str; 4] = [
    EMAIL_ATTRIBUTE_IDENTIFIER,
    CITY_ATTRIBUTE_IDENTIFIER,
    OVER18_ATTRIBUTE_IDENTIFIER,
    HOUSENUMBER_ATTRIBUTE_IDENTIFIER,
];

pub fn attribute_label(identifier: &AttributeIdentifier) -> &str {
    let ident = identifier.0.as_ref();
    match ident {
        EMAIL_ATTRIBUTE_IDENTIFIER => "Email",
        CITY_ATTRIBUTE_IDENTIFIER => "City",
        OVER18_ATTRIBUTE_IDENTIFIER => "Over 18",
        HOUSENUMBER_ATTRIBUTE_IDENTIFIER => "Huisnummer",
        s => s,
    }
}

pub fn attribute_type(identifier: &AttributeIdentifier) -> AttributeType {
    let ident = identifier.0.as_ref();
    match ident {
        OVER18_ATTRIBUTE_IDENTIFIER => AttributeType::Boolean,
        HOUSENUMBER_ATTRIBUTE_IDENTIFIER => AttributeType::Number,
        _ => AttributeType::String,
    }
}

pub fn remaining_attribute_options(chosen: &[AttributeIdentifier]) -> Vec<AttributeIdentifier> {
    ALLOWED_ATTRIBUTES
        .iter()
        .map(|item| AttributeIdentifier(item.to_string()))
        .filter(|ident| !chosen.contains(ident) && ident.0 != EMAIL_ATTRIBUTE_IDENTIFIER)
        .collect()
}
