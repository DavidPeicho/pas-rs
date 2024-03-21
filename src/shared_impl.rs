use std::marker::PhantomData;

/// Slice error
///
/// An error is raised during when creating a slice via [`crate::Slice::new`],
/// or [`crate::SliceMut::new`].
#[derive(Copy, Clone, PartialEq)]
pub enum SliceError {
    /// Provided offset is out of bounds regarding the slice size.
    ///
    /// ## Example
    ///
    /// ```rust,should_panic
    /// use strided_slice::{Slice};
    ///
    /// let data: Vec<u32> = Vec::new();
    /// // Panics, since the slice doesn't have a size of at least 16 bytes.
    /// let slice: Slice<u32> = Slice::new(&data, 16, 1);
    /// ```
    OffsetOutOfBounds {
        /// Slice size, in **bytes**
        size: usize,
        /// Byte offset
        offset: usize,
    },
    /// Sliced attribute byte size is bigger than the stride.
    ///
    /// ## Example
    ///
    /// ```rust,should_panic
    /// use strided_slice::{Slice};
    ///
    /// let data: Vec<u16> = vec!(0_u16, 1, 2);
    /// // Panics, since the slice have a stride of 1 * std::mem::size_of::<u16>(),
    /// // but the requested attribute has size std::mem::size_of::<u32>().
    /// let slice: Slice<u32> = Slice::new(&data, 16, 1);
    /// ```
    AttributeLargerThanStride {
        /// Type name of the attribute read by the slice
        type_name: &'static str,
        /// Attribute size, in **bytes**
        attr: usize,
        /// Slice stride, in **bytes**
        stride: usize,
    },
    /// Attribute is not aligned to the request offset in the slice.
    ///
    /// ## Example
    ///
    /// ```rust,should_panic
    /// use strided_slice::{Slice};
    ///
    /// let data: Vec<u8> = vec!(0_u8, 1, 2);
    /// // Panics, since the offset will be unaligned
    /// let slice: Slice<u32> = Slice::new(&data, 1, 1);
    /// ```
    AlignmentFault {
        /// Type name of the attribute read by the slice
        type_name: &'static str,
        /// Byte offset
        offset: usize,
    },
}

impl std::fmt::Debug for SliceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OffsetOutOfBounds { size, offset } => {
                write!(
                    f,
                    "Byte offset is {}, but slice has a size of {} bytes",
                    offset, size
                )
            }
            Self::AttributeLargerThanStride {
                type_name,
                attr,
                stride,
            } => {
                write!(
                    f,
                    "Attribute '{:?}' with size {} bytes, larger than stride with size {}",
                    type_name, attr, stride
                )
            }
            Self::AlignmentFault { type_name, offset } => write!(
                f,
                "Attribute '{:?}' isn't aligned to the byte offset {}",
                type_name, offset
            ),
        }
    }
}

#[doc(hidden)]
/// Slice base implementation.
///
/// Do not use this type directly, instead:
/// - Use the [`slice_attr`], [`slice_attr_mut`], [`slice`], or [`slice_mut`] macros
/// - Use the [`Slice`] or [`SliceMut`] types
#[derive(Clone, Copy)]
pub struct SliceBase<Attr: Sized + 'static> {
    /// Start pointer, pointing on the first byte of the slice.
    pub(crate) start: *const u8,
    /// End pointer, pointing one byte **after** the end of the slice.
    pub(crate) end: *const u8,
    /// Stride, in **bytes**
    stride: usize,
    _phantom: PhantomData<Attr>,
}

impl<Attr: Sized> SliceBase<Attr> {
    pub(crate) fn new_typed<V: Pod>(
        data: &[V],
        offset: usize,
        elt_count: usize,
    ) -> Result<Self, SliceError> {
        let stride = std::mem::size_of::<V>() * elt_count;
        let bytes = std::mem::size_of_val(data);
        let ptr = data.as_ptr_range();
        Self::new(
            ptr.start as *const u8..ptr.end as *const u8,
            offset,
            stride,
            bytes,
        )
    }

    pub(crate) fn new(
        ptr_range: std::ops::Range<*const u8>,
        offset: usize,
        stride: usize,
        bytes: usize,
    ) -> Result<Self, SliceError> {
        let ptr: *const u8 = unsafe { ptr_range.start.add(offset) };
        // Empty slice are allowed, but we need to ensure that
        // the offset and stride are valid.
        if std::mem::size_of::<Attr>() > stride {
            Err(SliceError::AttributeLargerThanStride {
                type_name: std::any::type_name::<Attr>(),
                attr: std::mem::size_of::<Attr>(),
                stride,
            })
        } else if offset > 0 && offset >= bytes {
            Err(SliceError::OffsetOutOfBounds {
                size: bytes,
                offset,
            })
        } else if ptr.align_offset(std::mem::align_of::<Attr>()) != 0 {
            Err(SliceError::AlignmentFault {
                type_name: std::any::type_name::<Attr>(),
                offset,
            })
        } else {
            Ok(Self {
                start: ptr,
                end: ptr_range.end,
                stride,
                _phantom: PhantomData,
            })
        }
    }

    /// Get the reference at index.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use strided_slice::Slice;
    ///
    /// let data = [1, 2, 3, 4];
    /// let slice: Slice<u32> = Slice::new(&data, 0, 1);
    /// println!("{}", slice[0]); // Prints `1`
    /// println!("{}", slice[3]); // Prints `3`
    /// ```
    pub fn get(&self, index: usize) -> Option<&Attr> {
        self.get_ptr(index)
            .map(|ptr| unsafe { std::mem::transmute::<_, &Attr>(ptr) })
    }

    /// Number of elements in the slice.
    pub fn len(&self) -> usize {
        (self.end as usize)
            .checked_sub(self.start as usize)
            .unwrap()
            .div_ceil(self.stride)
    }

    /// `true` if the slice has size `0`, `false` otherwise
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get a pointer to the element at index `index`
    pub(crate) fn get_ptr(&self, index: usize) -> Option<*const u8> {
        if index < self.len() {
            let start = self.stride * index;
            Some(unsafe { self.start.add(start) })
        } else {
            None
        }
    }

    /// Slice stride.
    ///
    /// <div class="warning">The stride is not in **elements count**, but in **bytes**.</div>
    pub fn stride(&self) -> usize {
        self.stride
    }
}

/// Implement [`Iterator`] and related traits for [`SliceIterator`]/[`SliceIteratorMut`].
macro_rules! impl_iterator {
    ($name: ident -> $elem: ty) => {
        impl<'a, T: Pod> Iterator for $name<'a, T> {
            type Item = $elem;

            fn next(&mut self) -> Option<$elem> {
                // `end` is exclusive and points one byte after the end of the slice.
                if self.start >= self.end {
                    return None;
                }
                unsafe {
                    let ret = Some(std::mem::transmute::<_, $elem>(self.start));
                    self.start = self.start.add(self.stride);
                    ret
                }
            }
        }

        impl<'a, T: Pod + Debug> std::fmt::Debug for $name<'a, T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_list().entries(self.into_iter()).finish()
            }
        }
    };
}

use bytemuck::Pod;
pub(super) use impl_iterator;
