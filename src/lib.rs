extern crate libc;
#[macro_use]
extern crate cfg_if;
use std::sync::{Once};
use std::{u64, slice};
use shabal::{Shabal256, Digest};
use std::os::raw::c_void;
use pocc::plot::NONCE_SIZE;

mod pocc;

extern "C" {
    pub fn find_best_deadline_sph(
        scoops: *const u8,
        nonce_count: u64,
        gensig: *const u8,
        best_deadline: *mut u64,
        best_offset: *mut u64,
    ) -> ();
}

cfg_if! {
    if #[cfg(feature = "simd")] {
        extern "C" {
            pub fn init_shabal_avx512f() -> ();
            pub fn find_best_deadline_avx512f(
                scoops: *const u8,
                nonce_count: u64,
                gensig: *const u8,
                best_deadline: *mut u64,
                best_offset: *mut u64,
            ) -> ();

            pub fn init_shabal_avx2() -> ();
            pub fn find_best_deadline_avx2(
                scoops: *const u8,
                nonce_count: u64,
                gensig: *const u8,
                best_deadline: *mut u64,
                best_offset: *mut u64,
            ) -> ();

            pub fn init_shabal_avx() -> ();
            pub fn find_best_deadline_avx(
                scoops: *const u8,
                nonce_count: u64,
                gensig: *const u8,
                best_deadline: *mut u64,
                best_offset: *mut u64,
            ) -> ();

            pub fn init_shabal_sse2() -> ();
            pub fn find_best_deadline_sse2(
                scoops: *const u8,
                nonce_count: u64,
                gensig: *const u8,
                best_deadline: *mut u64,
                best_offset: *mut u64,
            ) -> ();

            pub fn init_noncegen_sse2() -> ();
            pub fn noncegen_sse2(
                cache: *mut u8,
                cache_size: usize,
                chunk_offset: usize,
                numeric_ID: u64,
                local_startnonce: u64,
                local_nonces: u64,
            );

            pub fn init_noncegen_avx() -> ();
            pub fn noncegen_avx(
                cache: *mut u8,
                cache_size: usize,
                chunk_offset: usize,
                numeric_ID: u64,
                local_startnonce: u64,
                local_nonces: u64,
            );

            pub fn init_noncegen_avx2() -> ();
            pub fn noncegen_avx2(
                cache: *mut u8,
                cache_size: usize,
                chunk_offset: usize,
                numeric_ID: u64,
                local_startnonce: u64,
                local_nonces: u64,
            );

            pub fn init_noncegen_avx512f() -> ();
            pub fn noncegen_avx512(
                cache: *mut u8,
                cache_size: usize,
                chunk_offset: usize,
                numeric_ID: u64,
                local_startnonce: u64,
                local_nonces: u64,
            );
        }
    }
}

cfg_if! {
    if #[cfg(feature = "neon")] {
        extern "C" {
            pub fn init_shabal_neon() -> ();
            pub fn find_best_deadline_neon(
                scoops: *const u8,
                nonce_count: u64,
                gensig: *const u8,
                best_deadline: *mut u64,
                best_offset: *mut u64,
            ) -> ();
        }
    }
}

#[no_mangle]
pub extern fn shabal_findBestDeadlineDirect(
    scoops: *const u8,
    nonce_count: u64,
    gensig: *const u8,
    best_deadline: *mut u64,
    best_offset: *mut u64,
) {
    #[cfg(feature = "simd")]
        unsafe {
        // TODO don't check on the fly...
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
                init_noncegen_avx512f();
            } else if is_x86_feature_detected!("avx2") {
                init_shabal_avx2();
                init_noncegen_avx2();
            } else if is_x86_feature_detected!("avx") {
                init_shabal_avx();
                init_noncegen_avx();
            } else if is_x86_feature_detected!("sse2") {
                init_shabal_sse2();
                init_noncegen_sse2();
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
    scoops: *const u8,
    nonce_count: u64,
    gensig: *const u8,
) -> u64 {
    let mut deadline: u64 = u64::MAX;
    let mut offset: u64 = 0;
    shabal_findBestDeadlineDirect(scoops, nonce_count, gensig, &mut deadline, &mut offset);
    return offset;
}

/// Create a new Shabal256 instance
///
/// Returns a pointer to the instance, which
/// can be used with the other functions to
/// manipulate the instance.
#[no_mangle]
pub extern fn shabal256_new() -> *mut c_void {
    return Box::into_raw(Box::new(Shabal256::new())) as *mut c_void;
}

/// Destroy a Shabal256 instance, clearing memory allocated for it.
///
/// `shabal` is the pointer to the instance returned from `shabal256_new()`
#[no_mangle]
pub extern fn shabal256_destroy(shabal: *mut c_void) {
    if !shabal.is_null() {
        unsafe {
            // Let it fall out of scope naturally once it is unboxed
            Box::from_raw(shabal as *mut Shabal256);
        }
    }
}

/// Reset a Shabal256 instance to its initial state
///
/// `shabal` is the pointer to the instance returned from `shabal256_new()`
#[no_mangle]
pub extern fn shabal256_reset(shabal: *mut c_void) {
    if !shabal.is_null() {
        unsafe {
            let shabal_borrowed = &mut *(shabal as *mut Shabal256);
            shabal_borrowed.reset();
        }
    }
}

/// Update a Shabal256 instance with new input data
///
/// `shabal` is the pointer to the instance returned from `shabal256_new()`
///
/// Inputs data into the digest from `data` starting at `offset` of length `len`
#[no_mangle]
pub extern fn shabal256_update(shabal: *mut c_void, data: *const u8, offset: usize, len: usize) {
    if !shabal.is_null() {
        unsafe {
            let array = slice::from_raw_parts(data.add(offset), len as usize);
            let shabal_borrowed = &mut *(shabal as *mut Shabal256);
            shabal_borrowed.input(array);
        }
    }
}

/// Retrieve the result of a Shabal256 digest and reset the digest.
///
/// Stores the data in `buffer` starting from `offset`. Stores 32 bytes of hash data.
///
/// `buffer` must have 32 bytes available from `offset` otherwise this will attempt to write beyond the array.
#[no_mangle]
pub extern fn shabal256_digest(shabal: *mut c_void, buffer: *mut u8, offset: usize) {
    if !shabal.is_null() {
        unsafe {
            let array = slice::from_raw_parts_mut(buffer.add(offset), 32);
            let shabal_borrowed = &mut *(shabal as *mut Shabal256);
            array.copy_from_slice(shabal_borrowed.result_reset().as_slice());
        }
    }
}

// TODO steal engraver's fast rust shabal

/// Creates PoC Nonces, with SIMD instructions for extra speed.
///
/// `plot_buffer` must be correct size - no size checks are performed.
#[no_mangle]
pub extern fn create_plots(
    account_id: u64,
    start_nonce: u64,
    nonce_count: u64,
    poc_version: u8,
    plot_buffer: *mut u8,
    plot_size: usize,
) {
    #[cfg(feature = "simd")]
    unsafe {
        // TODO don't check on the fly...
        // TODO poc1/2 switching
        if is_x86_feature_detected!("avx512f") {
            noncegen_avx512(
                plot_buffer,
                plot_size / NONCE_SIZE,
                0,
                account_id,
                start_nonce,
                nonce_count
            );
        } else if is_x86_feature_detected!("avx2") {
            noncegen_avx2(
                plot_buffer,
                plot_size / NONCE_SIZE,
                0,
                account_id,
                start_nonce,
                nonce_count
            );
        } else if is_x86_feature_detected!("avx") {
            noncegen_avx(
                plot_buffer,
                plot_size / NONCE_SIZE,
                0,
                account_id,
                start_nonce,
                nonce_count
            );
        } else if is_x86_feature_detected!("sse2") {
            noncegen_sse2(
                plot_buffer,
                plot_size / NONCE_SIZE,
                0,
                account_id,
                start_nonce,
                nonce_count
            );
        } else {
            pocc::plot::noncegen_rust(
                slice::from_raw_parts_mut(plot_buffer, plot_size as usize),
                0,
                account_id,
                start_nonce,
                nonce_count,
            );
        }
    }
    #[cfg(not(feature = "simd"))]
    unsafe {
        pocc::plot::noncegen_rust(
            slice::from_raw_parts_mut(plot_buffer, plot_size as usize),
            0,
            account_id,
            start_nonce,
            nonce_count,
        );
    }
}
