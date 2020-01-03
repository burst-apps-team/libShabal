extern crate libc;
#[macro_use]
extern crate cfg_if;
use libc::{c_char};
use shabal::{Shabal256, Digest};
use std::u64;

extern "C" {
    pub fn find_best_deadline_sph(
        scoops: *const c_char,
        nonce_count: u64,
        gensig: *const c_char,
        best_deadline: *mut u64,
        best_offset: *mut u64,
    ) -> ();
}

cfg_if! {
    if #[cfg(feature = "simd")] {
        extern "C" {
            pub fn init_shabal_avx512f() -> ();
            pub fn find_best_deadline_avx512f(
                scoops: *const c_char,
                nonce_count: u64,
                gensig: *const c_char,
                best_deadline: *mut u64,
                best_offset: *mut u64,
            ) -> ();

            pub fn init_shabal_avx2() -> ();
            pub fn find_best_deadline_avx2(
                scoops: *const c_char,
                nonce_count: u64,
                gensig: *const c_char,
                best_deadline: *mut u64,
                best_offset: *mut u64,
            ) -> ();

            pub fn init_shabal_avx() -> ();
            pub fn find_best_deadline_avx(
                scoops: *const c_char,
                nonce_count: u64,
                gensig: *const c_char,
                best_deadline: *mut u64,
                best_offset: *mut u64,
            ) -> ();

            pub fn init_shabal_sse2() -> ();
            pub fn find_best_deadline_sse2(
                scoops: *const c_char,
                nonce_count: u64,
                gensig: *const c_char,
                best_deadline: *mut u64,
                best_offset: *mut u64,
            ) -> ();
        }
    }
}

cfg_if! {
    if #[cfg(feature = "neon")] {
        extern "C" {
            pub fn init_shabal_neon() -> ();
            pub fn find_best_deadline_neon(
                scoops: *const c_char,
                nonce_count: u64,
                gensig: *const c_char,
                best_deadline: *mut u64,
                best_offset: *mut u64,
            ) -> ();
        }
    }
}

#[no_mangle]
pub extern fn shabal_findBestDeadlineDirect(
    scoops: *const c_char,
    nonce_count: u64,
    gensig: *const c_char,
    best_deadline: *mut u64,
    best_offset: *mut u64,
) {
    #[cfg(feature = "simd")]
        unsafe {
        if is_x86_feature_detected!("avx512f") {
            find_best_deadline_avx512f(
                scoops,
                nonce_count,
                gensig,
                best_deadline,
                best_offset,
            );
        } else if is_x86_feature_detected!("avx2") {
            find_best_deadline_avx2(
                scoops,
                nonce_count,
                gensig,
                best_deadline,
                best_offset,
            );
        } else if is_x86_feature_detected!("avx") {
            find_best_deadline_avx(
                scoops,
                nonce_count,
                gensig,
                best_deadline,
                best_offset,
            );
        } else if is_x86_feature_detected!("sse2") {
            find_best_deadline_sse2(
                scoops,
                nonce_count,
                gensig,
                best_deadline,
                best_offset,
            );
        } else {
            find_best_deadline_sph(
                scoops,
                nonce_count,
                gensig,
                best_deadline,
                best_offset,
            );
        }
    }
    #[cfg(feature = "neon")]
        unsafe {
        #[cfg(target_arch = "arm")]
        let neon = is_arm_feature_detected!("neon");
        #[cfg(target_arch = "aarch64")]
        let neon = true;
        if neon {
            find_best_deadline_neon(
                scoops,
                nonce_count,
                gensig,
                best_deadline,
                best_offset,
            );
        } else {
            find_best_deadline_sph(
                scoops,
                nonce_count,
                gensig,
                best_deadline,
                best_offset,
            );
        }
    }
    #[cfg(not(any(feature = "simd", feature = "neon")))]
        unsafe {
        find_best_deadline_sph(
            scoops,
            nonce_count,
            gensig,
            best_deadline,
            best_offset,
        );
    }
}

#[no_mangle]
pub extern fn shabal_init() {
    #[cfg(feature = "simd")]
    unsafe {
        if is_x86_feature_detected!("avx512f") {
            init_shabal_avx512f();
        } else if is_x86_feature_detected!("avx2") {
            init_shabal_avx2();
        } else if is_x86_feature_detected!("avx") {
            init_shabal_avx();
        } else if is_x86_feature_detected!("sse2") {
            init_shabal_sse2();
        }
    }
    #[cfg(feature = "neon")]
    unsafe {
        #[cfg(target_arch = "arm")]
        let neon = is_arm_feature_detected!("neon");
        #[cfg(target_arch = "aarch64")]
        let neon = true;

        if neon {
            init_shabal_neon();
        }
    }
}

#[no_mangle]
pub extern fn shabal_findBestDeadline(
    scoops: *const c_char,
    nonce_count: u64,
    gensig: *const c_char,
) -> u64 {
    let mut deadline: u64 = u64::MAX;
    let mut offset: u64 = 0;
    shabal_findBestDeadlineDirect(scoops, nonce_count, gensig, &mut deadline, &mut offset);
    return offset;
}

#[no_mangle]
pub extern fn shabal_new() {
    let data = "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".as_bytes();
    let mut hasher = Shabal256::new();
    hasher.input(data);
    let _result = hasher.result();
}

#[cfg(test)]
mod tests {
    use super::*;
    use shabal_new;
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn time() -> u128 {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        return since_the_epoch.as_millis();
    }

    #[test]
    pub fn test() {
        let start = time();
        for _i in 0..2048 {
            shabal_new()
        }
        let end = time();
        let duration = end - start;
        println!("Duration: {} ms", duration);
    }
}
