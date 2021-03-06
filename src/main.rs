#![feature(proc_macro_hygiene, decl_macro)]
#![allow(unused_variables)]
#[macro_use]
extern crate rocket;
use futures::future::{join_all, try_join_all};
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CONNECTION, CONTENT_LENGTH,
    CONTENT_TYPE, HOST, ORIGIN, REFERER, USER_AGENT,
};
use reqwest::{Client, ClientBuilder};
use rocket::request::Form;
use rocket::State;
use scraper::{Html, Selector};
use std::time::Duration;

#[derive(FromForm, Debug)]
struct PunchForm {
    username: String,
    password: String,
    inout: String,
}
#[derive(FromForm, Debug)]
struct StatusForm {
    username: String,
    password: String,
}
#[derive(FromForm, Debug)]
struct UpdateForm {
    username: String,
    password: String,
    time_in: Option<String>,
    time_out: Option<String>,
    edit_comment: Option<String>,
    reload: Option<bool>,
}

struct Session {
    client: Client,
    logged_in: bool,
}
#[derive(PartialEq, Eq)]
enum ClockStatus {
    IN,
    OUT,
}

#[launch]
async fn rocket_main() -> _ {
    rocket::ignite()
        .mount(
            "/",
            routes![punch_clock, punch_clock_get, status, update_shift],
        )
        .manage(Session {
            client: new_client(),
            logged_in: false,
        })
}

#[get("/aggietime/punch")]
fn punch_clock_get() -> &'static str {
    "Please send a post request with the form values to log in"
}
#[post("/aggietime/punch", data = "<punchform>")]
async fn punch_clock(punchform: Form<PunchForm>, state: State<'_, Session>) -> String {
    let client = &state.client;
    let r_login = login(&punchform.username, &punchform.password, client, &state).await;
    let r_login = match r_login {
        Ok(r_login) => r_login,
        Err(error_msg) => return error_msg,
    };
    let r_login_url = r_login.url().to_string();
    let vals = get_sync_values(
        r_login,
        vec!["SYNCHRONIZER_TOKEN", "SYNCHRONIZER_URI", "posId"],
    )
    .await;
    let r_p_punch = client
        .post("https://aggietime.usu.edu/dashboard/clock/punch")
        .headers(construct_headers(r_login_url))
        .body(format!(
            "SYNCHRONIZER_TOKEN={}&SYNCHRONIZER_URI={}&posId={}&comment=&projectName=&toStatus={}",
            vals[0], vals[1], vals[2], punchform.inout
        ))
        .send()
        .await;
    let result = match r_p_punch {
        Ok(_) => String::from(format!(
            "Success clocking {}!",
            punchform.inout.to_ascii_lowercase()
        )),
        Err(_) => String::from(format!(
            "Failed to clocked {}.",
            punchform.inout.to_ascii_lowercase()
        )),
    };
    result
}

#[post("/aggietime/status", data = "<statusform>")]
async fn status(statusform: Form<StatusForm>, state: State<'_, Session>) -> String {
    let client = &state.client;
    let r_login = login(&statusform.username, &statusform.password, client, &state).await;
    let r_login = match r_login {
        Ok(r_login) => r_login,
        Err(_) => return String::from("Failure logging in"),
    };
    let r_login_text = r_login.text().await.unwrap();
    let status = get_status(client, &r_login_text).await;
    if status == ClockStatus::IN {
        String::from("You are clocked in")
    } else {
        String::from("You are clocked out")
    }
}
async fn get_status(client: &Client, r_text: &str) -> ClockStatus {
    let to_status = parse_attr(&r_text, "input", "id", "toStatus", "value")
        .await
        .unwrap();
    if to_status == "OUT" {
        ClockStatus::IN
    } else {
        ClockStatus::OUT
    }
}

async fn get_dashboard(client: &Client) -> Result<reqwest::Response, String> {
    let r_get_login = client
        .get("https://aggietime.usu.edu/dashboard")
        .send()
        .await;
    let r_get_login = match r_get_login {
        Ok(r) => return Ok(r),
        Err(_) => return Err(String::from("Failed to get the dashboard")),
    };
}

/*#[post("/aggietime/get")]
async fn get_shift()*/

#[post("/aggietime/update", data = "<updateform>")]
async fn update_shift(updateform: Form<UpdateForm>, state: State<'_, Session>) -> String {
    let client = &state.client;
    let r_login: reqwest::Response;
    let r_login = login(&updateform.username, &updateform.password, client, &state).await;
    let r_login = match r_login {
        Ok(r_login) => r_login,
        Err(error_msg) => return error_msg,
    };
    let text = r_login.text().await.unwrap();
    let mut futures = Vec::new();
    futures.push(parse_attr(&text, "input", "name", "id", "value")); // shift_id
    futures.push(parse_attr(
        &text,
        "span",
        "class",
        "in-date smaller",
        "inner",
    )); // date
    futures.push(parse_attr(&text, "span", "class", "in-time bold", "inner")); // time_in
    futures.push(parse_attr(&text, "span", "class", "out-time bold", "inner")); // time_out
    let elements = join_all(futures).await;
    let shift_id = match &elements[0] {
        Ok(shift_id) => shift_id,
        Err(error_msg) => return String::from(error_msg),
    };
    let event_date = match &elements[1] {
        Ok(event_date) => event_date,
        Err(error_msg) => return String::from(error_msg),
    };
    let time_in = match &elements[2] {
        Ok(time_in) => time_in,
        Err(error_msg) => return String::from(error_msg),
    };
    let time_out = match &elements[3] {
        Ok(time_out) => time_out,
        Err(error_msg) => return String::from(error_msg),
    };
    format!("{}: {}: {}: {}", event_date, shift_id, time_in, time_out)
}

async fn login(
    username: &str,
    password: &str,
    client: &Client,
    state: &State<'_, Session>,
) -> Result<reqwest::Response, String> {
    if !state.logged_in {
        let r_get_login = client
            .get("https://aggietime.usu.edu/login/auth")
            .send()
            .await;
        let r_get_login = match r_get_login {
            Ok(r) => r,
            Err(_) => return Err(String::from("Failed to get the initial login page")),
        };
        let mut headers = construct_headers(r_get_login.url().to_string());
        let vals =
            get_sync_values(r_get_login, vec!["SYNCHRONIZER_TOKEN", "SYNCHRONIZER_URI"]).await;
        let post_data = format!(
            "SYNCHRONIZER_TOKEN={}&SYNCHRONIZER_URI={}&j_username={}&j_password={}&login-submit=",
            vals[0], vals[1], username, password
        );
        headers.append(
            CONTENT_LENGTH,
            HeaderValue::from_str(&post_data.len().to_string()).unwrap(),
        );
        let r_p_login = client
            .post("https://aggietime.usu.edu/j_spring_security_check")
            .body(post_data)
            .headers(headers)
            .send()
            .await
            .unwrap();
        if r_p_login.url().to_string().contains("dashboard") {
            Ok(r_p_login)
        } else {
            Err(String::from("Failed to log in"))
        }
    } else {
        Ok(get_dashboard(client).await?)
    }
}

fn new_client() -> Client {
    let timeout = Duration::new(5, 0);
    let client = ClientBuilder::new()
        .timeout(timeout)
        .cookie_store(true)
        .build();
    let client = match client {
        Ok(client) => client,
        Err(error) => panic!(error),
    };
    client
}

async fn get_sync_values(r: reqwest::Response, ids: Vec<&str>) -> Vec<String> {
    let r_text = r.text().await.unwrap();
    let futures = ids
        .iter()
        .map(|id| parse_attr(&r_text, "input", "id", id, "value"));
    try_join_all(futures).await.unwrap()
}

async fn parse_attr(
    r_text: &str,
    tag: &str,
    id_type: &str,
    id: &str,
    attr: &str,
) -> Result<String, String> {
    let selector = format!(r#"{}[{}="{}"]"#, tag, id_type, id);
    let html = Html::parse_document(&r_text);
    let val = Selector::parse(&selector);
    let val = match val {
        Ok(val) => html.select(&val).next(),
        Err(_) => return Err(String::from("Failed to parse the selector")),
    };
    if attr != "inner" {
        let val = match val {
            Some(val) => val.value().attr(attr),
            None => return Err(format!("Nothing matched that selector: {}", &selector)),
        };
        let val = match val {
            Some(val) => val,
            None => {
                return Err(format!(
                    "The element found did not have the attribute {}",
                    attr
                ))
            }
        };
        Ok(val.to_string())
    } else {
        let val = match val {
            Some(val) => val.inner_html().split_whitespace().collect(),
            None => return Err(format!("Nothing matched that selector: {}", &selector)),
        };
        Ok(val)
    }
}

fn construct_headers(r_get_url: String) -> HeaderMap {
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

#[cfg(test)]
mod tests {
    /*use super::*;
    #[tokio::test]
    async fn test_login() {
        let client = new_client();
        let status = login("A02226665", "***REMOVED***", &client)
            .await
            .unwrap();
        assert!(status.url().to_string().contains("dashboard"));
    }*/
}
