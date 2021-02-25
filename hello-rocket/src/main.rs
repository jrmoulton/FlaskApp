#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use std::time::Duration;
use reqwest::ClientBuilder;
use reqwest::Result;
use reqwest::header::{HeaderMap, HeaderValue, HOST, CONTENT_TYPE, ORIGIN, 
    ACCEPT_ENCODING, CONNECTION, ACCEPT, USER_AGENT,  REFERER, CONTENT_LENGTH, 
    ACCEPT_LANGUAGE};
use rocket::request::Form;

#[derive(FromForm)]
struct PunchForm {
    username: String,
    password: String,
    inout: String,
}

fn login(username: String, password: String, client: Client) {
    r_login = client.get("https://aggietime.usu.edu/login/auth");
}

fn construct_headers() -> header::HeaderMap {
    // TODO define length 
    // length = str(len(
    // f"SYNCHRONIZER_TOKEN={variables['sync_token']}&SYNCHRONIZER_URI={variables['sync_uri']}&posId={variables['posId']}&comment=&projectName=&toStatus={inout}"))
    let mut headers = header::HeaderMap::new();
    headers.insert(HOST, HeaderValue::from_static("aggietime.usu.edu"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));
    headers.insert(ORIGIN, HeaderValue::from_static("https://aggietime.usu.edu"));
    headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br"));
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"));
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.2 Safari/605.1.15"));
    headers.insert(REFERER, HeaderValue::from_static("https://aggietime.usu.edu/dashboard"));
    headers.insert(CONTENT_LENGTH, HeaderValue::from_static(length));
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-us"));
}

#[get("/aggietime/punch")]
fn index() -> &'static str {
    "Please send a post request with the form values to log in"
}

#[post("/aggietime/punch", data = "<punchform>")]
fn punch_clock(punchform: Form<PunchForm>) -> String {
    let timeout = Duration::new(5, 0);
    let client = ClientBuilder::new().timeout(timeout).build()?;
    let response = client.post("https://aggietime.usu.edu/dashboard/clock/punch")
    .heaaders(construct_headers())
    .body("SYNCHRONIZER_TOKEN={sync_token}&SYNCHRONIZER_URI={sync_uri}&j_username={username}&j_password={password}&login-submit=")
    .send()
    .await()?;
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}
