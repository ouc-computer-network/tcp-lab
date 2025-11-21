use anyhow::{Context, Result};
use pyo3::prelude::*;
use pyo3::types::PyList;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct PythonEnvironment {
    sys_paths: Vec<PathBuf>,
    python_home: Option<PathBuf>,
}

impl PythonEnvironment {
    pub fn from_uv(project_root: PathBuf, extra_paths: &[PathBuf]) -> Result<Self> {
        // Get both sys.path and Python home from uv environment
        let script = r#"
import json, sys, sysconfig
print(json.dumps({
    "sys_path": sys.path,
    "prefix": sys.prefix,
    "base_prefix": sys.base_prefix
}))
"#;
        let output = Command::new("uv")
            .arg("run")
            .arg("python")
            .arg("-c")
            .arg(script)
            .current_dir(&project_root)
            .output()
            .with_context(|| {
                format!(
                    "failed to invoke `uv run python` (PATH = {:?})",
                    std::env::var("PATH")
                )
            })?;

        if !output.status.success() {
            anyhow::bail!(
                "`uv run python` failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        #[derive(Deserialize)]
        struct PythonInfo {
            sys_path: Vec<String>,
            #[allow(dead_code)]
            prefix: String,
            base_prefix: String,
        }

        let info: PythonInfo = serde_json::from_slice(&output.stdout)
            .context("failed to parse Python info JSON emitted by uv")?;

        let mut paths: Vec<PathBuf> = info.sys_path.into_iter().map(PathBuf::from).collect();
        paths.extend(extra_paths.iter().cloned());

        // Use base_prefix as Python home (points to the actual Python installation)
        let python_home = PathBuf::from(info.base_prefix);

        Ok(Self {
            sys_paths: paths,
            python_home: Some(python_home),
        })
    }

    pub fn from_paths(paths: Vec<PathBuf>) -> Self {
        Self {
            sys_paths: paths,
            python_home: None,
        }
    }

    pub fn python_home(&self) -> Option<&PathBuf> {
        self.python_home.as_ref()
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
