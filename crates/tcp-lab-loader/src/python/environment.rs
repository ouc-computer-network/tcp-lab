use anyhow::{Context, Result};
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct PythonEnvironment {
    sys_paths: Vec<PathBuf>,
}

impl PythonEnvironment {
    pub fn from_uv(project_root: PathBuf, extra_paths: &[PathBuf]) -> Result<Self> {
        let script = "import json, sys; print(json.dumps(sys.path))";
        let output = Command::new("uv")
            .arg("run")
            .arg("python")
            .arg("-c")
            .arg(script)
            .current_dir(&project_root)
            .output()
            .context("failed to invoke `uv run python`")?;

        if !output.status.success() {
            anyhow::bail!(
                "`uv run python` failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let raw_paths: Vec<String> = serde_json::from_slice(&output.stdout)
            .context("failed to parse sys.path JSON emitted by uv")?;

        let mut paths: Vec<PathBuf> = raw_paths.into_iter().map(PathBuf::from).collect();
        paths.extend(extra_paths.iter().cloned());

        Ok(Self { sys_paths: paths })
    }

    pub fn from_paths(paths: Vec<PathBuf>) -> Self {
        Self { sys_paths: paths }
    }

    pub fn inject(&self, py: Python<'_>) -> PyResult<()> {
        if self.sys_paths.is_empty() {
            return Ok(());
        }
        let sys = py.import("sys")?;
        let py_path: Bound<'_, PyList> = sys.getattr("path")?.cast_into()?;
        for path in &self.sys_paths {
            if let Some(value) = path.to_str() {
                py_path.insert(0, value)?;
            }
        }
        Ok(())
    }
}
