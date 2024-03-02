use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManifestV2Schema2 {
    pub(crate) schema_version: u8,
    pub(crate) media_type: String,
    pub(crate) config: Option<ManifestConfig>,
    pub(crate) layers: Option<Vec<Layer>>,
    pub(crate) subject: Option<Subject>,
    pub(crate) annotations: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManifestConfig {
    media_type: String,
    digest: String,
    size: u32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Layer {
    media_type: String,
    digest: String,
    size: u32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Subject {
    media_type: String,
    digest: String,
    size: u32,
}
