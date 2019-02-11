use libc::{c_void, uint64_t};
#[macro_use]
extern crate cfg_if;

use std::u64;

extern "C" {
    pub fn find_best_deadline_sph(
        scoops: *mut c_void,
        nonce_count: uint64_t,
        gensig: *const c_void,
        best_deadline: *mut uint64_t,
        best_offset: *mut uint64_t,
    ) -> ();
}

cfg_if! {
    if #[cfg(feature = "simd")] {
        extern "C" {
            pub fn find_best_deadline_avx512f(
                scoops: *mut c_void,
                nonce_count: uint64_t,
                gensig: *const c_void,
                best_deadline: *mut uint64_t,
                best_offset: *mut uint64_t,
            ) -> ();

            pub fn find_best_deadline_avx2(
                scoops: *mut c_void,
                nonce_count: uint64_t,
                gensig: *const c_void,
                best_deadline: *mut uint64_t,
                best_offset: *mut uint64_t,
            ) -> ();

            pub fn find_best_deadline_avx(
                scoops: *mut c_void,
                nonce_count: uint64_t,
                gensig: *const c_void,
                best_deadline: *mut uint64_t,
                best_offset: *mut uint64_t,
            ) -> ();

            pub fn find_best_deadline_sse2(
                scoops: *mut c_void,
                nonce_count: uint64_t,
                gensig: *const c_void,
                best_deadline: *mut uint64_t,
                best_offset: *mut uint64_t,
            ) -> ();
        }
    }
}

cfg_if! {
    if #[cfg(feature = "neon")] {
        extern "C" {
            pub fn find_best_deadline_neon(
                scoops: *mut c_void,
                nonce_count: uint64_t,
                gensig: *const c_void,
                best_deadline: *mut uint64_t,
                best_offset: *mut uint64_t,
            ) -> ();
        }
    }
}

#[no_mangle]
pub extern fn find_best_deadline(
    scoops: *mut c_void,
    nonce_count: uint64_t,
    gensig: *const c_void,
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
pub extern fn find_best_deadline_assisted(
    scoops: *mut c_void,
    nonce_count: uint64_t,
    gensig: *const c_void,
) -> uint64_t {
    let mut deadline: u64 = u64::MAX;
    let mut offset: u64 = 0;
    find_best_deadline(scoops, nonce_count, gensig, &mut deadline, &mut offset);
    println!("scoop length is {}, best deadline is {}, best offset is {}", deadline, offset);
    return offset;
}
