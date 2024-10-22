/// Module for the Snapdragon QAIRT
use std::{collections::HashMap, ffi::OsStr, path::PathBuf};

use tensor_rs::tensor::Tensor;

enum ModelFormat {
    Dlc,
    Binary,
}

/// Based on QairtModel in python api
struct Model {
    path: PathBuf,
    backend: String,
    // executor: Option<Box<dyn Executor>>,
}

impl Model {
    /// Creates a new QairtModel from the given path to a .dlc or .bin file
    fn new<P>(path: &P, backend: String) -> Self
    where
        P: AsRef<OsStr>,
    {
        Self {
            path: PathBuf::from(path),
            backend,
            // executor: None,
        }
    }

    /// Determines the format of the model
    fn format(&self) -> Result<ModelFormat, &str> {
        match self.path.extension().map(|ext| ext.to_str()).flatten() {
            Some("dlc") => Ok(ModelFormat::Dlc),
            Some("bin") => Ok(ModelFormat::Binary),
            _ => Err("Unsupported model format"),
        }
    }

    /// Determines if the model is loaded or not
    fn is_loaded(&self) -> bool {
        // self.executor.is_some()
        false
    }

    fn load(&mut self) -> Result<(), &str> {
        if self.is_loaded() {
            return Err("Model is already loaded. Call unload() first");
        }

        // todo: load model
        // self.executor = Some(Box::new());

        Ok(())
    }

    fn unload(&mut self) -> Result<(), &str> {
        if !self.is_loaded() {
            return Err("Model is not loaded");
        }

        // todo: unload model
        // self.executor = None;

        Ok(())
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        if self.is_loaded() {
            self.unload().expect("Unable to unload model")
        }
    }
}

struct ModelInput {
    string: Option<String>,
    array: Option<Tensor>,
    dict: Option<HashMap<String, Tensor>>,
}

impl ModelInput {
    fn from_str(s: &str) -> Self {
        Self {
            string: Some(s.to_string()),
            array: None,
            dict: None,
        }
    }

    fn from_array(array: Tensor) -> Self {
        Self {
            string: None,
            array: Some(array),
            dict: None,
        }
    }

    fn from_dict(dict: HashMap<String, Tensor>) -> Self {
        Self {
            string: None,
            array: None,
            dict: Some(dict),
        }
    }
}
