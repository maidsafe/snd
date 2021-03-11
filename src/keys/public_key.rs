// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! Module providing keys, keypairs, and signatures.
//!
//! The easiest way to get a `PublicKey` is to create a random `Keypair` first through one of the
//! `new` functions. A `PublicKey` can't be generated by itself; it must always be derived from a
//! secret key.

use crate::{utils, Error, Result};
use crate::{Keypair, Signature};

use serde::{Deserialize, Serialize};
use signature::Verifier;
use std::{
    cmp::Ordering,
    convert::TryInto,
    fmt::{self, Debug, Display, Formatter, LowerHex, UpperHex},
    hash::{Hash, Hasher},
};
// use threshold_crypto::{self};
use xor_name::{XorName, XOR_NAME_LEN};

/// Wrapper for different public key types.
#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum PublicKey {
    /// Ed25519 public key.
    Ed25519(ed25519_dalek::PublicKey),
    /// BLS public key.
    Bls(threshold_crypto::PublicKey),
    /// BLS public key share.
    BlsShare(threshold_crypto::PublicKeyShare),
}

impl PublicKey {
    /// Construct and ed25519 public key from
    /// a hex-encoded string.
    ///
    /// It is often useful
    /// to parse such raw strings in user-facing
    /// apps like CLI
    pub fn ed25519_from_hex(hex: &str) -> Result<Self> {
        let bytes = hex::decode(hex).map_err(|e| Error::FailedToParse(e.to_string()))?;
        let pk = ed25519_dalek::PublicKey::from_bytes(bytes.as_ref())
            .map_err(|e| Error::FailedToParse(e.to_string()))?;
        Ok(Self::from(pk))
    }

    /// Construct and ed25519 public key from
    /// a hex-encoded string.
    ///
    /// It is often useful
    /// to parse such raw strings in user-facing
    /// apps like CLI
    pub fn bls_from_hex(hex: &str) -> Result<Self> {
        let bytes = hex::decode(hex).map_err(|e| Error::FailedToParse(e.to_string()))?;
        let bytes_fixed_len: &[u8; threshold_crypto::PK_SIZE] = bytes.as_slice().try_into()
            .map_err(|_| Error::FailedToParse(format!(
                "Couldn't parse BLS public key from hex. The provided string must represent exactly {} bytes.",
                threshold_crypto::PK_SIZE
            )))?;
        let pk = threshold_crypto::PublicKey::from_bytes(bytes_fixed_len)
            .map_err(|e| Error::FailedToParse(e.to_string()))?;
        Ok(Self::from(pk))
    }

    /// Returns the bytes of the underlying public key
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            PublicKey::Ed25519(pub_key) => pub_key.to_bytes().into(),
            PublicKey::Bls(pub_key) => pub_key.to_bytes().into(),
            PublicKey::BlsShare(pub_key) => pub_key.to_bytes().into(),
        }
    }

    /// Returns the ed25519 key, if applicable.
    pub fn ed25519(&self) -> Option<ed25519_dalek::PublicKey> {
        if let Self::Ed25519(key) = self {
            Some(*key)
        } else {
            None
        }
    }

    /// Returns the BLS key, if applicable.
    pub fn bls(&self) -> Option<threshold_crypto::PublicKey> {
        if let Self::Bls(key) = self {
            Some(*key)
        } else {
            None
        }
    }

    /// Returns the BLS key share, if applicable.
    pub fn bls_share(&self) -> Option<threshold_crypto::PublicKeyShare> {
        if let Self::BlsShare(key) = self {
            Some(*key)
        } else {
            None
        }
    }

    /// Returns `Ok(())` if `signature` matches the message and `Err(Error::InvalidSignature)`
    /// otherwise.
    pub fn verify<T: AsRef<[u8]>>(&self, signature: &Signature, data: T) -> Result<()> {
        let is_valid = match (self, signature) {
            (Self::Ed25519(pub_key), Signature::Ed25519(sig)) => {
                pub_key.verify(data.as_ref(), sig).is_ok()
            }
            (Self::Bls(pub_key), Signature::Bls(sig)) => pub_key.verify(sig, data),
            (Self::BlsShare(pub_key), Signature::BlsShare(sig)) => pub_key.verify(&sig.share, data),
            _ => return Err(Error::SigningKeyTypeMismatch),
        };
        if is_valid {
            Ok(())
        } else {
            Err(Error::InvalidSignature)
        }
    }

    /// Returns the `PublicKey` serialised and encoded in z-base-32.
    pub fn encode_to_zbase32(&self) -> Result<String> {
        utils::encode(&self)
    }

    /// Creates from z-base-32 encoded string.
    pub fn decode_from_zbase32<I: AsRef<str>>(encoded: I) -> Result<Self> {
        utils::decode(encoded)
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for PublicKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        utils::serialise(&self).unwrap_or_default().hash(state)
    }
}

impl Ord for PublicKey {
    fn cmp(&self, other: &PublicKey) -> Ordering {
        utils::serialise(&self)
            .unwrap_or_default()
            .cmp(&utils::serialise(other).unwrap_or_default())
    }
}

impl PartialOrd for PublicKey {
    fn partial_cmp(&self, other: &PublicKey) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<PublicKey> for XorName {
    fn from(public_key: PublicKey) -> Self {
        let bytes = match public_key {
            PublicKey::Ed25519(pub_key) => {
                return XorName(pub_key.to_bytes());
            }
            PublicKey::Bls(pub_key) => pub_key.to_bytes(),
            PublicKey::BlsShare(pub_key) => pub_key.to_bytes(),
        };
        let mut xor_name = XorName::random();
        xor_name.0.clone_from_slice(&bytes[..XOR_NAME_LEN]);
        xor_name
    }
}

impl From<ed25519_dalek::PublicKey> for PublicKey {
    fn from(public_key: ed25519_dalek::PublicKey) -> Self {
        Self::Ed25519(public_key)
    }
}

impl From<threshold_crypto::PublicKey> for PublicKey {
    fn from(public_key: threshold_crypto::PublicKey) -> Self {
        Self::Bls(public_key)
    }
}

impl From<threshold_crypto::PublicKeyShare> for PublicKey {
    fn from(public_key: threshold_crypto::PublicKeyShare) -> Self {
        Self::BlsShare(public_key)
    }
}

impl From<&Keypair> for PublicKey {
    fn from(keypair: &Keypair) -> Self {
        keypair.public_key()
    }
}

impl Debug for PublicKey {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "PublicKey::")?;
        match self {
            Self::Ed25519(pub_key) => {
                write!(
                    formatter,
                    "Ed25519({:<8})",
                    hex::encode(&pub_key.to_bytes())
                )
            }
            Self::Bls(pub_key) => write!(
                formatter,
                "Bls({:<8})",
                hex::encode(&pub_key.to_bytes()[..XOR_NAME_LEN])
            ),
            Self::BlsShare(pub_key) => write!(
                formatter,
                "BlsShare({:<8})",
                hex::encode(&pub_key.to_bytes()[..XOR_NAME_LEN])
            ),
        }
    }
}

impl Display for PublicKey {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Debug::fmt(self, formatter)
    }
}

impl LowerHex for PublicKey {
    /// Useful for displaying public key in user-facing apps
    /// E.g. in cli and in human-readable messaging like for sn_authd
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let bytes = self.clone().to_bytes();
        write!(f, "{}", hex::encode(bytes))
    }
}

impl UpperHex for PublicKey {
    /// Useful for displaying public key in user-facing apps
    /// E.g. in cli and in human-readable messaging like for sn_authd
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let bytes = self.clone().to_bytes();
        write!(f, "{}", hex::encode_upper(bytes))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::utils;
    use threshold_crypto::{self};

    fn gen_keypairs() -> Vec<Keypair> {
        let mut rng = rand::thread_rng();
        let bls_secret_key = threshold_crypto::SecretKeySet::random(1, &mut rng);
        vec![
            Keypair::new_ed25519(&mut rng),
            Keypair::new_bls_share(
                0,
                bls_secret_key.secret_key_share(0),
                bls_secret_key.public_keys(),
            ),
        ]
    }

    pub fn gen_keys() -> Vec<PublicKey> {
        gen_keypairs().iter().map(PublicKey::from).collect()
    }

    #[test]
    fn zbase32_encode_decode_public_key() -> Result<()> {
        let keys = gen_keys();

        for key in keys {
            assert_eq!(
                key,
                PublicKey::decode_from_zbase32(&key.encode_to_zbase32()?)?
            );
        }

        Ok(())
    }

    // Test serialising and deserialising public keys.
    #[test]
    fn serialisation_public_key() -> Result<()> {
        let keys = gen_keys();

        for key in keys {
            let encoded = utils::serialise(&key)?;
            let decoded: PublicKey = utils::deserialise(&encoded)?;

            assert_eq!(decoded, key);
        }

        Ok(())
    }
}
