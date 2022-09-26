use anyhow::Result;
use reqwest::{
    header::{self, HeaderMap, HeaderName, HeaderValue},
    Client, Method, Response,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Write;
use std::str::FromStr;
use url::Url;

use crate::{config::ResponseProfile, ExtraArgs};

#[derive(Debug)]
pub struct ResponseExt(Response);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestProfile {
    #[serde(with = "http_serde::method", default)]
    pub method: Method,
    pub url: Url,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub params: Option<serde_json::Value>,
    #[serde(
        skip_serializing_if = "HeaderMap::is_empty",
        with = "http_serde::header_map"
    )]
    pub headers: HeaderMap,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub body: Option<serde_json::Value>,
}

impl RequestProfile {
    pub async fn send(&self, args: &super::ExtraArgs) -> Result<ResponseExt> {
        let req = Client::new().request(self.method.clone(), self.url.clone());

        let (headers, query, body) = self.generate(args)?;

        let res = req.headers(headers).query(&query).body(body).send().await?;

        Ok(ResponseExt(res))
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

        match content_type {
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

impl ResponseExt {
    pub async fn filter_text(&self, profile: &ResponseProfile) -> Result<String> {
        let mut output = String::new();
        let headers = self.0.headers();

        for (k, v) in headers.iter() {
            if !profile.skip_headers.contains(&k.to_string()) {
                write!(output, "{}:{:?}\r", k, v)?;
            }
        }

        let content_type = get_content_type(&headers);
        let text = self.0.text().await?;

        match content_type {
            Some("application/json") => {
                let text = self.filter_json(&text, profile.skip_body)?;
                output.push_str(&text);
            }
            _ => output.push_str(&text),
        }

        todo!()
    }

    fn filter_json(&self, text: &str, skip: Vec<String>) -> Result<String> {
        let mut json: serde_json::Value = serde_json::from_str(text)?;
        for k in skip {
            json[k] = json!(null);
        }
        Ok(serde_json::to_string_pretty(&json)?)
    }
}

fn get_content_type(headers: &HeaderMap) -> Option<&str> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().unwrap().split(';').next());
    content_type
}
