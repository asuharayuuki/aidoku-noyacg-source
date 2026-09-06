use aidoku::{
    Result,
    alloc::{String, format, string::ToString},
    imports::{
        defaults::{DefaultValue, defaults_get, defaults_set},
        net::{HttpMethod, Request},
    },
    serde::de::DeserializeOwned,
};
use serde_json::Value;

use crate::{USER_AGENT, WEB_URL};

const SESSION_KEY: &str = "session";
const USERNAME_KEY: &str = "username";
const PASSWORD_KEY: &str = "password";
const JUST_LOGGED_IN_KEY: &str = "justLoggedIn";

fn base_request(url: &str, method: HttpMethod, session: &str) -> Result<Request> {
    Ok(Request::new(url, method)?
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("allow-adult", "false")
        .header("User-Agent", USER_AGENT)
        .header("Referer", WEB_URL)
        .header("Cookie", &format!("NOY_SESSION={session}")))
}

pub fn set_credentials(username: &str, password: &str) {
    defaults_set(USERNAME_KEY, DefaultValue::String(username.to_string()));
    defaults_set(PASSWORD_KEY, DefaultValue::String(password.to_string()));
}

fn credentials() -> Option<(String, String)> {
    let username = defaults_get::<String>(USERNAME_KEY)?;
    let password = defaults_get::<String>(PASSWORD_KEY)?;
    if username.is_empty() || password.is_empty() {
        None
    } else {
        Some((username, password))
    }
}

pub fn clear_login() {
    for key in [SESSION_KEY, USERNAME_KEY, PASSWORD_KEY, JUST_LOGGED_IN_KEY] {
        defaults_set(key, DefaultValue::Null);
    }
}

pub fn mark_just_logged_in() {
    defaults_set(JUST_LOGGED_IN_KEY, DefaultValue::Bool(true));
}

pub fn consume_just_logged_in() -> bool {
    let value = defaults_get::<bool>(JUST_LOGGED_IN_KEY).unwrap_or(false);
    if value {
        defaults_set(JUST_LOGGED_IN_KEY, DefaultValue::Null);
    }
    value
}

pub fn login(username: &str, password: &str) -> Result<bool> {
    let url = format!("{WEB_URL}/api/login");
    let body = format!(
        "user={}&pass={}",
        aidoku::helpers::uri::encode_uri_component(username),
        aidoku::helpers::uri::encode_uri_component(password)
    );
    let response = base_request(&url, HttpMethod::Post, "")?
        .body(body.as_bytes())
        .send()?;
    if response.status_code() != 200 {
        return Ok(false);
    }
    let cookie = response.get_header("set-cookie").unwrap_or_default();
    let session = cookie
        .split("NOY_SESSION=")
        .nth(1)
        .and_then(|value| value.split(';').next())
        .unwrap_or("");
    if session.is_empty() {
        return Ok(false);
    }
    defaults_set(SESSION_KEY, DefaultValue::String(session.to_string()));
    Ok(true)
}

fn session() -> Result<String> {
    if let Some(session) = defaults_get::<String>(SESSION_KEY).filter(|value| !value.is_empty()) {
        return Ok(session);
    }
    let (username, password) =
        credentials().ok_or_else(|| aidoku::error!("请先在图源设置中登录"))?;
    if login(&username, &password)? {
        return defaults_get::<String>(SESSION_KEY).ok_or_else(|| aidoku::error!("登录会话缺失"));
    }
    Err(aidoku::error!("NoyAcg 登录失败"))
}

fn is_login_response(value: &Value) -> bool {
    value.get("status").and_then(Value::as_str) == Some("login")
        || value
            .as_array()
            .and_then(|values| values.first())
            .and_then(Value::as_str)
            == Some("login")
}

fn request_value(path: &str, body: Option<&str>, retry: bool) -> Result<Value> {
    let url = format!("{WEB_URL}/api{path}");
    let session = session()?;
    let request = base_request(
        &url,
        if body.is_some() {
            HttpMethod::Post
        } else {
            HttpMethod::Get
        },
        &session,
    )?;
    let value: Value = match body {
        Some(body) => request.body(body.as_bytes()).json_owned()?,
        None => request.json_owned()?,
    };
    if retry && is_login_response(&value) {
        defaults_set(SESSION_KEY, DefaultValue::Null);
        return request_value(path, body, false);
    }
    Ok(value)
}

pub fn get<T: DeserializeOwned>(path: &str) -> Result<T> {
    Ok(serde_json::from_value(request_value(path, None, true)?)?)
}

pub fn post<T: DeserializeOwned>(path: &str, body: &str) -> Result<T> {
    Ok(serde_json::from_value(request_value(
        path,
        Some(body),
        true,
    )?)?)
}
