// use crate::utils::diff_text;
use crate::ExtraArgs;
use crate::{req::RequestProfile, utils::diff_text};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;

pub fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    v == &T::default()
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skip_body: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffProfile {
    pub req1: RequestProfile,
    pub req2: RequestProfile,
    #[serde(skip_serializing_if = "is_default", default)]
    pub res: ResponseProfile,
}

impl DiffProfile {
    pub async fn diff(&self, args: ExtraArgs) -> Result<String> {
        let res1 = self.req1.send(&args).await?;
        let res2 = self.req2.send(&args).await?;

        let text1 = res1.filter_text(&self.res).await?;
        let text2 = res2.filter_text(&self.res).await?;
        diff_text(&text1, &text2)
    }

    pub fn validate(&self) -> Result<()> {
        self.req1.validate().context("req1 failed to validate ")?;
        self.req2.validate().context("req1 failed to validate ")?;
        Ok(())
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

    pub fn validate(&self) -> Result<()> {
        for (name, profile) in &self.profiles {
            profile
                .validate()
                .context(format!("failed to validate profile: {}", name))?;
        }
        Ok(())
    }
}
