use core::convert::TryFrom;
use core::convert::TryInto;

use arrayvec::ArrayVec;
use ibe::kiltz_vahlis_one::Identity;
use js_sys::Uint8Array;
use rand::Rng;

use common::{AttributeValue, RecipientMessage, SealedMessage};

use crate::js_functions::{decrypt, encrypt};
use crate::types::{FormData, Recipient};

fn derive_identity(to: &Recipient, timestamp: u64) -> Option<Identity> {
    let mut buf = ArrayVec::<u8, 1024>::new();

    buf.try_extend_from_slice(&timestamp.to_be_bytes()).ok()?;

    // TODO use all attributes
    let attribute: AttributeValue = to.attributes[0].clone();

    let at = attribute.identifier.0.as_bytes();
    let at_len = u8::try_from(at.len()).ok()?;
    buf.try_extend_from_slice(&[at_len]).ok()?;
    buf.try_extend_from_slice(at).ok()?;

    let av = attribute.value.as_bytes();
    let av_len = u8::try_from(av.len()).ok()?;
    buf.try_extend_from_slice(&[av_len]).ok()?;
    buf.try_extend_from_slice(av).ok()?;

    Some(Identity::derive(&buf))
}

pub async fn seal(
    public_key: String,
    form: &FormData,
    message: String,
) -> Option<Vec<RecipientMessage>> {
    let b: [u8; 25056] = base64::decode(public_key).ok()?.try_into().ok()?;
    let pk = Option::from(ibe::kiltz_vahlis_one::PublicKey::from_bytes(&b))?;

    let timestamp = (js_sys::Date::now() / 1000.0) as u64;

    let mut messages = Vec::<RecipientMessage>::with_capacity(form.to.len());

    for to in &form.to {
        let derived = derive_identity(to, timestamp)?;

        let mut rng = rand::thread_rng();
        let (c, k) = ibe::kiltz_vahlis_one::encrypt(&pk, &derived, &mut rng);
        let iv: [u8; 16] = rng.gen();

        let packed = message.clone();

        let ct = encrypt(packed.clone(), &k.to_bytes(), &iv).await;
        let ct = Uint8Array::new(&ct);

        let message = SealedMessage {
            iv: base64::encode(&iv.to_vec()),
            ct: base64::encode(&ct.to_vec()),
            c_key: base64::encode(&c.to_bytes()),
            timestamp,
            attributes: to.attributes.clone(),
        };

        messages.push(RecipientMessage {
            to: to.to.clone(),
            sealed: message,
        });
    }

    Some(messages)
}

pub async fn unseal(sm: &SealedMessage, usk: String) -> Option<String> {
    let usk_data: [u8; 192] = base64::decode(&usk).ok()?.try_into().ok()?;
    let usk = Option::from(ibe::kiltz_vahlis_one::UserSecretKey::from_bytes(&usk_data))?;

    let ct_key_data: [u8; 144] = base64::decode(&sm.c_key).ok()?.try_into().ok()?;
    let ct_key = Option::from(ibe::kiltz_vahlis_one::CipherText::from_bytes(&ct_key_data))?;

    let k = ibe::kiltz_vahlis_one::decrypt(&usk, &ct_key);

    let iv = base64::decode(&sm.iv).ok()?;
    let ct = base64::decode(&sm.ct).ok()?;

    decrypt(&ct, &k.to_bytes(), &iv).await.as_string()
}
