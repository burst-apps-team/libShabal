use crate::curve25519;
use std::ptr::null;

pub fn get_public_key(private_key: &[u8], public_key_buffer: &mut [u8]) {
    curve25519::keygen(public_key_buffer, None, private_key);
}

pub extern fn get_shared_secret(private_key: &mut [u8], public_key: &[u8], shared_secret_buffer: &mut [u8]) {
    curve25519::keygen(shared_secret_buffer, Some(private_key), public_key)
}

pub extern fn sign(private_key: &[u8], message_sha256: &[u8], signature_buffer: &mut [u8]) {
    // TODO
}

pub extern fn verify(public_key: &[u8], signature: &[u8], message_sha256: &[u8], enforce_canonical: bool) -> bool {
    // TODO
    return false;
}