use crate::config::app_data_dir;
use serde::Deserialize;
use serde_json::Value;
use std::fs;

const API_BASE: &str = "https://oppw4.prism.am/api";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiRequest {
  method: String,
  path: String,
  body: Option<String>,
  token: Option<String>,
}

#[tauri::command]
pub fn api_request(input: ApiRequest) -> Result<Value, String> {
  let client = reqwest::blocking::Client::new();
  let method = input.method.parse().map_err(|err| format!("Invalid API method: {err}"))?;
  let url = if input.path.starts_with("http://") || input.path.starts_with("https://") {
    input.path
  } else {
    format!("{API_BASE}{}", input.path)
  };
  let mut request = client
    .request(method, url)
    .header("accept", "application/json")
    .header("user-agent", "oppw4-launcher");

  if let Ok(viewer_id) = launcher_viewer_id() {
    request = request.header("cookie", format!("oppw4_viewer={viewer_id}"));
  }
  if let Some(token) = input.token.filter(|value| !value.trim().is_empty()) {
    request = request.bearer_auth(token);
  }
  if let Some(body) = input.body {
    request = request.header("content-type", "application/json").body(body);
  }

  let response = request.send().map_err(|err| format!("API request failed: {err}"))?;
  let status = response.status();
  let text = response.text().map_err(|err| format!("Could not read API response: {err}"))?;
  let json = serde_json::from_str::<Value>(&text).unwrap_or_else(|_| serde_json::json!({ "error": text }));
  if !status.is_success() {
    let message = json
      .get("error")
      .and_then(Value::as_str)
      .unwrap_or("API request failed");
    return Err(message.to_string());
  }
  Ok(json)
}

fn launcher_viewer_id() -> Result<String, String> {
  let path = app_data_dir()?.join("viewer-id");
  if let Ok(existing) = fs::read_to_string(&path) {
    let existing = existing.trim();
    if !existing.is_empty() {
      return Ok(existing.to_string());
    }
  }
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|err| format!("Could not create app data directory: {err}"))?;
  }
  let value = format!("launcher-{}-{}", now_label(), std::process::id());
  fs::write(&path, &value).map_err(|err| format!("Could not write launcher viewer id: {err}"))?;
  Ok(value)
}

fn now_label() -> String {
  use std::time::{SystemTime, UNIX_EPOCH};
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_secs().to_string())
    .unwrap_or_else(|_| "0".to_string())
}
