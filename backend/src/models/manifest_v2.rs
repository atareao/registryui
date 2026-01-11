use serde::{Serialize, Deserialize};
use super::layer_descriptor::LayerDescriptor;
use super::config_descriptor::ConfigDescriptor;

#[derive(Serialize, Deserialize, Debug)]
pub struct ManifestV2 {
    #[serde(rename = "schemaVersion")]
    pub schema_version: i32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub config: ConfigDescriptor,
    pub layers: Vec<LayerDescriptor>,
}
