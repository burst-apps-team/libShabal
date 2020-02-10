use std::ptr::{null_mut};
use sha2::{Sha256, Digest};
use std::intrinsics::copy_nonoverlapping;

extern "C" {
    pub fn curve25519_c_keygen(p: *mut u8, s: *mut u8, k: *mut u8);
    pub fn curve25519_c_sign(v: *mut u8, h: *mut u8, x: *mut u8, s: *mut u8);
    pub fn curve25519_c_verify(y: *mut u8, v: *mut u8, h: *mut u8, p: *mut u8);
    pub fn curve25519_c_isCanonicalSignature(signature: *const u8) -> u8;
    pub fn curve25519_c_isCanonicalPublicKey(public_key: *const u8) -> u8;
}

pub fn get_public_key(private_key: &mut [u8], public_key_buffer: &mut [u8]) {
    unsafe {
        curve25519_c_keygen(public_key_buffer.as_mut_ptr(), null_mut(), private_key.as_mut_ptr());
    }
}

pub extern fn get_shared_secret(private_key: &mut [u8], public_key: &mut [u8], shared_secret_buffer: &mut [u8]) {
    unsafe {
        curve25519_c_keygen(shared_secret_buffer.as_mut_ptr(), private_key.as_mut_ptr(), public_key.as_mut_ptr())
    }
}

pub extern fn sign(private_key: &mut [u8], message_sha256: &[u8], signature_buffer: &mut [u8]) {
    unsafe {
        let mut public_key: [u8; 32] = [0; 32];
        let mut shared_key: [u8; 32] = [0; 32];
        curve25519_c_keygen(public_key.as_mut_ptr(), shared_key.as_mut_ptr(), private_key.as_mut_ptr());

        let mut sha256 = Sha256::new();
        sha256.input(message_sha256);
        sha256.input(shared_key);
        let mut x = sha256.result_reset();

        let mut y: [u8; 32] = [0; 32];
        curve25519_c_keygen(y.as_mut_ptr(), null_mut(), (&mut x).as_mut_ptr());

        sha256.input(message_sha256);
        sha256.input(y);
        let mut h = sha256.result();

        curve25519_c_sign(signature_buffer.as_mut_ptr(), h.as_mut_ptr(), x.as_mut_ptr(), shared_key.as_mut_ptr());
        copy_nonoverlapping(h.as_ptr(), signature_buffer.as_mut_ptr().add(32), 32);
    }
}

pub extern fn verify(public_key: &mut [u8], signature: &[u8], message_sha256: &[u8], enforce_canonical: bool) -> bool {
    unsafe {
        if enforce_canonical {
            if curve25519_c_isCanonicalPublicKey(public_key.as_ptr()) == 0 { return false; }
            if curve25519_c_isCanonicalSignature(signature.as_ptr()) == 0 { return false; }
        }

        let mut y: [u8; 32] = [0; 32];
        let mut v: [u8; 32] = [0; 32];
        let mut h: [u8; 32] = [0; 32];
        copy_nonoverlapping(signature.as_ptr(), v.as_mut_ptr(), 32);
        copy_nonoverlapping(signature.as_ptr().add(32), h.as_mut_ptr(), 32);
        curve25519_c_verify(y.as_mut_ptr(), v.as_mut_ptr(), h.as_mut_ptr(), public_key.as_mut_ptr());

        let mut sha256 = Sha256::new();
        sha256.input(message_sha256);
        sha256.input(y);
        let h2 = sha256.result();

        return eq(&h, &h2);
    }
}

fn eq(slice1: &[u8], slice2: &[u8]) -> bool {
    return slice1 == slice2;
}
