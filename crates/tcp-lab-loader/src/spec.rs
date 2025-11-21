use anyhow::{Context, Result};

use crate::BuiltinProtocol;

/// Parse a Python implementation spec of the form `module.Class`.
pub fn parse_python_spec(spec: &str) -> Result<(String, String)> {
    spec.rsplit_once('.')
        .map(|(module, class)| (module.to_string(), class.to_string()))
        .context("Python class should be provided as module.Class")
}

/// Map a user-visible builtin name to the enum used by the loader.
pub fn builtin_by_name(name: &str, is_sender: bool) -> Result<BuiltinProtocol> {
    match name {
        "rdt2" => Ok(if is_sender {
            BuiltinProtocol::Rdt2Sender
        } else {
            BuiltinProtocol::Rdt2Receiver
        }),
        other => anyhow::bail!("Unknown builtin '{other}'. Try 'rdt2'."),
    }
}
