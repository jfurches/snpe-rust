use libloading::Library;
use semver::{BuildMetadata, Prerelease, Version};

#[cfg(not(doctest))]
mod snpe_bindings {
    include!(concat!(env!("OUT_DIR"), "/snpe_bindings.rs"));

    pub static LIB: &str = concat!(env!("LIB_DIR"), "/libSNPE.so");
}

struct SNPE {}

impl SNPE {
    fn get_version(&self) -> Version {
        let version: Version;
        unsafe {
            let snpe = snpe_bindings::SNPE::new(snpe_bindings::LIB).unwrap();

            let library_version = snpe.Snpe_Util_GetLibraryVersion();

            version = Version {
                major: snpe.Snpe_DlVersion_GetMajor(library_version) as u64,
                minor: snpe.Snpe_DlVersion_GetMinor(library_version) as u64,
                patch: snpe.Snpe_DlVersion_GetTeeny(library_version) as u64,
                pre: Prerelease::EMPTY,
                build: BuildMetadata::EMPTY,
            }
        }

        version
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use crate::snpe::SNPE;

    #[test]
    fn test_version() {
        let snpe = SNPE {};
        let version = snpe.get_version();
        assert_eq!(version, Version::parse("2.26.0").unwrap());
    }
}
