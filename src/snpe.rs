use std::ffi::{CStr, CString};
use std::path::PathBuf;

use libloading::Library;
use semver::{BuildMetadata, Prerelease, Version};
use tensor_rs::tensor::Tensor;

pub mod snpe_bindings {
    include!(concat!(env!("OUT_DIR"), "/snpe_bindings.rs"));

    pub static LIB: &str = concat!(env!("SNPE_LIB_DIR"), "/libSNPE.so");

    pub unsafe fn get() -> SNPE {
        SNPE::new(LIB).unwrap()
    }
}

/// Instance of the SNPE runtime
struct Snpe {
    handle: snpe_bindings::Snpe_SNPE_Handle_t,
}

impl Snpe {
    /// Creates a new SNPE instance
    fn new() -> Self {
        // todo: implement

        Self {
            handle: std::ptr::null_mut(),
        }
    }

    // fn get_input_tensors(&self) -> Result<Vec<TensorInfo>, &str> {
    //     let snpe = unsafe { snpe_bindings::SNPE::new(snpe_bindings::LIB).unwrap() };

    //     let cnames = self.get_input_tensor_cnames()?;
    //     let friendly_names = self.get_input_tensor_names()?;
    //     let result: Vec<TensorInfo> = vec![];

    //     for i in 0..cnames.len() {
    //         snpe.Snpe_GetIn
    //     }
    // }

    /// Returns the list of names of input tensors to the network
    fn get_input_tensor_names(&self) -> Result<Vec<String>, &str> {
        let names = self
            .get_input_tensor_cnames()?
            .into_iter()
            .map(|c| c.to_string_lossy().to_string())
            .collect();

        Ok(names)
    }

    /// Returns a list of owned input tensor names
    fn get_input_tensor_cnames(&self) -> Result<Vec<CString>, &str> {
        let mut result: Vec<CString> = vec![];

        let snpe = unsafe { snpe_bindings::get() };
        let inputNamesHandle = unsafe { snpe.Snpe_SNPE_GetInputTensorNames(self.handle) };

        if inputNamesHandle.is_null() {
            return Err("Failed to get input tensor names");
        }

        let n = unsafe { snpe.Snpe_StringList_Size(inputNamesHandle) };
        for i in 0..n {
            let cstr = unsafe { CStr::from_ptr(snpe.Snpe_StringList_At(inputNamesHandle, i)) };

            // Copy the string into rust
            result.push(cstr.to_owned());
        }

        // Free the string list in C
        unsafe {
            snpe.Snpe_StringList_Delete(inputNamesHandle);
        }

        Ok(result)
    }
}

/// Returns the SNPE library version
fn get_version() -> Version {
    let version: Version;
    unsafe {
        let snpe = snpe_bindings::get();

        let versionHandle = snpe.Snpe_Util_GetLibraryVersion();

        version = Version {
            major: snpe.Snpe_DlVersion_GetMajor(versionHandle) as u64,
            minor: snpe.Snpe_DlVersion_GetMinor(versionHandle) as u64,
            patch: snpe.Snpe_DlVersion_GetTeeny(versionHandle) as u64,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        };

        snpe.Snpe_DlVersion_Delete(versionHandle);
    }

    version
}

/// Returns the list of available accelerator devices
fn get_available_devices() -> Vec<Device> {
    let devices = [Device::Cpu, Device::Gpu, Device::Npu, Device::Aip];
    let mut available: Vec<Device> = vec![];

    unsafe {
        let snpe = snpe_bindings::get();

        for device in devices {
            if snpe.Snpe_Util_IsRuntimeAvailable(device.id()) != 0 {
                available.push(device);
            }
        }
    }

    available
}

/// Enum containing the possible runtime environments for the SNPE library
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Device {
    /// Standard Cpu device with float32 math
    Cpu,
    /// Adreno Gpu device with 16 bit data and 32 bit math
    Gpu,
    /// Hexagon Npu device with 8 bit fixed math
    Npu,
    /// Running on the Snapdragon AIX+HVX. 8 bit fixed math
    Aip,
}

impl Device {
    /// Returns a friendly name for the device
    fn name(&self) -> String {
        match self {
            Device::Cpu => "CPU",
            Device::Gpu => "GPU",
            Device::Npu => "NPU",
            Device::Aip => "AIP",
        }
        .to_string()
    }

    /// Returns the SNPE runtime id
    fn id(&self) -> i32 {
        match self {
            Device::Cpu => snpe_bindings::Snpe_Runtime_t_SNPE_RUNTIME_CPU_FLOAT32,
            Device::Gpu => snpe_bindings::Snpe_Runtime_t_SNPE_RUNTIME_GPU_FLOAT32_16_HYBRID,
            Device::Npu => snpe_bindings::Snpe_Runtime_t_SNPE_RUNTIME_DSP_FIXED8_TF,
            Device::Aip => snpe_bindings::Snpe_Runtime_t_SNPE_RUNTIME_AIP_FIXED_TF,
        }
    }

    /// Returns if the device is available
    fn is_available(&self) -> bool {
        unsafe {
            let snpe = snpe_bindings::get();
            snpe.Snpe_Util_IsRuntimeAvailable(self.id()) != 0
        }
    }
}

struct TensorInfo {
    name: String,
    shape: Vec<u64>,
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use crate::snpe::{get_available_devices, get_version, Device, Snpe};

    #[test]
    fn test_version() {
        let version = get_version();
        assert_eq!(version, Version::parse("2.26.0").unwrap());
    }

    #[test]
    fn test_runtimes() {
        let devices = get_available_devices();
        assert!(devices.len() > 0);

        let device = Device::Cpu;
        assert!(device.is_available());
    }

    #[test]
    #[cfg(all(target_arch = "aarch64", target_os = "windows"))]
    fn test_windows_on_arm() {
        assert!(Device::Cpu.is_available());
        assert!(Device::Npu.is_available());
    }

    #[test]
    #[cfg(all(target_arch = "aarch64", target_os = "android"))]
    fn test_android_on_arm() {
        assert!(Device::Cpu.is_available());
        assert!(Device::Gpu.is_available());
    }
}
