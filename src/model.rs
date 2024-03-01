use std::collections::HashMap;
use std::default::Default;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct ManifestV2Schema2 {
    schema_version: u8,
    media_type: String,
    config: Option<ManifestConfig>,
    layers: Vec<Layer>,
    subject: Subject,
    annotations: HashMap<String, String>
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct ManifestConfig {
    media_type: String,
    digest: String,
    size: u32
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct Layer {
    media_type: String,
    digest: String,
    size: u32
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub(crate) struct Subject {
    media_type: String,
    digest: String,
    size: u32
}