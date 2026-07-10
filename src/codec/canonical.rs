use serde::Serialize;

use crate::{NexusError, NexusResult};

pub fn canonical_bytes<T: Serialize>(value: &T) -> NexusResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| NexusError::Serialization(error.to_string()))
}
