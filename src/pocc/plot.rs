use crate::pocc::shabal256_fast::shabal256_fast;
use std::ptr::copy_nonoverlapping;

const HASH_SIZE: usize = 32;
const HASH_CAP: usize = 4096;
const NUM_SCOOPS: usize = 4096;
pub const SCOOP_SIZE: usize = 64;
pub const NONCE_SIZE: usize = (NUM_SCOOPS * SCOOP_SIZE);
const MESSAGE_SIZE: usize = 16;

// cache:		    cache to save to
// local_num:		thread number
// numeric_id:		numeric account id
// loc_startnonce	nonce to start generation at
// local_nonces: 	number of nonces to generate (count from 0)
pub fn noncegen_rust(
    cache: &mut [u8],
    numeric_id: u64,
    local_startnonce: u64,
    local_nonces: u64,
    poc_version: u8,
) {
    let numeric_id: [u32; 2] = unsafe { std::mem::transmute(numeric_id.to_be()) };

    let mut final_buffer = [0u8; HASH_SIZE];

    // prepare termination strings
    let mut t1 = [0u32; MESSAGE_SIZE];
    t1[0..2].copy_from_slice(&numeric_id);
    t1[4] = 0x80;

    let mut t2 = [0u32; MESSAGE_SIZE];
    t2[8..10].copy_from_slice(&numeric_id);
    t2[12] = 0x80;

    let mut t3 = [0u32; MESSAGE_SIZE];
    t3[0] = 0x80;

    let mut hash_buffer = [0u8; HASH_SIZE];

    for n in 0..local_nonces {
        let offset = n as usize * NONCE_SIZE;
        let buffer = &mut cache[offset..offset + NONCE_SIZE];

        // generate nonce numbers & change endianness
        let nonce: [u32; 2] = unsafe { std::mem::transmute((local_startnonce + n).to_be()) };

        // store nonce numbers in relevant termination strings
        t1[2..4].copy_from_slice(&nonce);
        t2[10..12].copy_from_slice(&nonce);

        // start shabal rounds

        // 3 cases: first 128 rounds uses case 1 or 2, after that case 3
        // case 1: first 128 rounds, hashes are even: use termination string 1
        // case 2: first 128 rounds, hashes are odd: use termination string 2
        // case 3: round > 128: use termination string 3
        // round 1
        let hash = shabal256_fast(&[], &t1);

        buffer[NONCE_SIZE - HASH_SIZE..NONCE_SIZE].copy_from_slice(&hash);
        let hash = unsafe { std::mem::transmute::<[u8; 32], [u32; 8]>(hash) };

        // store first hash into smart termination string 2
        t2[0..8].copy_from_slice(&hash);
        // round 2 - 128
        for i in (NONCE_SIZE - HASH_CAP + HASH_SIZE..=NONCE_SIZE - HASH_SIZE).rev().step_by(HASH_SIZE) {
            // check if msg can be divided into 512bit packages without a
            // remainder
            if i % 64 == 0 {
                // last msg = seed + termination
                let hash = &shabal256_fast(&buffer[i..NONCE_SIZE], &t1);
                buffer[i - HASH_SIZE..i].copy_from_slice(hash);
            } else {
                // last msg = 256 bit data + seed + termination
                let hash = &shabal256_fast(&buffer[i..NONCE_SIZE], &t2);
                buffer[i - HASH_SIZE..i].copy_from_slice(hash);
            }
        }

        // round 128-8192
        for i in (HASH_SIZE..=NONCE_SIZE - HASH_CAP).rev().step_by(HASH_SIZE) {
            let hash = &shabal256_fast(&buffer[i..i + HASH_CAP], &t3);
            buffer[i - HASH_SIZE..i].copy_from_slice(hash);
        }

        // generate final hash
        final_buffer.copy_from_slice(&shabal256_fast(&buffer[0..NONCE_SIZE], &t1));

        // XOR with final
        for i in 0..NONCE_SIZE {
            buffer[i] ^= final_buffer[i % HASH_SIZE];
        }

        // PoC2 shuffle
        if poc_version == 2 {
            unsafe {
                let mut rev_pos = NONCE_SIZE - HASH_SIZE;
                for pos in (32..NONCE_SIZE / 2).step_by(64) {
                    let hash_buffer_ptr = hash_buffer.as_mut_ptr();
                    let buffer_pos = buffer.as_mut_ptr().add(pos);
                    let buffer_rev_pos = buffer.as_mut_ptr().add(rev_pos);
                    copy_nonoverlapping(buffer_pos, hash_buffer_ptr, HASH_SIZE);
                    copy_nonoverlapping(buffer_rev_pos, buffer_pos, HASH_SIZE);
                    copy_nonoverlapping(hash_buffer_ptr, buffer_rev_pos, HASH_SIZE);
                    rev_pos -= 64;
                }
            }
        }
    }
}

pub fn noncegen_single_rust(
    cache: &mut [u8],
    numeric_id: u64,
    nonce: u64,
    poc_version: u8,
) {
    let numeric_id: [u32; 2] = unsafe { std::mem::transmute(numeric_id.to_be()) };

    let mut final_buffer = [0u8; HASH_SIZE];

    // prepare termination strings
    let mut t1 = [0u32; MESSAGE_SIZE];
    t1[0..2].copy_from_slice(&numeric_id);
    t1[4] = 0x80;

    let mut t2 = [0u32; MESSAGE_SIZE];
    t2[8..10].copy_from_slice(&numeric_id);
    t2[12] = 0x80;

    let mut t3 = [0u32; MESSAGE_SIZE];
    t3[0] = 0x80;

    // generate nonce numbers & change endianness
    let nonce: [u32; 2] = unsafe { std::mem::transmute(nonce.to_be()) };

    // store nonce numbers in relevant termination strings
    t1[2..4].copy_from_slice(&nonce);
    t2[10..12].copy_from_slice(&nonce);

    // start shabal rounds

    // 3 cases: first 128 rounds uses case 1 or 2, after that case 3
    // case 1: first 128 rounds, hashes are even: use termination string 1
    // case 2: first 128 rounds, hashes are odd: use termination string 2
    // case 3: round > 128: use termination string 3
    // round 1
    let hash = shabal256_fast(&[], &t1);

    cache[NONCE_SIZE - HASH_SIZE..NONCE_SIZE].copy_from_slice(&hash);
    let hash = unsafe { std::mem::transmute::<[u8; 32], [u32; 8]>(hash) };

    // store first hash into smart termination string 2
    t2[0..8].copy_from_slice(&hash);
    // round 2 - 128
    for i in (NONCE_SIZE - HASH_CAP + HASH_SIZE..=NONCE_SIZE - HASH_SIZE)
        .rev()
        .step_by(HASH_SIZE)
        {
            // check if msg can be divided into 512bit packages without a
            // remainder
            if i % 64 == 0 {
                // last msg = seed + termination
                let hash = &shabal256_fast(&cache[i..NONCE_SIZE], &t1);
                cache[i - HASH_SIZE..i].copy_from_slice(hash);
            } else {
                // last msg = 256 bit data + seed + termination
                let hash = &shabal256_fast(&cache[i..NONCE_SIZE], &t2);
                cache[i - HASH_SIZE..i].copy_from_slice(hash);
            }
        }

    // round 128-8192
    for i in (HASH_SIZE..=NONCE_SIZE - HASH_CAP).rev().step_by(HASH_SIZE) {
        let hash = &shabal256_fast(&cache[i..i + HASH_CAP], &t3);
        cache[i - HASH_SIZE..i].copy_from_slice(hash);
    }

    // generate final hash
    final_buffer.copy_from_slice(&shabal256_fast(&cache[0..NONCE_SIZE], &t1));

    // XOR with final
    for i in 0..NONCE_SIZE {
        cache[i] ^= final_buffer[i % HASH_SIZE];
    }

    // PoC2 shuffle
    if poc_version == 2 {
        unsafe {
            let mut hash_buffer = [0u8; HASH_SIZE];
            let mut rev_pos = NONCE_SIZE - HASH_SIZE;
            for pos in (32..NONCE_SIZE / 2).step_by(64) {
                let hash_buffer_ptr = hash_buffer.as_mut_ptr();
                let cache_pos = cache.as_mut_ptr().add(pos);
                let cache_rev_pos = cache.as_mut_ptr().add(rev_pos);
                copy_nonoverlapping(cache_pos, hash_buffer_ptr, HASH_SIZE);
                copy_nonoverlapping(cache_rev_pos, cache_pos, HASH_SIZE);
                copy_nonoverlapping(hash_buffer_ptr, cache_rev_pos, HASH_SIZE);
                rev_pos -= 64;
            }
        }
    }
}
