extern crate thiserror;
use thiserror::Error;

use super::snpe_bindings::*;

/// Class of errors possible during DL container loading
#[derive(Debug, Error)]
pub enum DlContainerError {
    #[error("Model parsing failed")]
    ModelParsingFailed(String),

    #[error("Unknown layer code")]
    UnknownLayerCode(String),

    #[error("Missing layer parameter")]
    MissingLayerParam(String),

    #[error("Layer parameter is not supported")]
    LayerParamNotSupported(String),

    #[error("Layer parameter is invalid")]
    LayerParamInvalid(String),

    #[error("Tensor data is missing")]
    TensorDataMissing(String),

    #[error("Model load failed")]
    ModelLoadFailed(String),

    #[error("Missing records")]
    MissingRecords(String),

    #[error("Invalid record")]
    InvalidRecord(String),

    #[error("Write failure")]
    WriteFailure(String),

    #[error("Read failure")]
    ReadFailure(String),

    #[error("Bad container")]
    BadContainer(String),

    #[error("Bad DNN format version")]
    BadDnnFormatVersion(String),

    #[error("Unknown axis annotation")]
    UnknownAxisAnnotation(String),

    #[error("Unknown shuffle type")]
    UnknownShuffleType(String),

    #[error("Temp file failure")]
    TempFileFailure(String),

    #[error("Unknown error")]
    Unknown,
}

impl DlContainerError {
    pub fn from_error(code: Snpe_ErrorCode_t, message: String) -> Self {
        match code {
            Snpe_ErrorCode_t_SNPE_ERRORCODE_DLCONTAINER_MODEL_PARSING_FAILED => {
                Self::ModelParsingFailed(message)
            }
            Snpe_ErrorCode_t_SNPE_ERRORCODE_DLCONTAINER_UNKNOWN_LAYER_CODE => {
                Self::UnknownLayerCode(message)
            }
            Snpe_ErrorCode_t_SNPE_ERRORCODE_DLCONTAINER_MISSING_LAYER_PARAM => {
                Self::MissingLayerParam(message)
            }
            Snpe_ErrorCode_t_SNPE_ERRORCODE_DLCONTAINER_READ_FAILURE => Self::ReadFailure(message),
            Snpe_ErrorCode_t_SNPE_ERRORCODE_DLCONTAINER_BAD_CONTAINER => {
                Self::BadContainer(message)
            }
            Snpe_ErrorCode_t_SNPE_ERRORCODE_DLCONTAINER_BAD_DNN_FORMAT_VERSION => {
                Self::BadDnnFormatVersion(message)
            }
            Snpe_ErrorCode_t_SNPE_ERRORCODE_DLCONTAINER_UNKNOWN_AXIS_ANNOTATION => {
                Self::UnknownAxisAnnotation(message)
            }
            Snpe_ErrorCode_t_SNPE_ERRORCODE_DLCONTAINER_UNKNOWN_SHUFFLE_TYPE => {
                Self::UnknownShuffleType(message)
            }
            Snpe_ErrorCode_t_SNPE_ERRORCODE_DLCONTAINER_TEMP_FILE_FAILURE => {
                Self::TempFileFailure(message)
            }
            _ => Self::Unknown,
        }
    }
}
