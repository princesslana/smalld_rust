use serde_json::json;
use smalld::{Error, SmallD, SmallDBuilder};

const DUMMY_TOKEN: &str = "DuMmY.ToKeN";
const HTTP_BIN: &str = "http://httpbin.org";

fn subject() -> SmallD {
    SmallDBuilder::new()
        .token(DUMMY_TOKEN)
        .base_url(HTTP_BIN)
        .build()
        .unwrap()
}

fn http_bin(path: &str) -> String {
    format!("{}{}", HTTP_BIN, path)
}

fn assert_strlike_eq<A: AsRef<str>, B: AsRef<str>>(lhs: A, rhs: B) {
    assert_eq!(lhs.as_ref(), rhs.as_ref());
}

#[test]
fn it_makes_get_request() {
    let rsp = subject().resource("/get").get().unwrap();
    assert_strlike_eq(rsp["url"].as_str().unwrap(), http_bin("/get"));
}

#[test]
fn it_makes_get_request_with_parameters() {
    let rsp = subject()
        .resource("/get")
        .query("key1", "value1")
        .query("key2", "value2")
        .get()
        .unwrap();

    assert_strlike_eq(
        rsp["url"].as_str().unwrap(),
        http_bin("/get?key1=value1&key2=value2"),
    );
}

#[test]
fn it_makes_post_request() {
    let json = json!({"foo": "bar"});
    let rsp = subject()
        .resource("/post")
        .post(json!({"foo": "bar"}))
        .unwrap();
    assert_strlike_eq(rsp["url"].as_str().unwrap(), http_bin("/post"));
    assert_eq!(rsp["json"], json);
}

#[test]
fn it_makes_put_request() {
    let json = json!({"foo": "bar"});
    let rsp = subject()
        .resource("/put")
        .put(json!({"foo": "bar"}))
        .unwrap();
    assert_strlike_eq(rsp["url"].as_str().unwrap(), http_bin("/put"));
    assert_eq!(rsp["json"], json);
}

#[test]
fn it_makes_patch_request() {
    let json = json!({"foo": "bar"});
    let rsp = subject()
        .resource("/patch")
        .patch(json!({"foo": "bar"}))
        .unwrap();
    assert_strlike_eq(rsp["url"].as_str().unwrap(), http_bin("/patch"));
    assert_eq!(rsp["json"], json);
}

#[test]
fn it_makes_delete_request() {
    let rsp = subject().resource("/delete").delete().unwrap();
    assert_strlike_eq(rsp["url"].as_str().unwrap(), http_bin("/delete"));
}

#[test]
fn it_sends_user_agent() {
    let rsp = subject().resource("/user-agent").get().unwrap();
    assert_strlike_eq(
        rsp["user-agent"].as_str().unwrap(),
        "DiscordBot (https://github.com/princesslana/smalld_rust, 0.0.0-dev)",
    );
}

#[test]
fn it_sends_auth_header() {
    let rsp = subject().resource("/headers").get().unwrap();
    assert_strlike_eq(
        rsp["headers"]["Authorization"].as_str().unwrap(),
        format!("Bot {}", DUMMY_TOKEN),
    );
}

#[test]
fn it_handles_204_response() {
    let rsp = subject().resource("/status/204").delete().unwrap();
    assert_eq!(rsp, json!({}));
}

#[test]
fn it_errors_on_404_response() {
    let rsp = subject().resource("/status/404").get();
    assert!(matches!(rsp, Err(Error::HttpError(_))));
}
