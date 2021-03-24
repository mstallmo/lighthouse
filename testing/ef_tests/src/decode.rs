use super::*;
use snap::raw::Decoder;
use std::fs::{self};
use std::path::Path;

pub fn yaml_decode<T: serde::de::DeserializeOwned>(string: &str) -> Result<T, Error> {
    serde_yaml::from_str(string).map_err(|e| Error::FailedToParseTest(format!("{:?}", e)))
}

pub fn yaml_decode_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, Error> {
    fs::read_to_string(path)
        .map_err(|e| {
            Error::FailedToParseTest(format!("Unable to load {}: {:?}", path.display(), e))
        })
        .and_then(|s| yaml_decode(&s))
}

/// Decode a Snappy encoded file.
///
/// Files in the EF tests are unframed, so we need to use `snap::raw::Decoder`.
pub fn snappy_decode_file(path: &Path) -> Result<Vec<u8>, Error> {
    let bytes = fs::read(path).map_err(|e| {
        Error::FailedToParseTest(format!("Unable to load {}: {:?}", path.display(), e))
    })?;
    let mut decoder = Decoder::new();
    decoder.decompress_vec(&bytes).map_err(|e| {
        Error::FailedToParseTest(format!(
            "Error decoding snappy encoding for {}: {:?}",
            path.display(),
            e
        ))
    })
}

pub fn ssz_decode_file<T: ssz::Decode>(path: &Path) -> Result<T, Error> {
    let bytes = snappy_decode_file(path)?;
    T::from_ssz_bytes(&bytes).map_err(|e| {
        match e {
            // NOTE: this is a bit hacky, but seemingly better than the alternatives
            ssz::DecodeError::BytesInvalid(message)
                if message.contains("Blst") || message.contains("Milagro") =>
            {
                Error::InvalidBLSInput(message)
            }
            e => Error::FailedToParseTest(format!(
                "Unable to parse SSZ at {}: {:?}",
                path.display(),
                e
            )),
        }
    })
}
