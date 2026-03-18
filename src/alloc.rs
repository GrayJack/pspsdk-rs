use core::{
    alloc::{AllocError, Allocator, GlobalAlloc},
    ptr::{self, NonNull},
};

use crate::sys::{
    mem::{
        sceKernelAllocPartitionMemory, sceKernelFreePartitionMemory, sceKernelGetBlockHeadAddr,
        MemoryBlockId, MemoryBlockKind, MemoryPartitionId,
    },
    SceUid,
};

const DEFAULT_PARTITION_ID: MemoryPartitionId = if cfg!(feature = "kernel") {
    MemoryPartitionId::MainKernel
} else {
    MemoryPartitionId::MainUser
};

/// An general allocator for the PSP OS.
pub struct SystemAlloc;

/// An allocator to a specific PSP RAM partition
pub struct PartitionAlloc {
    partition: MemoryPartitionId,
}

impl PartitionAlloc {
    pub const fn new(partition: MemoryPartitionId) -> Self {
        Self { partition }
    }
}


unsafe impl GlobalAlloc for SystemAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let size = layout.size() + size_of::<SceUid>() + layout.align();

        let res = unsafe {
            sceKernelAllocPartitionMemory(
                DEFAULT_PARTITION_ID,
                c"SystemAlloc".as_ptr().cast(),
                MemoryBlockKind::Low,
                size,
                0,
            )
        };

        match res.into_result() {
            Ok(id) => {
                let mut ptr: *mut u8 = sceKernelGetBlockHeadAddr(id).cast();
                unsafe {
                    *ptr.cast() = id;

                    ptr = ptr.wrapping_add(size_of::<SceUid>());

                    // We must add at least one, to store this value.
                    let align_padding = 1 + ptr.wrapping_add(1).align_offset(layout.align());
                    *ptr.wrapping_add(align_padding - 1) = align_padding as u8;
                    ptr.wrapping_add(align_padding)
                }
            },
            Err(_) => ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        if ptr.is_null() {
            return;
        }
        unsafe {
            let align_padding = *ptr.wrapping_sub(1);

            let id = *ptr.wrapping_sub(align_padding as usize).cast::<MemoryBlockId>().offset(-1);

            let _res = sceKernelFreePartitionMemory(id);
        }
    }
}

unsafe impl Allocator for SystemAlloc {
    fn allocate(
        &self, layout: core::alloc::Layout,
    ) -> Result<ptr::NonNull<[u8]>, core::alloc::AllocError> {
        match layout.size() {
            0 => Ok(NonNull::slice_from_raw_parts(layout.dangling_ptr(), 0)),
            // SAFETY: `layout` is non-zero in size,
            size => {
                let size = size + size_of::<SceUid>() + layout.align();

                let res = unsafe {
                    sceKernelAllocPartitionMemory(
                        DEFAULT_PARTITION_ID,
                        c"SystemAlloc".as_ptr().cast(),
                        MemoryBlockKind::Low,
                        size,
                        0,
                    )
                };

                match res.into_result() {
                    Ok(id) => {
                        let mut ptr: *mut u8 = sceKernelGetBlockHeadAddr(id).cast();
                        unsafe {
                            *ptr.cast() = id;

                            ptr = ptr.wrapping_add(size_of::<SceUid>());

                            // We must add at least one, to store this value.
                            let align_padding =
                                1 + ptr.wrapping_add(1).align_offset(layout.align());
                            *ptr.wrapping_add(align_padding - 1) = align_padding as u8;

                            let ptr = NonNull::new_unchecked(ptr.wrapping_add(align_padding));

                            Ok(NonNull::slice_from_raw_parts(ptr, size))
                        }
                    },
                    Err(_) => Err(AllocError),
                }
            },
        }
    }

    unsafe fn deallocate(&self, ptr: ptr::NonNull<u8>, layout: core::alloc::Layout) {
        if layout.size() != 0 {
            unsafe {
                let align_padding = *ptr.sub(1).as_ptr();

                let id =
                    *ptr.sub(align_padding as usize).cast::<MemoryBlockId>().offset(-1).as_ptr();

                let _res = sceKernelFreePartitionMemory(id);
            }
        }
    }
}

unsafe impl GlobalAlloc for PartitionAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let size = layout.size() + size_of::<SceUid>() + layout.align();

        let res = unsafe {
            sceKernelAllocPartitionMemory(
                self.partition,
                c"SystemAlloc".as_ptr().cast(),
                MemoryBlockKind::Low,
                size,
                0,
            )
        };

        match res.into_result() {
            Ok(id) => {
                let mut ptr: *mut u8 = sceKernelGetBlockHeadAddr(id).cast();
                unsafe {
                    *ptr.cast() = id;

                    ptr = ptr.wrapping_add(size_of::<SceUid>());

                    // We must add at least one, to store this value.
                    let align_padding = 1 + ptr.wrapping_add(1).align_offset(layout.align());
                    *ptr.wrapping_add(align_padding - 1) = align_padding as u8;
                    ptr.wrapping_add(align_padding)
                }
            },
            Err(_) => ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        if ptr.is_null() {
            return;
        }
        unsafe {
            let align_padding = *ptr.wrapping_sub(1);

            let id = *ptr.wrapping_sub(align_padding as usize).cast::<MemoryBlockId>().offset(-1);

            let _res = sceKernelFreePartitionMemory(id);
        }
    }
}

unsafe impl Allocator for PartitionAlloc {
    fn allocate(
        &self, layout: core::alloc::Layout,
    ) -> Result<ptr::NonNull<[u8]>, core::alloc::AllocError> {
        match layout.size() {
            0 => Ok(NonNull::slice_from_raw_parts(layout.dangling_ptr(), 0)),
            // SAFETY: `layout` is non-zero in size,
            size => {
                let size = size + size_of::<SceUid>() + layout.align();

                let res = unsafe {
                    sceKernelAllocPartitionMemory(
                        self.partition,
                        c"SystemAlloc".as_ptr().cast(),
                        MemoryBlockKind::Low,
                        size,
                        0,
                    )
                };

                match res.into_result() {
                    Ok(id) => {
                        let mut ptr: *mut u8 = sceKernelGetBlockHeadAddr(id).cast();
                        unsafe {
                            *ptr.cast() = id;

                            ptr = ptr.wrapping_add(size_of::<SceUid>());

                            // We must add at least one, to store this value.
                            let align_padding =
                                1 + ptr.wrapping_add(1).align_offset(layout.align());
                            *ptr.wrapping_add(align_padding - 1) = align_padding as u8;

                            let ptr = NonNull::new_unchecked(ptr.wrapping_add(align_padding));

                            Ok(NonNull::slice_from_raw_parts(ptr, size))
                        }
                    },
                    Err(_) => Err(AllocError),
                }
            },
        }
    }

    unsafe fn deallocate(&self, ptr: ptr::NonNull<u8>, layout: core::alloc::Layout) {
        if layout.size() != 0 {
            unsafe {
                let align_padding = *ptr.sub(1).as_ptr();

                let id =
                    *ptr.sub(align_padding as usize).cast::<MemoryBlockId>().offset(-1).as_ptr();

                let _res = sceKernelFreePartitionMemory(id);
            }
        }
    }
}
