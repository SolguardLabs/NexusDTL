use crate::{NexusError, NexusResult};

pub fn run() -> NexusResult<()> {
    let scenario = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "routed".to_owned());
    let report = super::scenarios::run_named(&scenario)?;
    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| NexusError::Serialization(error.to_string()))?;
    println!("{json}");
    Ok(())
}
