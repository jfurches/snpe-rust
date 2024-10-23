mod errors;
use std::ffi::CString;
use std::path::PathBuf;

use self::errors::DlContainerError;
use super::snpe::snpe_bindings;

/// Model instance for the SNPE runtime
#[derive(Debug)]
struct Model {
    /// File path to the .dlc model file
    path: PathBuf,

    /// Internal handle to the c object
    _handle: snpe_bindings::Snpe_DlContainer_Handle_t,
}

impl Model {
    /// Creates a new Model from the given path to a .dlc or .bin file
    fn new<P>(path: P) -> Result<Model, DlContainerError>
    where
        P: AsRef<str>,
    {
        let handle: snpe_bindings::Snpe_DlContainer_Handle_t;
        let c_path = CString::new(path.as_ref()).unwrap();
        let model: Model;
        let pathbuf = PathBuf::from(path.as_ref());

        unsafe {
            let snpe = snpe_bindings::SNPE::new(snpe_bindings::LIB).unwrap();
            handle = snpe.Snpe_DlContainer_Open(c_path.as_ptr());

            if handle.is_null() {
                let code = snpe.Snpe_ErrorCode_getLastErrorCode();
                let msg = snpe.Snpe_ErrorCode_GetLastErrorString();
                let msg_str = String::from(std::ffi::CStr::from_ptr(msg).to_str().unwrap());

                return Err(DlContainerError::from_error(code, msg_str));
            }

            model = Model {
                path: pathbuf,
                _handle: handle,
            };
        }

        Ok(model)
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        // Clean up the dlcontainer handle
        unsafe {
            let snpe = snpe_bindings::SNPE::new(snpe_bindings::LIB).unwrap();

            if !self._handle.is_null() {
                snpe.Snpe_DlContainer_Delete(self._handle);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::errors::DlContainerError;

    #[test]
    fn does_not_exist() {
        let model = super::Model::new("does_not_exist.dlc");
        assert!(matches!(model, Err(DlContainerError::ReadFailure(_))))
    }

    #[test]
    fn dummy_file() {
        let model = super::Model::new("test/data/dummy.dlc");
        assert!(matches!(model, Err(DlContainerError::ReadFailure(_))))
    }

    // todo: convert one of the models in the sdk and try loading
}
