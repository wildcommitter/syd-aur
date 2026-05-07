//
// hardened-malloc: Global allocator using GrapheneOS allocator
// src/lib.rs: Global allocator definition
//
// Copyright (c) 2025, 2026 Ali Polatel <alip@chesswob.org>
// Based in part upon hardened_malloc-rs/src/lib.rs which is
//   Copyright (c) strawberry <strawberry@puppygock.gay>
//   SPDX-License-Identifier: Apache-2.0 OR MIT
//
// SPDX-License-Identifier: MIT

#![no_std]

use core::{
    alloc::{GlobalAlloc, Layout},
    ffi::c_void,
};

// POSIX
pub use hardened_malloc_sys::posix_memalign;
// C standard
pub use hardened_malloc_sys::{aligned_alloc, calloc, free, malloc, realloc};
// hardened_malloc extensions
pub use hardened_malloc_sys::{free_sized, malloc_object_size, malloc_object_size_fast};

pub struct HardenedMalloc;

// hardened_malloc's malloc()/calloc()/realloc() guarantee 16-byte alignment.
// Honor over-aligned layouts via posix_memalign and matching plain free.
//
// All trait methods are `#[inline(never)]`: under `#[inline]`, LLVM produced
// incorrect code at over-aligned call sites (occasionally returning malloc-
// aligned, 16-byte, pointers for 32-byte-aligned types like AVX2 SIMD blocks).
const MIN_MALLOC_ALIGN: usize = 16;

unsafe impl GlobalAlloc for HardenedMalloc {
    #[inline(never)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= MIN_MALLOC_ALIGN {
            malloc(layout.size()) as *mut u8
        } else {
            let mut p: *mut c_void = core::ptr::null_mut();
            if posix_memalign(&mut p, layout.align(), layout.size()) != 0 {
                core::ptr::null_mut()
            } else {
                p as *mut u8
            }
        }
    }

    #[inline(never)]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= MIN_MALLOC_ALIGN {
            calloc(layout.size(), 1) as *mut u8
        } else {
            let p = self.alloc(layout);
            if !p.is_null() {
                core::ptr::write_bytes(p, 0, layout.size());
            }
            p
        }
    }

    #[inline(never)]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // For over-aligned allocations, posix_memalign may have placed us
        // in a larger size class (the slab system rounds up for alignment).
        // free_sized would detect the size mismatch and abort; use plain
        // free, which discovers the slot's true size from metadata.
        if layout.align() <= MIN_MALLOC_ALIGN {
            free_sized(ptr as *mut c_void, layout.size());
        } else {
            free(ptr as *mut c_void);
        }
    }

    #[inline(never)]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if layout.align() <= MIN_MALLOC_ALIGN {
            realloc(ptr as *mut c_void, size) as *mut u8
        } else {
            let new_layout = match Layout::from_size_align(size, layout.align()) {
                Ok(l) => l,
                Err(_) => return core::ptr::null_mut(),
            };
            let new_ptr = self.alloc(new_layout);
            if !new_ptr.is_null() {
                let copy = core::cmp::min(layout.size(), size);
                core::ptr::copy_nonoverlapping(ptr, new_ptr, copy);
                self.dealloc(ptr, layout);
            }
            new_ptr
        }
    }
}
