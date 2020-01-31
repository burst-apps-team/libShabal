extern crate libc;
#[macro_use]
extern crate cfg_if;
use std::sync::{Once};
use libc::{c_char};
use std::{u64, slice};
use shabal::{Shabal256, Digest};

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
    static INITIALIZE: Once = Once::new();
    INITIALIZE.call_once(|| {
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
    });
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
pub extern fn shabal256_new() -> *mut Shabal256 {
    let mut shabal = Shabal256::new();
    return Box::into_raw(Box::new(shabal));
}

#[no_mangle]
pub extern fn shabal256_destroy(shabal: *mut Shabal256) {
    if !shabal.is_null() {
        unsafe {
            // Let it fall out of scope naturally once it is unboxed
            Box::from_raw(shabal);
        }
    }
}

#[no_mangle]
pub extern fn shabal256_update(shabal: *mut Shabal256, data: *const u8, data_len: usize) {
    if !shabal.is_null() {
        unsafe {
            let array = slice::from_raw_parts(data, data_len as usize);
            let mut shabal_borrowed = &mut *shabal;
            shabal_borrowed.input(array);
        }
    }
}

/// Buffer must have 32 bytes available from the offset
#[no_mangle]
pub extern fn shabal256_digest(shabal: *mut Shabal256, buffer: *mut u8, buffer_len: usize, offset: usize) {
    if !shabal.is_null() {
        unsafe {
            let array = slice::from_raw_parts_mut(buffer, buffer_len as usize);
            let mut shabal_borrowed = &mut *shabal;
            let result = shabal_borrowed.result_reset();
            for i in 0..31 {
                array[offset + i] = result[i];
            }
        }
    }
}
