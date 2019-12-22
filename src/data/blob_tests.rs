// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::super::{utils, PublicKey, XorName};
use crate::data::{BlobAddress, PrivateBlob, PublicBlob};
use bincode::deserialize as deserialise;
use hex::encode;
use rand::{self, Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::{env, iter, thread};
use threshold_crypto::SecretKey;
use unwrap::unwrap;

#[test]
fn deterministic_name() {
    let data1 = b"Hello".to_vec();
    let data2 = b"Goodbye".to_vec();

    let owner1 = PublicKey::Bls(SecretKey::random().public_key());
    let owner2 = PublicKey::Bls(SecretKey::random().public_key());

    let idata1 = PrivateBlob::new(data1.clone(), owner1);
    let idata2 = PrivateBlob::new(data1, owner2);
    let idata3 = PrivateBlob::new(data2.clone(), owner1);
    let idata3_clone = PrivateBlob::new(data2, owner1);

    assert_eq!(idata3, idata3_clone);

    assert_ne!(idata1.name(), idata2.name());
    assert_ne!(idata1.name(), idata3.name());
    assert_ne!(idata2.name(), idata3.name());
}

#[test]
fn deterministic_test() {
    let value = "immutable data value".to_owned().into_bytes();
    let blob = PublicBlob::new(value);
    let blob_name = encode(blob.name().0.as_ref());
    let expected_name = "fac2869677ee06277633c37ac7e8e5c655f3d652f707c7a79fab930d584a3016";

    assert_eq!(&expected_name, &blob_name);
}

#[test]
fn serialisation() {
    let mut rng = get_rng();
    let len = rng.gen_range(1, 10_000);
    let value = iter::repeat_with(|| rng.gen()).take(len).collect();
    let blob = PublicBlob::new(value);
    let serialised = utils::serialise(&blob);
    let parsed = unwrap!(deserialise(&serialised));
    assert_eq!(blob, parsed);
}

fn get_rng() -> XorShiftRng {
    let env_var_name = "RANDOM_SEED";
    let seed = env::var(env_var_name)
        .ok()
        .map(|value| {
            unwrap!(
                value.parse::<u64>(),
                "Env var 'RANDOM_SEED={}' is not a valid u64.",
                value
            )
        })
        .unwrap_or_else(rand::random);
    println!(
        "To replay this '{}', set env var {}={}",
        unwrap!(thread::current().name()),
        env_var_name,
        seed
    );
    XorShiftRng::seed_from_u64(seed)
}

#[test]
fn zbase32_encode_decode_blob_address() {
    let name = XorName(rand::random());
    let address = BlobAddress::Public(name);
    let encoded = address.encode_to_zbase32();
    let decoded = unwrap!(BlobAddress::decode_from_zbase32(&encoded));
    assert_eq!(address, decoded);
}
