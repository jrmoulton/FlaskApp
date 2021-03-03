#![feature(proc_macro_hygiene, decl_macro)]
#[allow(unused_variables)]
#[macro_use]
extern crate rocket;
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CONNECTION, CONTENT_LENGTH,
    CONTENT_TYPE, HOST, ORIGIN, REFERER, USER_AGENT,
};
use reqwest::{Client, ClientBuilder};
use rocket::request::Form;
extern crate http;
use scraper::{Html, Selector};
use std::time::Duration;

#[derive(FromForm, Debug)]
struct PunchForm {
    username: String,
    password: String,
    inout: String,
}

async fn parse_id(r_text: &str, id: &str) -> Result<String, reqwest::Error> {
    let selector = format!("{}{}", "#", id);
    let html = Html::parse_document(&r_text);
    let val = html
        .select(&Selector::parse(&selector).unwrap())
        .next()
        .unwrap()
        .value()
        .attr("value")
        .unwrap();
    Ok(String::from(val))
}

async fn login(
    username: String,
    password: String,
    client: &Client,
) -> Result<String, reqwest::Error> {
    let r_get_login = client
        .get("https://aggietime.usu.edu/login/auth")
        .send()
        .await?;
    let mut headers = construct_headers(r_get_login.url().to_string());
    let r_get_login_text = r_get_login.text().await?;
    let sync_token = parse_id(&r_get_login_text, "SYNCHRONIZER_TOKEN").await?;
    let sync_uri = parse_id(&r_get_login_text, "SYNCHRONIZER_URI").await?;
    let post_data = format!(
        "SYNCHRONIZER_TOKEN={}&SYNCHRONIZER_URI={}&j_username={}&j_password={}&login-submit=",
        sync_token, sync_uri, username, password
    );
    headers.append(
        CONTENT_LENGTH,
        HeaderValue::from_str(&post_data.len().to_string()).unwrap(),
    );
    let r_p_login = client
        .post("https://aggietime.usu.edu/j_spring_security_check")
        .body(post_data)
        .send()
        .await?;
    println!("{}", sync_token);
    println!("{}", sync_uri);
    println!("{}", r_p_login.status());
    println!("{}", r_p_login.url());
    Ok(r_p_login.url().to_string())
}

fn construct_headers(r_get_url: String) -> HeaderMap {
    // TODO define length
    // length = str(len(
    // f"SYNCHRONIZER_TOKEN={variables['sync_token']}&SYNCHRONIZER_URI={variables['sync_uri']}&posId={variables['posId']}&comment=&projectName=&toStatus={inout}"))
    let mut headers = HeaderMap::new();
    headers.insert(HOST, HeaderValue::from_static("aggietime.usu.edu"));
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );
    headers.insert(
        ORIGIN,
        HeaderValue::from_static("https://aggietime.usu.edu"),
    );
    headers.insert(
        ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br"),
    );
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.2 Safari/605.1.15"));
    headers.insert(REFERER, HeaderValue::from_str(&r_get_url).unwrap());
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-us"));
    headers
}

#[get("/aggietime/punch")]
fn index() -> &'static str {
    "Please send a post request with the form values to log in"
}

#[post("/aggietime/punch", data = "<punchform>")]
async fn punch_clock(punchform: Form<PunchForm>) -> String {
    let timeout = Duration::new(5, 0);
    let client = ClientBuilder::new().timeout(timeout).build();
    let client = match client {
        Ok(client) => client,
        Err(error) => panic!(error),
    };
    let url = login(
        punchform.username.clone(),
        punchform.password.clone(),
        &client,
    )
    .await
    .unwrap();
    let _response = client.post("https://aggietime.usu.edu/dashboard/clock/punch")
    .headers(construct_headers(url))
    .body("SYNCHRONIZER_TOKEN={sync_token}&SYNCHRONIZER_URI={sync_uri}&j_username={username}&j_password={password}&login-submit=")
    .send();
    "HI".to_string();
    todo!()
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}

#[tokio::main]
#[test]
async fn test_login() {
    let timeout = Duration::new(5, 0);
    let client = ClientBuilder::new().timeout(timeout).build();
    let client = match client {
        Ok(client) => client,
        Err(error) => panic!(error),
    };
    let status_url = login(
        String::from("A02226665"),
        String::from("***REMOVED***"),
        &client,
    )
    .await
    .unwrap();
    assert!(status_url.contains("dashboard"));
}
