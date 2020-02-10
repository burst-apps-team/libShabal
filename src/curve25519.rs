pub fn get_public_key(private_key: &[u8], public_key_buffer: &mut [u8]) {
    // TODO
}

pub extern fn get_shared_secret(private_key: &[u8], public_key: &[u8], shared_secret_buffer: &mut [u8]) {
    // TODO
}

pub extern fn sign(private_key: &[u8], message_sha256: &[u8], signature_buffer: &mut [u8]) {
    // TODO
}

pub extern fn verify(public_key: &[u8], signature: &[u8], message_sha256: &[u8], enforce_canonical: bool) -> bool {
    // TODO
    return false;
}