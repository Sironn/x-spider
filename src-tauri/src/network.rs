use std::collections::HashMap;
use std::process::Command;

use reqwest::Method;
use serde_json::Value;

#[derive(Default, serde::Serialize)]
pub struct Response {
  status: u16,
  headers: HashMap<String, Vec<String>>,
  body: Value,
}

fn normalize_proxy_address(value: &str) -> String {
  let value = value.trim();

  for scheme in ["http://", "https://", "socks5://", "socks5h://"] {
    if let Some(stripped) = value.strip_prefix(scheme) {
      return stripped.trim_end_matches('/').to_string();
    }
  }

  value.trim_end_matches('/').to_string()
}

fn to_proxy_url(value: &str) -> String {
  if value.contains("://") {
    value.to_string()
  } else {
    format!("http://{}", value)
  }
}

fn insert_proxy(
  proxies: &mut HashMap<String, String>,
  scheme: &str,
  host: &str,
  port: Option<&str>,
) {
  if host.trim().is_empty() {
    return;
  }

  let mut address = normalize_proxy_address(host);
  if let Some(port) = port.map(str::trim).filter(|p| !p.is_empty()) {
    if !address.rsplit_once(':')
      .map(|(_, existing)| existing == port)
      .unwrap_or(false) {
      address = format!("{}:{}", address, port);
    }
  }

  if !address.is_empty() {
    proxies.insert(scheme.to_string(), address);
  }
}

#[cfg(windows)]
fn get_system_proxy_url() -> Result<HashMap<String, String>, String> {
  let key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings";
  let output = Command::new("reg")
    .args(["query", key])
    .output()
    .map_err(|err| format!("Failed to query Windows proxy settings: {}", err))?;

  if !output.status.success() {
    return Err("Windows proxy query failed".to_string());
  }

  let stdout = String::from_utf8_lossy(&output.stdout);
  let mut enabled = false;
  let mut proxy_server = String::new();

  for line in stdout.lines() {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 {
      continue;
    }

    match parts[0] {
      "ProxyEnable" => {
        enabled = parts[2]
          .trim_start_matches("0x")
          .parse::<u32>()
          .map(|value| value != 0)
          .unwrap_or(false);
      }
      "ProxyServer" => {
        proxy_server = parts[2..].join(" ");
      }
      _ => {}
    }
  }

  if !enabled || proxy_server.is_empty() {
    return Ok(HashMap::new());
  }

  let mut proxies = HashMap::new();
  for entry in proxy_server.split(';') {
    if let Some((scheme, address)) = entry.split_once('=') {
      match scheme.to_ascii_lowercase().as_str() {
        "http" | "https" => insert_proxy(&mut proxies, scheme, address, None),
        _ => {}
      }
    } else {
      insert_proxy(&mut proxies, "http", entry, None);
      insert_proxy(&mut proxies, "https", entry, None);
    }
  }

  Ok(proxies)
}

#[cfg(target_os = "macos")]
fn get_system_proxy_url() -> Result<HashMap<String, String>, String> {
  let output = Command::new("/usr/sbin/scutil")
    .args(["--proxy"])
    .output()
    .map_err(|err| format!("Failed to query macOS proxy settings: {}", err))?;

  if !output.status.success() {
    return Err("macOS proxy query failed".to_string());
  }

  let stdout = String::from_utf8_lossy(&output.stdout);
  let mut values = HashMap::<String, String>::new();

  for line in stdout.lines() {
    let Some((key, value)) = line.split_once(':') else {
      continue;
    };
    values.insert(key.trim().to_string(), value.trim().to_string());
  }

  let mut proxies = HashMap::new();

  if values.get("HTTPEnable").map(String::as_str) == Some("1") {
    insert_proxy(
      &mut proxies,
      "http",
      values.get("HTTPProxy").map(String::as_str).unwrap_or(""),
      values.get("HTTPPort").map(String::as_str),
    );
  }

  if values.get("HTTPSEnable").map(String::as_str) == Some("1") {
    insert_proxy(
      &mut proxies,
      "https",
      values.get("HTTPSProxy").map(String::as_str).unwrap_or(""),
      values.get("HTTPSPort").map(String::as_str),
    );
  }

  Ok(proxies)
}

#[cfg(not(any(windows, target_os = "macos")))]
fn get_system_proxy_url() -> Result<HashMap<String, String>, String> {
  let mut proxies = HashMap::new();

  for key in [
    "HTTPS_PROXY",
    "https_proxy",
    "HTTP_PROXY",
    "http_proxy",
    "ALL_PROXY",
    "all_proxy",
  ] {
    if let Ok(value) = std::env::var(key) {
      if !value.trim().is_empty() {
        insert_proxy(&mut proxies, "http", &value, None);
        insert_proxy(&mut proxies, "https", &value, None);
        break;
      }
    }
  }

  Ok(proxies)
}

#[tauri::command]
pub async fn network_fetch(
  method: String,
  url: String,
  body: String,
  enable_proxy: bool,
  proxy_url: String,
  response_type: String,
  headers: HashMap<String, String>,
) -> Result<Response, String> {
  let map_reqwest_err = |err: reqwest::Error| err.to_string();

  let method: Method = match method.to_uppercase().as_str() {
    "GET" => Ok(Method::GET),
    "POST" => Ok(Method::POST),
    "PATCH" => Ok(Method::PATCH),
    "PUT" => Ok(Method::PUT),
    "DELETE" => Ok(Method::DELETE),
    "HEAD" => Ok(Method::HEAD),
    _ => Err("Invalid method".to_string()),
  }?;

  let client = {
    let mut b = reqwest::Client::builder();

    if enable_proxy {
      if proxy_url.is_empty() {
        if let Ok(proxies) = get_system_proxy_url() {
          let http_proxy = proxies
            .get("http")
            .map(|value| to_proxy_url(value));
          let https_proxy = proxies
            .get("https")
            .or_else(|| proxies.get("http"))
            .map(|value| to_proxy_url(value));

          if let Some(proxy) = http_proxy {
            b = b.proxy(
              reqwest::Proxy::http(proxy)
                .map_err(|_| "Failed to set system HTTP proxy".to_string())?,
            );
          }

          if let Some(proxy) = https_proxy {
            b = b.proxy(
              reqwest::Proxy::https(proxy)
                .map_err(|_| "Failed to set system HTTPS proxy".to_string())?,
            );
          }
        }
      } else {
        let proxy_http = reqwest::Proxy::http(proxy_url.clone())
          .map_err(|_| "Failed to set proxy url".to_string())?;
        let proxy_https = reqwest::Proxy::https(proxy_url.clone())
          .map_err(|_| "Failed to set proxy url".to_string())?;
        b = b.proxy(proxy_http).proxy(proxy_https);
      }
    } else {
      b = b.no_proxy();
    }

    b.build()
      .map_err(|_| "Failed to build reqwest client".to_string())?
  };

  let request = {
    let mut req = client.request(method.clone(), url);
    for (k, v) in headers {
      req = req.header(k, v);
    }

    if !matches!(method, Method::GET) {
      req = req.body(body);
    }

    req
  };

  let response = request.send().await.map_err(map_reqwest_err)?;

  let status = response.status().as_u16();
  let resp_headers = {
    let reqwest_headers = response.headers();
    let mut h: HashMap<String, Vec<String>> = HashMap::with_capacity(reqwest_headers.len());

    for (k, v) in reqwest_headers {
      let v = v.to_str();
      if let Err(_) = v {
        continue;
      }

      let v = v.unwrap().to_string();
      h.entry(k.to_string())
        .and_modify(|arr: &mut Vec<String>| arr.push(v.clone()))
        .or_insert_with(|| vec![v]);
    }

    h
  };

  let body: Value = {
    match response_type.as_str() {
      "json" => response
        .json()
        .await
        .map_err(map_reqwest_err)
        .map(|res: serde_json::Map<String, Value>| Value::Object(res)),
      "text" => response
        .text()
        .await
        .map_err(map_reqwest_err)
        .map(Value::String),
      "binary" => {
        let bytes = response.bytes().await.map_err(map_reqwest_err)?;
        serde_json::to_value(bytes.to_vec()).map_err(|err| err.to_string())
      }
      _ => Err("Unsupported response type".to_string()),
    }
  }?;

  Ok(Response {
    status,
    body,
    headers: resp_headers,
  })
}

#[tauri::command]
pub async fn network_get_system_proxy_url() -> Result<HashMap<String, String>, String> {
  get_system_proxy_url()
}
