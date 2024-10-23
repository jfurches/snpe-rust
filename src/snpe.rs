use std::ffi::CString;
use std::path::PathBuf;

use libloading::Library;
use semver::{BuildMetadata, Prerelease, Version};

pub mod snpe_bindings {
    include!(concat!(env!("OUT_DIR"), "/snpe_bindings.rs"));

    pub static LIB: &str = concat!(env!("SNPE_LIB_DIR"), "/libSNPE.so");
}

/// Instance of the SNPE runtime
struct Snpe {}

impl Snpe {
    /// Creates a new SNPE instance
    fn new() -> Self {
        Self {}
    }
}

/// Returns the SNPE library version
fn get_version() -> Version {
    let version: Version;
    unsafe {
        let snpe = snpe_bindings::SNPE::new(snpe_bindings::LIB).unwrap();

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
        let snpe = snpe_bindings::SNPE::new(snpe_bindings::LIB).unwrap();

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
            let snpe = snpe_bindings::SNPE::new(snpe_bindings::LIB).unwrap();
            snpe.Snpe_Util_IsRuntimeAvailable(self.id()) != 0
        }
    }
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
