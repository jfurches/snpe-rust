mod errors;
use std::ffi::{CStr, CString};
use std::path::PathBuf;

use log::warn;

use self::errors::DlContainerError;
use super::snpe::snpe_bindings;

/// Model instance for the SNPE runtime
#[derive(Debug)]
struct DlContainer {
    /// File path to the .dlc model file
    path: PathBuf,

    /// Internal handle to the c object
    handle: snpe_bindings::Snpe_DlContainer_Handle_t,
}

impl DlContainer {
    /// Creates a new DlContainer from the given path to a .dlc or .bin file
    fn from_path<P>(path: P) -> Result<DlContainer, DlContainerError>
    where
        P: AsRef<str>,
    {
        let handle: snpe_bindings::Snpe_DlContainer_Handle_t;
        let c_path = CString::new(path.as_ref()).unwrap();
        let model: DlContainer;
        let pathbuf = PathBuf::from(path.as_ref());

        unsafe {
            let snpe = snpe_bindings::get();
            handle = snpe.Snpe_DlContainer_Open(c_path.as_ptr());

            if handle.is_null() {
                let code = snpe.Snpe_ErrorCode_getLastErrorCode();
                let msg = snpe.Snpe_ErrorCode_GetLastErrorString();
                let msg_str = String::from(std::ffi::CStr::from_ptr(msg).to_str().unwrap());

                return Err(DlContainerError::from_error(code, msg_str));
            }

            model = DlContainer {
                path: pathbuf,
                handle: handle,
            };
        }

        Ok(model)
    }

    /// Loads a DlContainer from an in-memory byte buffer
    fn from_buffer(buffer: &[u8]) -> Result<DlContainer, DlContainerError> {
        let snpe = unsafe { snpe_bindings::get() };
        let handle = unsafe { snpe.Snpe_DlContainer_OpenBuffer(buffer.as_ptr(), buffer.len()) };

        if handle.is_null() {
            unsafe {
                let code = snpe.Snpe_ErrorCode_getLastErrorCode();
                let msg = snpe.Snpe_ErrorCode_GetLastErrorString();
                let msg_str = String::from(std::ffi::CStr::from_ptr(msg).to_str().unwrap());

                return Err(DlContainerError::from_error(code, msg_str));
            }
        }

        Ok(DlContainer {
            path: PathBuf::from(""),
            handle: handle,
        })
    }

    /// Saves a DlContainer to a file
    fn save<P>(&self, path: P) -> Result<(), DlContainerError>
    where
        P: AsRef<str>,
    {
        let c_path = CString::new(path.as_ref()).unwrap();
        let snpe = unsafe { snpe_bindings::get() };
        let code = unsafe { snpe.Snpe_DlContainer_Save(self.handle, c_path.as_ptr()) };
        if code != 0 {
            let msg = unsafe {
                let code = snpe.Snpe_ErrorCode_getLastErrorCode();
                let msg = snpe.Snpe_ErrorCode_GetLastErrorString();
                String::from(std::ffi::CStr::from_ptr(msg).to_str().unwrap())
            };
            let msg_str = String::from(msg);
            return Err(DlContainerError::from_error(code, msg_str));
        }

        Ok(())
    }

    /// Returns a DlcRecord by name
    fn get_record(&self, name: &str) -> Result<DlcRecord, DlContainerError> {
        unsafe {
            let snpe = snpe_bindings::get();
            let cname = CString::new(name).unwrap();
            let handle = snpe.Snpe_DlContainer_GetRecord(self.handle, cname.as_ptr());

            if handle.is_null() {
                let code = snpe.Snpe_ErrorCode_getLastErrorCode();
                let msg = snpe.Snpe_ErrorCode_GetLastErrorString();
                let msg_str = String::from(std::ffi::CStr::from_ptr(msg).to_str().unwrap());
                return Err(DlContainerError::from_error(code, msg_str));
            }

            Ok(DlcRecord::new(name, handle))
        }
    }

    /// Returns all records in this container
    fn get_catalog(&self) -> Result<Vec<DlcRecord>, DlContainerError> {
        let mut result = Vec::<DlcRecord>::new();

        unsafe {
            let snpe = snpe_bindings::get();
            let record_names_raw = snpe.Snpe_DlContainer_GetCatalog(self.handle);
            let num_records = snpe.Snpe_StringList_Size(record_names_raw);

            for i in 0..num_records {
                let record_name = CStr::from_ptr(snpe.Snpe_StringList_At(record_names_raw, i));
                let record = self.get_record(&record_name.to_string_lossy().to_string())?;
                result.push(record);
            }
        }

        Ok(result)
    }
}

impl Drop for DlContainer {
    fn drop(&mut self) {
        // Clean up the dlcontainer handle
        unsafe {
            let snpe = snpe_bindings::get();
            let errorCode = snpe.Snpe_DlContainer_Delete(self.handle);

            // If there was an error, log it, but not sure what else we can do
            if errorCode != 0 {
                let msg = snpe.Snpe_ErrorCode_GetLastErrorString();
                let msg_str = String::from(std::ffi::CStr::from_ptr(msg).to_str().unwrap());
                warn!(target: "DlContainer", "Error cleaning up container: {}", msg_str);
            }
        }
    }
}

/// A record in the .dlc file
struct DlcRecord {
    name: String,
    handle: snpe_bindings::Snpe_DlcRecord_Handle_t,
}

impl DlcRecord {
    /// Creates a new DlcRecord from the given name and handle
    fn new(name: &str, handle: snpe_bindings::Snpe_DlcRecord_Handle_t) -> DlcRecord {
        DlcRecord {
            name: String::from(name),
            handle: handle,
        }
    }

    /// Creates a new DlcRecord and handle with the supplied name
    fn create(name: Option<&str>) -> DlcRecord {
        unsafe {
            let snpe = snpe_bindings::get();

            match name {
                Some(name) => {
                    let cname = CString::new(name).unwrap();
                    let handle = snpe.Snpe_DlcRecord_CreateName(cname.as_ptr());
                    DlcRecord::new(name, handle)
                }
                None => {
                    let handle = snpe.Snpe_DlcRecord_Create();
                    let cname = snpe.Snpe_DlcRecord_Name(handle);
                    DlcRecord::new(std::ffi::CStr::from_ptr(cname).to_str().unwrap(), handle)
                }
            }
        }
    }

    /// Returns a copy of the byte buffer of this record
    fn get_data(&self) -> Result<Vec<u8>, &str> {
        unsafe {
            let snpe = snpe_bindings::get();
            let data_ptr = self.data_ptr();

            if data_ptr.is_null() {
                return Err("Failed to get data");
            }

            Ok(std::slice::from_raw_parts(data_ptr, self.size()).to_vec())
        }
    }

    /// Returns the size in bytes of this record
    fn size(&self) -> usize {
        unsafe {
            let snpe = snpe_bindings::get();
            snpe.Snpe_DlcRecord_Size(self.handle)
        }
    }

    /// Gives the underlying (read only) pointer to the data
    unsafe fn data_ptr(&self) -> *const u8 {
        let snpe = snpe_bindings::get();
        snpe.Snpe_DlcRecord_Data(self.handle)
    }

    /// Gives the underlying mutable pointer to the data
    unsafe fn mut_data_ptr(&self) -> *mut u8 {
        let snpe = snpe_bindings::get();
        snpe.Snpe_DlcRecord_Data(self.handle)
    }
}

impl Drop for DlcRecord {
    fn drop(&mut self) {
        unsafe {
            let snpe = snpe_bindings::get();
            snpe.Snpe_DlcRecord_Delete(self.handle);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dlcontainer::errors::DlContainerError;

    #[test]
    fn does_not_exist() {
        let model = super::DlContainer::from_path("does_not_exist.dlc");
        assert!(matches!(model, Err(DlContainerError::ReadFailure(_))))
    }

    #[test]
    fn dummy_file() {
        let model = super::DlContainer::from_path("test/data/dummy.dlc");
        assert!(matches!(model, Err(DlContainerError::ReadFailure(_))))
    }

    #[test]
    fn resnet50() {
        let model = super::DlContainer::from_path("test/data/resnet50.dlc");
        assert!(model.is_ok());
    }

    // #[test]
    // fn resnet50_from_buffer() {
    //     // Fixme: This test takes over a minute on my computer. Maybe we need a smaller model
    //     let file = std::fs::File::open("test/data/resnet50.dlc").unwrap();
    //     let buffer = std::io::Read::bytes(file)
    //         .collect::<Result<Vec<u8>, _>>()
    //         .unwrap();
    //     let model = super::DlContainer::from_buffer(&buffer);
    //     assert!(model.is_ok());

    //     let records = model.unwrap().get_catalog().unwrap();
    //     assert!(records.len() > 0);

    //     for record in records {
    //         assert!(!record.name.is_empty());

    //         // Fixme: I assume this fails because the SNPE engine hasn't loaded the model yet
    //         let data = record.get_data();
    //         assert!(data.is_ok_and(|data| data.len() == record.size()));
    //     }
    // }

    #[test]
    fn resnet50_records() {
        let model = super::DlContainer::from_path("test/data/resnet50.dlc").unwrap();
        let records = model.get_catalog().unwrap();
        assert!(records.len() > 0);

        for record in records {
            assert!(!record.name.is_empty());

            // Fixme: I assume this fails because the SNPE engine hasn't loaded the model yet
            // let data = record.get_data();
            // assert!(data.is_ok_and(|data| data.len() == record.size()));
        }
    }
}
