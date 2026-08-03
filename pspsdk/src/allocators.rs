//! Allocators for the PSP system.
use core::{
    alloc::{AllocError, Allocator, GlobalAlloc},
    ffi::CStr,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

use crate::{
    io,
    sync::OnceLock,
    sys::{
        mem::{
            sceKernelAllocPartitionMemory, sceKernelFreePartitionMemory, sceKernelGetBlockHeadAddr,
            MemoryBlockId,
        },
        thread::{
            sceKernelCreateVpl, sceKernelDeleteVpl, sceKernelFreeVpl, sceKernelTryAllocateVpl,
            VplAttributes, VplId,
        },
        SceUid,
    },
};

pub use crate::sys::mem::{MemoryBlockKind, MemoryPartitionId};

#[global_allocator]
static GLOBAL_ALLOC: SystemAlloc = SystemAlloc;

const DEFAULT_PARTITION_ID: MemoryPartitionId = cfg_select! {
    all(prx, feature = "kernel") => MemoryPartitionId::MainKernel,
    _ => MemoryPartitionId::MainUser
};

const DEFAULT_VPL_SIZE: usize = cfg_select! {
    // 2 KB
    all(prx, feature = "kernel") => 2048,
    // 16 KB
    all(prx, not(feature = "kernel")) => 16*1024,
    // 64 KB
    _ => 64*1024,
};

/// An general allocator for the PSP OS.
pub struct SystemAlloc;

/// An allocator to a specific PSP RAM partition
pub struct PartitionAlloc {
    partition: MemoryPartitionId,
}

impl PartitionAlloc {
    /// Creates a new `PartitionAlloc` that will allocate in the given PSP `partition`.
    ///
    /// The allocation policy is to allocate from the lowest available address.
    pub const fn new(partition: MemoryPartitionId) -> Self {
        Self { partition }
    }
}


unsafe impl GlobalAlloc for SystemAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let size = layout.size() + size_of::<SceUid>();

        let res = unsafe {
            sceKernelAllocPartitionMemory(
                DEFAULT_PARTITION_ID,
                c"SystemAlloc".as_ptr().cast(),
                MemoryBlockKind::LowAligned,
                size,
                layout.align(),
            )
        };

        match res.into_result() {
            Ok(id) => {
                let mut ptr: *mut u8 = sceKernelGetBlockHeadAddr(id).cast();

                if ptr.is_null() {
                    return ptr;
                }

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
                let size = size + size_of::<SceUid>();

                let res = unsafe {
                    sceKernelAllocPartitionMemory(
                        DEFAULT_PARTITION_ID,
                        c"SystemAlloc".as_ptr().cast(),
                        MemoryBlockKind::LowAligned,
                        size,
                        layout.align(),
                    )
                };

                match res.into_result() {
                    Ok(id) => {
                        let mut ptr: *mut u8 = sceKernelGetBlockHeadAddr(id).cast();

                        if ptr.is_null() {
                            return Err(AllocError);
                        }

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
        let size = layout.size() + size_of::<SceUid>();

        let res = unsafe {
            sceKernelAllocPartitionMemory(
                self.partition,
                c"PartitionAlloc".as_ptr().cast(),
                MemoryBlockKind::LowAligned,
                size,
                layout.align(),
            )
        };

        match res.into_result() {
            Ok(id) => {
                let mut ptr: *mut u8 = sceKernelGetBlockHeadAddr(id).cast();

                if ptr.is_null() {
                    return ptr;
                }

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
                let size = size + size_of::<SceUid>();

                let res = unsafe {
                    sceKernelAllocPartitionMemory(
                        self.partition,
                        c"PartitionAlloc".as_ptr().cast(),
                        MemoryBlockKind::LowAligned,
                        size,
                        layout.align(),
                    )
                };

                match res.into_result() {
                    Ok(id) => {
                        let mut ptr: *mut u8 = sceKernelGetBlockHeadAddr(id).cast();

                        if ptr.is_null() {
                            return Err(AllocError);
                        }

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

/// A builder for the [`VariablePoolAlloc`].
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct VariablePoolAllocBuilder {
    name: &'static CStr,
    partition: MemoryPartitionId,
    pool_size: usize,
    attr: VplAttributes,
}

/// A memory allocator using the PSP Variable-sized Memory Pool.
pub struct VariablePoolAlloc {
    id: VplId,
}

impl VariablePoolAlloc {
    /// Creates a variable memory pool allocator with a given size.
    ///
    /// The RAM partition used is the default partition on the compilation context. For user
    /// application is [`MemoryPartitionId::MainUser`] while for kernel it is
    /// [`MemoryPartitionId::MainKernel`]. To specify or configure the `VariablePoolAlloc`
    /// further, you can use [`VariablePoolAlloc::builder`] and build to your liking.
    pub fn new(pool_size: usize) -> io::Result<Self> {
        Self::builder().name(c"SDK_VPL").size(pool_size).create()
    }

    /// Creates a builder for the `VariablePoolAlloc` that can be used to configure it's
    /// behavior before creating the allocator.
    #[must_use]
    pub const fn builder() -> VariablePoolAllocBuilder {
        VariablePoolAllocBuilder::new()
    }
}

unsafe impl Allocator for VariablePoolAlloc {
    fn allocate(&self, layout: core::alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        match layout.size() {
            0 => Ok(NonNull::slice_from_raw_parts(layout.dangling_ptr(), 0)),
            // SAFETY: `layout` is non-zero in size,
            size => unsafe {
                let size = size + size_of::<SceUid>() + layout.align();

                let mut mem_block = MaybeUninit::uninit();
                sceKernelTryAllocateVpl(self.id, size, mem_block.as_mut_ptr())
                    .into_result()
                    .map_err(|_| AllocError)?;

                let ptr: *mut u8 = mem_block.assume_init().cast();
                let align_padding = ptr.wrapping_add(1).align_offset(layout.align());

                let ptr = NonNull::new_unchecked(ptr.wrapping_add(align_padding));

                Ok(NonNull::slice_from_raw_parts(ptr, size))
            },
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: core::alloc::Layout) {
        if layout.size() != 0 {
            unsafe {
                let _res = sceKernelFreeVpl(self.id, ptr.as_ptr().cast());
            }
        }
    }
}

impl Drop for VariablePoolAlloc {
    fn drop(&mut self) {
        let _ = sceKernelDeleteVpl(self.id);
    }
}

impl VariablePoolAllocBuilder {
    #[inline]
    #[must_use]
    const fn new() -> Self {
        Self {
            name: c"",
            partition: DEFAULT_PARTITION_ID,
            pool_size: 0,
            attr: VplAttributes::WaitByFIFO,
        }
    }

    /// Sets the name for the [`VariablePoolAlloc`] when created.
    ///
    /// This name is only used for debug purposes.
    pub const fn name(&mut self, name: &'static CStr) -> &mut Self {
        self.name = name;
        self
    }

    /// Sets the PSP RAM partition the [`VariablePoolAlloc`] will use.
    ///
    /// If not set, it will default to [`MemoryPartitionId::MainUser`] on user-level software, or
    /// [`MemoryPartitionId::MainKernel`] on kernel-level software.
    pub const fn partition(&mut self, partition: MemoryPartitionId) -> &mut Self {
        self.partition = partition;
        self
    }

    /// Sets the size of the [`VariablePoolAlloc`].
    pub const fn size(&mut self, size: usize) -> &mut Self {
        self.pool_size = size;
        self
    }

    /// Sets the attributes passed to the [`VariablePoolAlloc`] on creation.
    ///
    /// If not set, it default to [`VplAttributes::WaitByFIFO`] (the type defaults).
    pub const fn attributes(&mut self, attr: VplAttributes) -> &mut Self {
        self.attr = attr;
        self
    }

    /// Creates the [`VariablePoolAlloc`] from the set configuration.
    pub fn create(&self) -> io::Result<VariablePoolAlloc> {
        let pool_size = if self.pool_size == 0 {
            DEFAULT_VPL_SIZE
        } else {
            self.pool_size
        };

        let id = unsafe {
            sceKernelCreateVpl(
                self.name.as_ptr().cast(),
                self.partition,
                self.attr,
                pool_size,
                None,
            )
            .into_result()?
        };

        Ok(VariablePoolAlloc { id })
    }
}

impl Default for VariablePoolAllocBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GlobalVariablePoolAlloc {
    inner: OnceLock<VariablePoolAlloc>,
}

impl GlobalVariablePoolAlloc {
    pub const fn new() -> Self {
        Self {
            inner: OnceLock::new(),
        }
    }
}

impl Default for GlobalVariablePoolAlloc {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAlloc for GlobalVariablePoolAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let allocator = self.inner.get_or_try_init(|| {
            VariablePoolAlloc::builder()
                .name(c"SDK_GLOBAL_VPL")
                .size(DEFAULT_VPL_SIZE)
                .create()
        });

        match allocator {
            Ok(allocator) => {
                let res = allocator.allocate(layout);
                res.map(|r| r.as_mut_ptr()).unwrap_or_else(|_| ptr::null_mut())
            },
            Err(_) => ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let allocator = self.inner.get_or_try_init(|| {
            VariablePoolAlloc::builder()
                .name(c"SDK_GLOBAL_VPL")
                .size(DEFAULT_VPL_SIZE)
                .create()
        });

        if let Ok(allocator) = allocator {
            unsafe {
                if !ptr.is_null() {
                    // Safety: we checked that is not null
                    let ptr = NonNull::new_unchecked(ptr);
                    allocator.deallocate(ptr, layout);
                }
            }
        }
    }
}

#[alloc_error_handler]
#[cfg(not(feature = "std"))]
fn aeh(layout: core::alloc::Layout) -> ! {
    use crate::{io::Write, sys::SceError};

    if cfg!(pbp) {
        crate::dprintln!(
            "Failed to allocate {} bytes with {} alignment",
            layout.size(),
            layout.align()
        )
    }

    if let Some(mut out) = crate::os::stdio::panic_output() {
        let _ = out.write_fmt(format_args!(
            "Failed to allocate {} bytes with {} alignment\n",
            layout.size(),
            layout.align()
        ));
    }

    for _ in 0..10 {
        if crate::sys::is_interrupt_enabled() {
            crate::process::exit(SceError::NO_MEMORY.to_inner().cast_signed());
        }
        crate::sys::spin_loop();
    }
    crate::process::abort()
}
