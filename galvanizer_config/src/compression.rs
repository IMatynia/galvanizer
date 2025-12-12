use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(tag = "algorithm")]
pub enum CompressionOptions {
    #[default]
    Disabled,
    Snap,
}
