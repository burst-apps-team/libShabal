extern crate libc;
#[macro_use]
extern crate cfg_if;
use std::sync::{Once};
use std::{u64, slice};
use shabal::{Shabal256, Digest};
use std::os::raw::c_void;
use pocc::plot::NONCE_SIZE;
use pocc::plot::SCOOP_SIZE;
use crate::simd::SimdExtension;

mod pocc;
mod simd;

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
                poc_version: u8,
            );

            pub fn init_noncegen_avx() -> ();
            pub fn noncegen_avx(
                cache: *mut u8,
                cache_size: usize,
                chunk_offset: usize,
                numeric_ID: u64,
                local_startnonce: u64,
                local_nonces: u64,
                poc_version: u8,
            );

            pub fn init_noncegen_avx2() -> ();
            pub fn noncegen_avx2(
                cache: *mut u8,
                cache_size: usize,
                chunk_offset: usize,
                numeric_ID: u64,
                local_startnonce: u64,
                local_nonces: u64,
                poc_version: u8,
            );

            pub fn init_noncegen_avx512f() -> ();
            pub fn noncegen_avx512(
                cache: *mut u8,
                cache_size: usize,
                chunk_offset: usize,
                numeric_ID: u64,
                local_startnonce: u64,
                local_nonces: u64,
                poc_version: u8,
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
    let supported_extension: &SimdExtension = &simd::SUPPORTED_SIMD_EXTENSION;
    unsafe {
        match supported_extension {
            simd::SimdExtension::AVX512f => {
                #[cfg(feature = "simd")] find_best_deadline_avx512f(scoops, nonce_count, gensig, best_deadline, best_offset);
            },
            simd::SimdExtension::AVX2 => {
                #[cfg(feature = "simd")] find_best_deadline_avx2(scoops, nonce_count, gensig, best_deadline, best_offset);
            },
            simd::SimdExtension::AVX => {
                #[cfg(feature = "simd")] find_best_deadline_avx(scoops, nonce_count, gensig, best_deadline, best_offset);
            },
            simd::SimdExtension::SSE2 => {
                #[cfg(feature = "simd")] find_best_deadline_sse2(scoops, nonce_count, gensig, best_deadline, best_offset);
            },
            simd::SimdExtension::NEON => {
                #[cfg(feature = "neon")] find_best_deadline_neon(scoops, nonce_count, gensig, best_deadline, best_offset);
            },
            simd::SimdExtension::NONE => find_best_deadline_sph(scoops, nonce_count, gensig, best_deadline, best_offset),
        }
    }
}

#[no_mangle]
pub extern fn shabal_init() {
    static INITIALIZE: Once = Once::new();
    INITIALIZE.call_once(|| {
        let supported_extension: &SimdExtension = &simd::SUPPORTED_SIMD_EXTENSION;
        unsafe {
            match supported_extension {
                simd::SimdExtension::AVX512f => {
                    #[cfg(feature = "simd")] {
                        init_shabal_avx512f();
                        init_noncegen_avx512f();
                    }
                },
                simd::SimdExtension::AVX2 => {
                    #[cfg(feature = "simd")] {
                        init_shabal_avx2();
                        init_noncegen_avx2();
                    }
                },
                simd::SimdExtension::AVX => {
                    #[cfg(feature = "simd")] {
                        init_shabal_avx();
                        init_noncegen_avx();
                    }
                },
                simd::SimdExtension::SSE2 => {
                    #[cfg(feature = "simd")] {
                        init_shabal_sse2();
                        init_noncegen_sse2();
                    }
                },
                simd::SimdExtension::NEON => {
                    #[cfg(feature = "neon")] {
                        init_shabal_neon();
                    }
                },
                _ => {}
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

/// Create a new Shabal256 instance TODO the rust one is really slow :(
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
    let supported_extension: &SimdExtension = &simd::SUPPORTED_SIMD_EXTENSION;
    unsafe {
        match supported_extension {
            simd::SimdExtension::AVX512f => {
                #[cfg(feature = "simd")]
                    noncegen_avx512(plot_buffer, plot_size / NONCE_SIZE, 0, account_id, start_nonce, nonce_count, poc_version);
            },
            simd::SimdExtension::AVX2 => {
                #[cfg(feature = "simd")]
                    noncegen_avx512(plot_buffer, plot_size / NONCE_SIZE, 0, account_id, start_nonce, nonce_count, poc_version);
            },
            simd::SimdExtension::AVX => {
                #[cfg(feature = "simd")]
                    noncegen_avx512(plot_buffer, plot_size / NONCE_SIZE, 0, account_id, start_nonce, nonce_count, poc_version);
            },
            simd::SimdExtension::SSE2 => {
                #[cfg(feature = "simd")]
                    noncegen_avx512(plot_buffer, plot_size / NONCE_SIZE, 0, account_id, start_nonce, nonce_count, poc_version);
            },
            _ => {
                let plot_buffer_borrowed = slice::from_raw_parts_mut(plot_buffer, plot_size);
                pocc::plot::noncegen_rust(plot_buffer_borrowed, 0, account_id, start_nonce, nonce_count, poc_version, );
            }
        }
    }
}

/// Creates a single PoC Nonce.
///
/// `plot_buffer` must be correct size - no size checks are performed.
#[no_mangle]
pub extern fn create_plot(
    account_id: u64,
    nonce: u64,
    poc_version: u8,
    plot_buffer: *mut u8,
) {
    unsafe {
        pocc::plot::noncegen_single_rust(
            slice::from_raw_parts_mut(plot_buffer, NONCE_SIZE as usize),
            account_id,
            nonce,
            poc_version,
        );
    }
}

/// Creates a single PoC Scoop.
///
/// `plot_buffer` must be correct size - no size checks are performed.
#[no_mangle]
pub extern fn create_scoop(
    account_id: u64,
    nonce: u64,
    scoop: u32,
    poc_version: u8,
    scoop_buffer: *mut u8,
) {
    let mut buffer = [0u8; NONCE_SIZE];
    pocc::plot::noncegen_single_rust(
        &mut buffer,
        account_id,
        nonce,
        poc_version,
    );
    let scoop_buffer_borrowed = unsafe { slice::from_raw_parts_mut(scoop_buffer, SCOOP_SIZE) };
    let offset = scoop as usize * SCOOP_SIZE;
    scoop_buffer_borrowed.clone_from_slice(&buffer[offset..offset + SCOOP_SIZE]);
}
