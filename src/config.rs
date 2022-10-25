use crate::utils::diff_text;
use crate::{req::RequestProfile, ExtraArgs};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skip_body: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffProfile {
    pub request1: RequestProfile,
    pub request2: RequestProfile,
    pub response: ResponseProfile,
}

impl DiffProfile {
    pub async fn diff(&self, args: ExtraArgs) -> Result<String> {
        let res1 = self.request1.send(&args).await?;
        let res2 = self.request2.send(&args).await?;

        let text1 = res1.filter_text(&self.response).await?;
        let text2 = res2.filter_text(&self.response).await?;
        let diff = diff_text(&text1, &text2);

        println!("text1 is {text1}");
        println!("text2 is {text2}");

        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffConfig {
    #[serde(flatten)]
    pub profiles: HashMap<String, DiffProfile>,
}

impl DiffConfig {
    pub async fn load_yaml(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path).await?;
        Self::from_yaml(&content)
    }
    pub fn from_yaml(context: &str) -> anyhow::Result<Self> {
        Ok(serde_yaml::from_str(context)?)
    }

    pub fn get_profile(&self, name: &str) -> Option<&DiffProfile> {
        self.profiles.get(name)
    }
}
