use crate::ExtraArgs;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Method, Response, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Write;
use std::str::FromStr;
use tokio::fs;

pub mod xdiff;
pub mod xreq;
pub use xdiff::*;
pub use xreq::*;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skip_body: Vec<String>,
}

#[derive(Debug)]
pub struct ResponseExt(Response);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestProfile {
    #[serde(with = "http_serde::method", default)]
    pub method: Method,
    pub url: Url,
    #[serde(skip_serializing_if = "empty_json_value", default)]
    pub params: Option<serde_json::Value>,
    #[serde(
        skip_serializing_if = "HeaderMap::is_empty",
        with = "http_serde::header_map",
        default
    )]
    pub headers: HeaderMap,
    #[serde(skip_serializing_if = "empty_json_value", default)]
    pub body: Option<serde_json::Value>,
}

impl RequestProfile {
    pub fn validate(&self) -> Result<()> {
        if let Some(params) = self.params.as_ref() {
            if !params.is_object() {
                return Err(anyhow::anyhow!(
                    "Params must be an object but got \n{}",
                    serde_yaml::to_string(params)?
                ));
            }
        }
        if let Some(body) = self.body.as_ref() {
            if !body.is_object() {
                return Err(anyhow::anyhow!(
                    "body must be an object but got \n{}",
                    serde_yaml::to_string(body)?
                ));
            }
        }
        Ok(())
    }
    pub async fn send(&self, args: &super::ExtraArgs) -> Result<ResponseExt> {
        let req = Client::new().request(self.method.clone(), self.url.clone());

        let (headers, query, body) = self.generate(args)?;

        let res = req.headers(headers).query(&query).body(body).send().await?;

        Ok(ResponseExt(res))
    }

    pub fn get_url(&self, args: &ExtraArgs) -> Result<String> {
        let mut url = self.url.clone();
        let (_, params, _) = self.generate(args)?;

        if !params.as_object().unwrap().is_empty() {
            let query = serde_qs::to_string(&params)?;
            url.set_query(Some(&query));
        }
        Ok(url.to_string())
    }

    pub fn generate(&self, args: &ExtraArgs) -> Result<(HeaderMap, serde_json::Value, String)> {
        let mut headers = self.headers.clone();
        let mut body = self.body.clone().unwrap_or_else(|| json!({}));
        let mut query = self.params.clone().unwrap_or_else(|| json!({}));

        for (k, v) in &args.headers {
            headers.insert(HeaderName::from_str(k)?, HeaderValue::from_str(v)?);
        }

        if !headers.contains_key(header::CONTENT_TYPE) {
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
        }

        for (k, v) in &args.query {
            query[k] = v.parse()?;
        }

        for (k, v) in &args.body {
            body[k] = v.parse()?;
        }

        let content_type = get_content_type(&headers);

        match content_type.as_deref() {
            Some("application/json") => {
                let body = serde_json::to_string(&body)?;
                Ok((headers, query, body))
            }
            Some("application/x-www-form-urlencoded" | "multipart/form-data") => {
                let body = serde_urlencoded::to_string(&body)?;
                Ok((headers, query, body))
            }
            _ => Ok((headers, query, body.to_string())),
        }

        // Ok((headers, query, body))
    }
}

impl FromStr for RequestProfile {
    type Err = anyhow::Error;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        let mut url = Url::parse(url)?;

        let qs = url.query_pairs();

        let mut params = json!({});
        for (k, v) in qs {
            params[&*k] = v.parse()?;
        }
        url.set_query(None);

        Ok(Self {
            method: Method::GET,
            url,
            params: Some(params),
            headers: HeaderMap::new(),
            body: None,
        })
    }
}

impl ResponseExt {
    pub async fn filter_text(self, profile: &ResponseProfile) -> Result<String> {
        fn filter_json(text: &str, skip: &Vec<String>) -> Result<String> {
            let json: serde_json::Value = serde_json::from_str(text)?;

            match json {
                serde_json::Value::Object(mut j) => {
                    for k in skip {
                        j[k] = json!(null);
                    }
                    Ok(serde_json::to_string_pretty(&j)?)
                }
                _ => Ok(String::new()),
            }
        }

        let mut output = String::new();
        write!(output, "{:?}{}\r", self.0.version(), self.0.status())?;
        let headers = self.0.headers();

        for (k, v) in headers.iter() {
            if !profile.skip_headers.contains(&k.to_string()) {
                write!(output, "{}:{:?}\r", k, v)?;
            }
        }

        let content_type = get_content_type(headers);
        let text = self.0.text().await?;

        match content_type.as_deref() {
            Some("application/json") => {
                let text = filter_json(&text, &profile.skip_body)?;
                output.push_str(&text);
            }
            _ => output.push_str(&text),
        }

        Ok(output)
    }

    pub fn get_headers_keys(&self) -> Vec<String> {
        self.0
            .headers()
            .iter()
            .map(|(k, _)| k.as_str().to_string())
            .collect()
    }
    pub fn into_inner(self) -> Response {
        self.0
    }
}

fn get_content_type(headers: &HeaderMap) -> Option<String> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().unwrap().split(';').next())
        .map(|x| x.to_string());
    content_type
}

fn empty_json_value(v: &Option<serde_json::Value>) -> bool {
    v.as_ref().map_or(true, |v| {
        v.is_null() || (v.is_object() && v.as_object().unwrap().is_empty())
    })
}

pub fn get_status_text(res: &Response) -> Result<String> {
    Ok(format!("{:?} {}\n", res.version(), res.status()))
}

pub fn get_header_text(res: &Response, skip_headers: &[String]) -> Result<String> {
    let mut output = String::new();

    let headers = res.headers();
    for (k, v) in headers.iter() {
        if !skip_headers.iter().any(|sh| sh == k.as_str()) {
            writeln!(&mut output, "{}: {:?}", k, v)?;
        }
    }
    writeln!(&mut output)?;

    Ok(output)
}

pub async fn get_body_text(res: Response, skip_body: &[String]) -> Result<String> {
    let content_type = get_content_type(res.headers());
    let text = res.text().await?;

    match content_type.as_deref() {
        Some("application/json") => filter_json(&text, skip_body),
        _ => Ok(text),
    }
}
fn filter_json(text: &str, skip: &[String]) -> Result<String> {
    let mut json: serde_json::Value = serde_json::from_str(text)?;

    // For now we just ignore non-object values, we don't know how to filter.
    // In future, we might support array of objects
    if let serde_json::Value::Object(ref mut obj) = json {
        for k in skip {
            obj.remove(k);
        }
    }

    Ok(serde_json::to_string_pretty(&json)?)
}

impl ResponseProfile {
    pub fn new(skip_headers: Vec<String>, skip_body: Vec<String>) -> Self {
        Self {
            skip_headers,
            skip_body,
        }
    }
}

#[async_trait]
pub trait LoadConfig
where
    Self: ValidateConfig + DeserializeOwned,
{
    async fn load_yaml(path: &str) -> Result<Self> {
        println!("{}", path);
        let content = fs::read_to_string(path).await?;
        Self::from_yaml(&content)
    }

    fn from_yaml(content: &str) -> Result<Self> {
        let config: Self = serde_yaml::from_str(content)?;
        config.validate()?;
        Ok(config)
    }
}

pub trait ValidateConfig {
    fn validate(&self) -> Result<()>;
}
