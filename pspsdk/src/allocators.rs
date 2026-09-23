//! Allocators for the PSP system.
use core::{
    alloc::{AllocError, Allocator, GlobalAlloc, Layout},
    ffi::CStr,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

use alloc::alloc::GlobalAllocator;

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

const DEFAULT_PARTITION_ID: MemoryPartitionId =
    cfg_select! {
        all(prx, feature = "kernel") => MemoryPartitionId::MainKernel,
        _ => MemoryPartitionId::MainUser,
    };

const DEFAULT_VPL_SIZE: usize = cfg_select! {
    // 2 KB
    all(prx, feature = "kernel") => 2048,
    // 16 KB
    all(prx, not(feature = "kernel")) => 16 * 1024,
    // 64 KB
    _ => 64 * 1024,
};

/// An general allocator for the PSP OS.
#[derive(Clone)]
pub struct SystemAlloc;

unsafe impl core::alloc::AllocatorClone for SystemAlloc {}
unsafe impl core::alloc::StaticAllocator for SystemAlloc {}
unsafe impl GlobalAllocator for SystemAlloc {}

/// An allocator to a specific PSP RAM partition
#[derive(Clone)]
pub struct PartitionAlloc {
    partition: MemoryPartitionId,
}

unsafe impl core::alloc::AllocatorClone for PartitionAlloc {}
unsafe impl core::alloc::StaticAllocator for PartitionAlloc {}
unsafe impl GlobalAllocator for PartitionAlloc {}

impl PartitionAlloc {
    /// Creates a new `PartitionAlloc` that will allocate in the given PSP `partition`.
    ///
    /// The allocation policy is to allocate from the lowest available address.
    pub const fn new(partition: MemoryPartitionId) -> Self {
        Self { partition }
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

                match res.ok() {
                    Some(id) => {
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
                    None => Err(AllocError),
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

                match res.ok() {
                    Some(id) => {
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
                    None => Err(AllocError),
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
#[derive(Clone)]
pub struct VariablePoolAlloc {
    id: VplId,
}

unsafe impl core::alloc::AllocatorClone for VariablePoolAlloc {}
unsafe impl core::alloc::StaticAllocator for VariablePoolAlloc {}
unsafe impl GlobalAllocator for VariablePoolAlloc {}

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

/// The default memory allocator provided by the operating system.
///
/// This is based on `malloc` on Unix platforms and `HeapAlloc` on Windows,
/// plus related functions. However, it is not valid to mix use of the backing
/// system allocator with `System`, as this implementation may include extra
/// work, such as to serve alignment requests greater than the alignment
/// provided directly by the backing system allocator.
///
/// This type implements the [`GlobalAlloc`] trait. Currently the default
/// global allocator is unspecified. Libraries, however, like `cdylib`s and
/// `staticlib`s are guaranteed to use the [`System`] by default and as such
/// work as if they had this definition:
///
/// ```rust
/// use pspsdk::alloc::System;
///
/// #[global_allocator]
/// static A: System = System;
///
/// fn main() {
///     let a = Box::new(4); // Allocates from the system allocator.
///     println!("{a}");
/// }
/// ```
///
/// You can also define your own wrapper around `System` if you'd like, such as
/// keeping track of the number of all bytes allocated:
///
/// ```rust
/// use core::{
///     alloc::{GlobalAlloc, Layout},
///     sync::atomic::{AtomicUsize, Ordering::Relaxed},
/// };
///
/// use pspsdk::alloc::System;
///
/// struct Counter;
///
/// static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
///
/// unsafe impl GlobalAlloc for Counter {
///     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
///         let ret = unsafe { System.alloc(layout) };
///         if !ret.is_null() {
///             ALLOCATED.fetch_add(layout.size(), Relaxed);
///         }
///         ret
///     }
///
///     unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
///         unsafe {
///             System.dealloc(ptr, layout);
///         }
///         ALLOCATED.fetch_sub(layout.size(), Relaxed);
///     }
/// }
///
/// #[global_allocator]
/// static A: Counter = Counter;
///
/// fn main() {
///     println!("allocated bytes before main: {}", ALLOCATED.load(Relaxed));
/// }
/// ```
///
/// It can also be used directly to allocate memory independently of whatever
/// global allocator has been selected for a Rust program. For example if a Rust
/// program opts in to using jemalloc as the global allocator, `System` will
/// still allocate memory using `malloc` and `HeapAlloc`.
#[derive(Copy, Debug, Clone, Default)]
// #[derive_const(Clone, Default)]
pub struct System;

unsafe impl core::alloc::AllocatorClone for System {}
unsafe impl core::alloc::StaticAllocator for System {}

impl System {
    #[inline]
    fn alloc_impl(&self, layout: Layout, zeroed: bool) -> Result<NonNull<[u8]>, AllocError> {
        match layout.size() {
            0 => Ok(layout.dangling_ptr().cast_slice(0)),
            // SAFETY: `layout` is non-zero in size,
            size => unsafe {
                let raw_ptr = if zeroed {
                    SystemAlloc.alloc_zeroed(layout)
                } else {
                    SystemAlloc.alloc(layout)
                };
                let ptr = NonNull::new(raw_ptr).ok_or(AllocError)?;
                Ok(ptr.cast_slice(size))
            },
        }
    }

    // SAFETY: Same as `Allocator::grow`
    #[inline]
    unsafe fn grow_impl(
        &self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout, zeroed: bool,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        match old_layout.size() {
            0 => self.alloc_impl(new_layout, zeroed),

            // SAFETY: `new_size` is non-zero as `new_size` is greater than or equal to `old_size`
            // as required by safety conditions and the `old_size == 0` case was handled in the
            // previous match arm. Other conditions must be upheld by the caller
            old_size if old_layout.align() == new_layout.align() => unsafe {
                let new_size = new_layout.size();

                // `realloc` probably checks for `new_size >= old_layout.size()` or something
                // similar.
                core::hint::assert_unchecked(new_size >= old_layout.size());

                let raw_ptr = SystemAlloc.realloc(ptr.as_ptr(), old_layout, new_size);
                let ptr = NonNull::new(raw_ptr).ok_or(AllocError)?;
                if zeroed {
                    raw_ptr.add(old_size).write_bytes(0, new_size - old_size);
                }
                Ok(ptr.cast_slice(new_size))
            },

            // SAFETY: because `new_layout.size()` must be greater than or equal to `old_size`,
            // both the old and new memory allocation are valid for reads and writes for `old_size`
            // bytes. Also, because the old allocation wasn't yet deallocated, it cannot overlap
            // `new_ptr`. Thus, the call to `copy_nonoverlapping` is safe. The safety contract
            // for `dealloc` must be upheld by the caller.
            old_size => unsafe {
                let new_ptr = self.alloc_impl(new_layout, zeroed)?;
                ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), old_size);
                Allocator::deallocate(self, ptr, old_layout);
                Ok(new_ptr)
            },
        }
    }
}

// The Allocator impl checks the layout size to be non-zero and forwards to the
// platform functions in `std::sys::*::alloc`.
unsafe impl Allocator for System {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.alloc_impl(layout, false)
    }

    #[inline]
    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.alloc_impl(layout, true)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() != 0 {
            // SAFETY: `layout` is non-zero in size,
            // other conditions must be upheld by the caller
            unsafe { SystemAlloc.dealloc(ptr.as_ptr(), layout) }
        }
    }

    #[inline]
    unsafe fn grow(
        &self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        // SAFETY: all conditions must be upheld by the caller
        unsafe { self.grow_impl(ptr, old_layout, new_layout, false) }
    }

    #[inline]
    unsafe fn grow_zeroed(
        &self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        // SAFETY: all conditions must be upheld by the caller
        unsafe { self.grow_impl(ptr, old_layout, new_layout, true) }
    }

    #[inline]
    unsafe fn shrink(
        &self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() <= old_layout.size(),
            "`new_layout.size()` must be smaller than or equal to `old_layout.size()`"
        );

        match new_layout.size() {
            // SAFETY: conditions must be upheld by the caller
            0 => unsafe {
                Allocator::deallocate(self, ptr, old_layout);
                Ok(new_layout.dangling_ptr().cast_slice(0))
            },

            // SAFETY: `new_size` is non-zero. Other conditions must be upheld by the caller
            new_size if old_layout.align() == new_layout.align() => unsafe {
                // `realloc` probably checks for `new_size <= old_layout.size()` or something
                // similar.
                core::hint::assert_unchecked(new_size <= old_layout.size());

                let raw_ptr = SystemAlloc.realloc(ptr.as_ptr(), old_layout, new_size);
                let ptr = NonNull::new(raw_ptr).ok_or(AllocError)?;
                Ok(ptr.cast_slice(new_size))
            },

            // SAFETY: because `new_size` must be smaller than or equal to `old_layout.size()`,
            // both the old and new memory allocation are valid for reads and writes for `new_size`
            // bytes. Also, because the old allocation wasn't yet deallocated, it cannot overlap
            // `new_ptr`. Thus, the call to `copy_nonoverlapping` is safe. The safety contract
            // for `dealloc` must be upheld by the caller.
            new_size => unsafe {
                let new_ptr = Allocator::allocate(self, new_layout)?;
                ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), new_size);
                Allocator::deallocate(self, ptr, old_layout);
                Ok(new_ptr)
            },
        }
    }
}

unsafe impl GlobalAllocator for System {}

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
