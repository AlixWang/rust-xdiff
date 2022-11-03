use anyhow::Result;
use reqwest::{
    header::{self, HeaderMap, HeaderName, HeaderValue},
    Client, Method, Response, Url,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Write;
use std::str::FromStr;

use crate::{config::ResponseProfile, ExtraArgs};

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
        println!("print the result {}", v);
        v.is_null() || (v.is_object() && v.as_object().unwrap().is_empty())
    })
}
