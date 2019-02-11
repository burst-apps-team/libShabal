extern crate libc;
use libc::{c_char, uint64_t};
#[macro_use]
extern crate cfg_if;

use std::u64;

extern "C" {
    pub fn find_best_deadline_sph(
        scoops: *const c_char,
        nonce_count: uint64_t,
        gensig: *const c_char,
        best_deadline: *mut uint64_t,
        best_offset: *mut uint64_t,
    ) -> ();
}

cfg_if! {
    if #[cfg(feature = "simd")] {
        extern "C" {
            pub fn init_shabal_avx512f() -> ();
            pub fn find_best_deadline_avx512f(
                scoops: *const c_char,
                nonce_count: uint64_t,
                gensig: *const c_char,
                best_deadline: *mut uint64_t,
                best_offset: *mut uint64_t,
            ) -> ();

            pub fn init_shabal_avx2() -> ();
            pub fn find_best_deadline_avx2(
                scoops: *const c_char,
                nonce_count: uint64_t,
                gensig: *const c_char,
                best_deadline: *mut uint64_t,
                best_offset: *mut uint64_t,
            ) -> ();

            pub fn init_shabal_avx() -> ();
            pub fn find_best_deadline_avx(
                scoops: *const c_char,
                nonce_count: uint64_t,
                gensig: *const c_char,
                best_deadline: *mut uint64_t,
                best_offset: *mut uint64_t,
            ) -> ();

            pub fn init_shabal_sse2() -> ();
            pub fn find_best_deadline_sse2(
                scoops: *const c_char,
                nonce_count: uint64_t,
                gensig: *const c_char,
                best_deadline: *mut uint64_t,
                best_offset: *mut uint64_t,
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
                nonce_count: uint64_t,
                gensig: *const c_char,
                best_deadline: *mut uint64_t,
                best_offset: *mut uint64_t,
            ) -> ();
        }
    }
}

#[no_mangle]
pub extern fn shabal_findBestDeadlineDirect(
    scoops: *const c_char,
    nonce_count: uint64_t,
    gensig: *const c_char,
    best_deadline: *mut uint64_t,
    best_offset: *mut uint64_t,
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
    nonce_count: uint64_t,
    gensig: *const c_char,
) -> uint64_t {
    let mut deadline: u64 = u64::MAX;
    let mut offset: u64 = 0;
    shabal_findBestDeadlineDirect(scoops, nonce_count, gensig, &mut deadline, &mut offset);
    return offset;
}
