use regex::Regex;
use wasm_bindgen_test::*;

use tguard_frontend::{
    mime::{convert_from_mime, convert_to_mime},
    types::{FormData, Recipient},
};

use wasm_bindgen_test::wasm_bindgen_test_configure;
wasm_bindgen_test_configure!(run_in_browser);

fn replace_boundary(mail: &str) -> String {
    let re_boundary = Regex::new(r"boundary=(.{40})").unwrap();
    let captures = re_boundary.captures(mail).unwrap();

    let boundary = &captures[1];
    let re = Regex::new(boundary).unwrap();
    re.replace_all(mail, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .to_string()
}

#[wasm_bindgen_test]
fn test_convert_to_mime() {
    let form_data = FormData {
        from: String::default(),
        to: vec![Recipient {
            to: String::default(),
            attributes: vec![],
        }],
        subject: "Test subject".to_owned(),
        message: "This is a test message.\nKind regards".to_owned(),
        attachments: vec![],
    };

    let mail = replace_boundary(&convert_to_mime(&form_data));
    let test_mail = "Content-Type: multipart/mixed; \r\n boundary=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\r\n\r\n--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: 7bit\r\n\r\nThis is a test message.\r\nKind regards\r\n--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa--\r\n";
    assert_eq!(mail, test_mail);
}

// message from Tguard
#[wasm_bindgen_test]
fn test_convert_from_mime() {
    let test_mail = "Content-Type: multipart/mixed; \r\n boundary=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\r\n\r\n--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: 7bit\r\n\r\nThis is a test message.\r\nKind regards\r\n--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa--\r\n";
    let mail = convert_from_mime(test_mail).unwrap();
    assert_eq!(
        mail.0,
        "This is a test message.\r\nKind regards\r".to_owned()
    );
    assert!(mail.1.is_empty());
}

// not a mime message
#[wasm_bindgen_test]
fn test_convert_from_mime_none() {
    let test_mail = "This is not a mime message";
    let mail = convert_from_mime(test_mail);
    assert!(mail.is_none());
}

// plain single part
#[wasm_bindgen_test]
fn test_convert_from_mime_plain() {
    let test_mail = r#"Content-Type: text/plain; charset=utf-8
Content-Transfer-Encoding: 7bit

This is a test message.
Kind regards
"#;
    let mail = convert_from_mime(test_mail).unwrap();
    assert_eq!(mail.0, "This is a test message.\nKind regards\n".to_owned());
    assert!(mail.1.is_empty());
}

// HTML message from Thunderbird
#[wasm_bindgen_test]
fn test_convert_from_mime_html_and_plain() {
    let test_mail = r#"Content-Type: multipart/alternative;
 boundary="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
 Content-Language: en-US

This is a multi-part message in MIME format.
--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
Content-Type: text/plain; charset=utf-8; format=flowed
Content-Transfer-Encoding: 7bit

This is an *HTML* test message
--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
Content-Type: text/html; charset=utf-8
Content-Transfer-Encoding: 7bit

<html>
  <head>
    <meta http-equiv="content-type" content="text/html; charset=UTF-8">
  </head>
  <body>
    This is an <b>HTML</b> test message
  </body>
</html>

--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa--
"#;
    let mail = convert_from_mime(test_mail).unwrap();
    assert_eq!(mail.0, "This is an *HTML* test message".to_owned());
    assert!(mail.1.is_empty());
}

// HTML message from Thunderbird, plain stripped
#[wasm_bindgen_test]
fn test_convert_from_mime_html_only() {
    let test_mail = r#"Content-Type: multipart/mixed;
 boundary="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
 Content-Language: en-US

--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
Content-Type: text/html; charset=utf-8
Content-Transfer-Encoding: 7bit

<html>
  <head>
    <meta http-equiv="content-type" content="text/html; charset=UTF-8">
  </head>
  <body>
    This is an <b>HTML</b> test message
  </body>
</html>

--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa--
"#;
    let mail = convert_from_mime(test_mail).unwrap();
    assert_eq!(mail.0, "This is an HTML test message".to_owned());
    assert!(mail.1.is_empty());
}

// HTML single part
#[wasm_bindgen_test]
fn test_convert_from_mime_html() {
    let test_mail = r#"Content-Type: text/html; charset=utf-8
Content-Transfer-Encoding: 7bit

<html>
  <head>
    <meta http-equiv="content-type" content="text/html; charset=UTF-8">
  </head>
  <body>
    This is an <b>HTML</b> test message
  </body>
</html>
"#;
    let mail = convert_from_mime(test_mail).unwrap();
    assert_eq!(mail.0, "This is an HTML test message".to_owned());
    assert!(mail.1.is_empty());
}

// HTML single part with script
#[wasm_bindgen_test]
fn test_convert_from_mime_html_script() {
    let test_mail = r#"Content-Type: text/html; charset=utf-8
Content-Transfer-Encoding: 7bit

<script>alert(1);</script>
"#;
    let mail = convert_from_mime(test_mail).unwrap();
    assert_eq!(mail.0, "alert(1);".to_owned());
    assert!(mail.1.is_empty());
}

// plain single part with script
#[wasm_bindgen_test]
fn test_convert_from_mime_plain_script() {
    let test_mail = r#"Content-Type: text/plain; charset=utf-8
Content-Transfer-Encoding: 7bit

<script>alert(1);</script>
"#;
    let mail = convert_from_mime(test_mail).unwrap();
    assert_eq!(mail.0, "<script>alert(1);</script>\n".to_owned());
    assert!(mail.1.is_empty());
}

// Message with attachment from Thunderbird
#[wasm_bindgen_test]
fn test_convert_from_mime_attachment() {
    let test_mail = r#"Content-Type: multipart/mixed;
 boundary="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
Content-Language: en-US

This is a multi-part message in MIME format.
--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
Content-Type: text/plain; charset=utf-8; format=flowed
Content-Transfer-Encoding: 7bit

Test message
--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
Content-Type: image/svg+xml;
 name="image.   "
Content-Transfer-Encoding: base64
Content-Disposition: attachment;
 filename="image.svg"

PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMTk5OS9zdmciPgo8c2NyaXB0PgphbGVy
dCgxKQo8L3NjcmlwdD4KPC9zdmc+Cg==
--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa--
"#;
    let mail = convert_from_mime(test_mail).unwrap();
    assert_eq!(mail.0, "Test message".to_owned());
    assert_eq!(mail.1.len(), 1);
    assert_eq!(mail.1[0].filename, "image.svg");
    assert_eq!(
        String::from_utf8(mail.1[0].content.clone()).unwrap(),
        "<svg xmlns=\"http://www.w3.org/1999/svg\">\n<script>\nalert(1)\n</script>\n</svg>\n"
    );
    assert_eq!(mail.1[0].mimetype, "image/svg+xml");
}

// Message from Thunderbird, forwarded with Thunderbird
#[wasm_bindgen_test]
fn test_convert_from_mime_forwarded() {
    let test_mail = r#"Content-Type: multipart/alternative;
 boundary="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
Content-Language: en-US

This is a multi-part message in MIME format.
--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
Content-Type: text/plain; charset=utf-8; format=flowed
Content-Transfer-Encoding: 7bit

This is a forwarded HTML test message


-------- Forwarded Message --------
Subject:  HTML to forward
Date:   Tue, 14 Dec 2021 17:02:44 +0100
From:   Name <email@example.com>
To:   email@example.com


This is an *HTML* test message
--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
Content-Type: text/html; charset=utf-8
Content-Transfer-Encoding: 7bit

<html>
  <head>
    <meta http-equiv="content-type" content="text/html; charset=UTF-8">
  </head>
  <body>
    <div class="moz-forward-container"
      -------- Forwarded Message --------
      <table class="moz-email-headers-table" cellspacing="0"
        cellpadding="0" border="0">
        <tbody>
          <tr>
            <th valign="BASELINE" nowrap="nowrap" align="RIGHT">Subject:
            </th>
            <td>This is an HTML test message</td>
          </tr>
          <tr>
            <th valign="BASELINE" nowrap="nowrap" align="RIGHT">Date: </th>
            <td>Tue, 14 Dec 2021 17:02:44 +0100</td>
          </tr>
          <tr>
            <th valign="BASELINE" nowrap="nowrap" align="RIGHT">From: </th>
            <td>Name <a class="moz-txt-link-rfc2396E" href="mailto:email@example.com">&lt;email@example.com&gt;</a></td>
          </tr>
          <tr>
            <th valign="BASELINE" nowrap="nowrap" align="RIGHT">To: </th>
            <td><a class="moz-txt-link-abbreviated" href="mailto:email@example.com">email@example.com</a></td>
          </tr>
        </tbody>
      </table>
      This is an *HTML* test message
    </div>
  </body>
</html>

--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa--
"#;
    let mail = convert_from_mime(test_mail).unwrap();
    assert_eq!(
        mail.0,
        "This is a forwarded HTML test message\n\n\n-------- Forwarded Message --------\nSubject:  HTML to forward\nDate:   Tue, 14 Dec 2021 17:02:44 +0100\nFrom:   Name <email@example.com>\nTo:   email@example.com\n\n\nThis is an *HTML* test message".to_owned()
    );
    assert!(mail.1.is_empty());
}

// Message from Thunderbird, downloaded with Thunderbird, attached with Thunderbird
#[wasm_bindgen_test]
fn test_convert_from_mime_email_attached() {
    let test_mail = r#"Content-Type: multipart/mixed;
 boundary="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
Content-Language: en-US

This is a multi-part message in MIME format.
--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
Content-Type: text/plain; charset=utf-8; format=flowed
Content-Transfer-Encoding: 7bit

Body of email with email as attachment
--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
Content-Type: message/rfc822;
 name="attached_email.eml"
Content-Transfer-Encoding: 7bit
Content-Disposition: attachment;
 filename="attached_email.eml"

To: Name <email@example.com>
From: Name <email@example.com>
Subject: HTML test
Message-ID: <random-message-id@example.com>
Date: Tue, 14 Dec 2021 11:48:25 +0100
User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:78.0) Gecko/20100101
 Thunderbird/78.14.0
MIME-Version: 1.0
Content-Type: multipart/alternative;
 boundary="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
Content-Language: en-US

This is a multi-part message in MIME format.
--bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
Content-Type: text/plain; charset=utf-8; format=flowed
Content-Transfer-Encoding: 7bit

This is an *HTML* test message

--bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
Content-Type: text/html; charset=utf-8
Content-Transfer-Encoding: 7bit

<html>
  <head>
    <meta http-equiv="content-type" content="text/html; charset=UTF-8">
  </head>
  <body>
    This is an <b>HTML</b> test message
  </body>
</html>

--bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb--
--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa--
"#;
    let mail = convert_from_mime(test_mail).unwrap();
    assert_eq!(mail.0, "Body of email with email as attachment".to_owned());
    assert_eq!(mail.1.len(), 1);
    assert_eq!(mail.1[0].filename, "attached_email.eml");
    assert_eq!(
        String::from_utf8(mail.1[0].content.clone()).unwrap(),
        r#"To: Name <email@example.com>
From: Name <email@example.com>
Subject: HTML test
Message-ID: <random-message-id@example.com>
Date: Tue, 14 Dec 2021 11:48:25 +0100
User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:78.0) Gecko/20100101
 Thunderbird/78.14.0
MIME-Version: 1.0
Content-Type: multipart/alternative;
 boundary="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
Content-Language: en-US

This is a multi-part message in MIME format.
--bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
Content-Type: text/plain; charset=utf-8; format=flowed
Content-Transfer-Encoding: 7bit

This is an *HTML* test message

--bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
Content-Type: text/html; charset=utf-8
Content-Transfer-Encoding: 7bit

<html>
  <head>
    <meta http-equiv="content-type" content="text/html; charset=UTF-8">
  </head>
  <body>
    This is an <b>HTML</b> test message
  </body>
</html>

--bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb--"#
    );
    assert_eq!(mail.1[0].mimetype, "message/rfc822");
}

#[wasm_bindgen_test]
fn test_convert_from_mime_email_base64_attached() {
    let test_mail = r#"Content-Type: multipart/mixed;
 boundary=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb

--bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
Content-Type: text/plain; charset=utf-8
Content-Transfer-Encoding: 7bit

This is a message with a base64 encoded attached email
--bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
Content-Disposition: attachment; filename="attached_email.eml"
Content-Type: message/rfc822
Content-Transfer-Encoding: base64

VG86IE5hbWUgPGVtYWlsQGV4YW1wbGUuY29tPg0KRnJvbTogTmFtZSA8ZW1haWxAZXhhbXBsZS5j
b20+DQpTdWJqZWN0OiBIVE1MIHRlc3QNCk1lc3NhZ2UtSUQ6IDxyYW5kb20tbWVzc2FnZS1pZEBl
eGFtcGxlLmNvbT4NCkRhdGU6IFR1ZSwgMTQgRGVjIDIwMjEgMTE6NDg6MjUgKzAxMDANCk1JTUUt
VmVyc2lvbjogMS4wDQpDb250ZW50LVR5cGU6IG11bHRpcGFydC9hbHRlcm5hdGl2ZTsNCiBib3Vu
ZGFyeT0iYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYSINCkNvbnRlbnQt
TGFuZ3VhZ2U6IGVuLVVTDQoNClRoaXMgaXMgYSBtdWx0aS1wYXJ0IG1lc3NhZ2UgaW4gTUlNRSBm
b3JtYXQuDQotLWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWENCkNvbnRl
bnQtVHlwZTogdGV4dC9wbGFpbjsgY2hhcnNldD11dGYtODsgZm9ybWF0PWZsb3dlZA0KQ29udGVu
dC1UcmFuc2Zlci1FbmNvZGluZzogN2JpdA0KDQpUaGlzIGlzIGFuICpIVE1MKiB0ZXN0IG1lc3Nh
Z2UNCi0tYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYQ0KQ29udGVudC1U
eXBlOiB0ZXh0L2h0bWw7IGNoYXJzZXQ9dXRmLTgNCkNvbnRlbnQtVHJhbnNmZXItRW5jb2Rpbmc6
IDdiaXQNCg0KPGh0bWw+DQogIDxoZWFkPg0KICAgIDxtZXRhIGh0dHAtZXF1aXY9ImNvbnRlbnQt
dHlwZSIgY29udGVudD0idGV4dC9odG1sOyBjaGFyc2V0PVVURi04Ij4NCiAgPC9oZWFkPg0KICA8
Ym9keT4NCiAgICBUaGlzIGlzIGFuIDxiPkhUTUw8L2I+IHRlc3QgbWVzc2FnZQ0KICA8L2JvZHk+
DQo8L2h0bWw+DQoNCi0tYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYS0t
DQo=
--bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb--"#;
    let mail = convert_from_mime(test_mail).unwrap();
    assert_eq!(
        mail.0,
        "This is a message with a base64 encoded attached email".to_owned()
    );
    assert_eq!(mail.1.len(), 1);
    assert_eq!(mail.1[0].filename, "attached_email.eml");
    assert_eq!(
        String::from_utf8(mail.1[0].content.clone()).unwrap(),
        "To: Name <email@example.com>\r\nFrom: Name <email@example.com>\r\nSubject: HTML test\r\nMessage-ID: <random-message-id@example.com>\r\nDate: Tue, 14 Dec 2021 11:48:25 +0100\r\nMIME-Version: 1.0\r\nContent-Type: multipart/alternative;\r\n boundary=\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"\r\nContent-Language: en-US\r\n\r\nThis is a multi-part message in MIME format.\r\n--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\r\nContent-Type: text/plain; charset=utf-8; format=flowed\r\nContent-Transfer-Encoding: 7bit\r\n\r\nThis is an *HTML* test message\r\n--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\r\nContent-Type: text/html; charset=utf-8\r\nContent-Transfer-Encoding: 7bit\r\n\r\n<html>\r\n  <head>\r\n    <meta http-equiv=\"content-type\" content=\"text/html; charset=UTF-8\">\r\n  </head>\r\n  <body>\r\n    This is an <b>HTML</b> test message\r\n  </body>\r\n</html>\r\n\r\n--aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa--\r\n"
    );
    assert_eq!(mail.1[0].mimetype, "message/rfc822");
}
