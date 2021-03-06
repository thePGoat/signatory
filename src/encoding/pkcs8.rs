//! Support for the **PKCS#8** private key format described in [RFC 5208]
//! and [RFC 5915].
//!
//! [RFC 5208]: https://tools.ietf.org/html/rfc5208
//! [RFC 5915]: https://tools.ietf.org/html/rfc5915

use error::Error;
#[cfg(feature = "std")]
use prelude::*;
#[cfg(feature = "std")]
use std::io::Write;
#[cfg(feature = "std")]
use std::{fs::File, io::Read, path::Path};
#[cfg(all(unix, feature = "std"))]
use std::{fs::OpenOptions, os::unix::fs::OpenOptionsExt};
#[cfg(feature = "std")]
use zeroize::secure_zero_memory;

#[cfg(feature = "std")]
use super::FILE_MODE;

/// Load this type from a **PKCS#8** private key
pub trait FromPkcs8: Sized {
    /// Load from the given **PKCS#8**-encoded private key, returning `Self`
    /// or an error if the given data couldn't be loaded.
    fn from_pkcs8<K: AsRef<[u8]>>(secret_key: K) -> Result<Self, Error>;

    /// Read **PKCS#8** data from the given `std::io::Read`.
    #[cfg(feature = "std")]
    fn read_pkcs8<R: Read>(mut reader: R) -> Result<Self, Error> {
        let mut bytes = vec![];
        reader
            .read_to_end(&mut bytes)
            .map_err(|e| err!(KeyInvalid, "error reading key: {}", e))?;
        let result = Self::from_pkcs8(&bytes);
        secure_zero_memory(&mut bytes);
        result
    }

    /// Read **PKCS#8** data from the file at the given path.
    #[cfg(feature = "std")]
    fn from_pkcs8_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::read_pkcs8(File::open(path)?)
    }
}

/// Generate a random **PKCS#8** private key of this type
#[cfg(feature = "std")]
pub trait GeneratePkcs8: Sized + FromPkcs8 {
    /// Randomly generate a **PKCS#8** private key for this type loadable
    /// via `from_pkcs8()`.
    fn generate_pkcs8() -> Result<SecretKey, Error>;

    /// Write randomly generated **PKCS#8** private key to the file at the
    /// given path.
    ///
    /// If the file does not exist, it will be created with a mode of
    /// `FILE_MODE` (i.e. `600`). If the file does exist, it will be erased
    /// and replaced.
    #[cfg(unix)]
    fn generate_pkcs8_file<P: AsRef<Path>>(path: P) -> Result<File, Error> {
        let secret_key = Self::generate_pkcs8()?;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(FILE_MODE)
            .open(path)?;

        file.write_all(secret_key.as_ref())?;
        Ok(file)
    }

    /// Encode `self` and write it to a file at the given path, returning the
    /// resulting `File` or a `Error`.
    ///
    /// If the file does not exist, it will be created.
    #[cfg(not(unix))]
    fn generate_pkcs8_file<P: AsRef<Path>>(path: P) -> Result<File, Error> {
        let secret_key = Self::generate_pkcs8()?;
        let mut file = File::create(path.as_ref())?;
        file.write_all(secret_key.as_ref())?;
        Ok(file)
    }
}

/// **PKCS#8** keypairs containing public keys and secret keys
#[cfg(feature = "alloc")]
pub struct SecretKey(Vec<u8>);

#[cfg(feature = "alloc")]
impl SecretKey {
    /// Create a new **PKCS#8** `PrivateKey` from the given bytes.
    // TODO: parse the document and verify it's well-formed
    pub fn new(secret_key_bytes: &[u8]) -> Result<Self, Error> {
        Ok(SecretKey(secret_key_bytes.to_vec()))
    }
}

#[cfg(feature = "alloc")]
impl AsRef<[u8]> for SecretKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg(feature = "alloc")]
impl Drop for SecretKey {
    fn drop(&mut self) {
        self.0.clear()
    }
}
