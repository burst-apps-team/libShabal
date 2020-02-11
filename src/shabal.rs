use shabal::{Shabal256, Digest};
use std::os::raw::c_void;
use std::slice;

pub fn shabal256_new() -> *mut c_void {
    return Box::into_raw(Box::new(Shabal256::new())) as *mut c_void;
}

pub fn shabal256_destroy(shabal: *mut c_void) {
    if !shabal.is_null() {
        unsafe {
            // Let it fall out of scope naturally once it is unboxed
            Box::from_raw(shabal as *mut Shabal256);
        }
    }
}

pub fn shabal256_reset(shabal: *mut c_void) {
    if !shabal.is_null() {
        unsafe {
            let shabal_borrowed = &mut *(shabal as *mut Shabal256);
            shabal_borrowed.reset();
        }
    }
}

pub fn shabal256_update(shabal: *mut c_void, data: *const u8, offset: usize, len: usize) {
    if !shabal.is_null() {
        unsafe {
            let array = slice::from_raw_parts(data.add(offset), len as usize);
            let shabal_borrowed = &mut *(shabal as *mut Shabal256);
            shabal_borrowed.input(array);
        }
    }
}

pub fn shabal256_digest(shabal: *mut c_void, buffer: *mut u8, offset: usize) {
    if !shabal.is_null() {
        unsafe {
            let array = slice::from_raw_parts_mut(buffer.add(offset), 32);
            let shabal_borrowed = &mut *(shabal as *mut Shabal256);
            array.copy_from_slice(shabal_borrowed.result_reset().as_slice());
        }
    }
}
