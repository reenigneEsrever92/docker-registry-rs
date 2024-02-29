use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
struct ManifestV2 {
    schema_version: u8
}