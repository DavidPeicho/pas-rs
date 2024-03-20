use std::{any::TypeId, marker::PhantomData};

/// Slice error
///
/// An error is raised during when creating a slice via [`Slice::new`], or [`SliceMut::new`].
#[derive(Copy, Clone, PartialEq)]
pub enum SliceError {
    /// Provided offset is out of bounds regarding the slice size, e.g.,
    ///
    /// ```rust
    /// let data: Vec<u32> = [];
    /// // Panics, since the slice doesn't have a size of at least 16 bytes.
    /// let slice: Slice<u16> = Slice::new(&data, 16, 1);
    /// ```
    OffsetOutOfBounds {
        size: usize,
        offset: usize,
    },
    /// Sliced attribute byte size is bigger than the stride, e.g.,
    ///
    /// ```rust
    /// let data: Vec<u16> = [0, 1, 2];
    /// // Panics, since the slice have a stride of 1 * std::mem::size_of::<u16>(),
    /// // but the requested attribute has size std::mem::size_of::<u32>().
    /// let slice: Slice<u32> = Slice::new(&data, 16, 1);
    /// ```
    AttributeLargerThanStride {
        type_id: TypeId,
        attr: usize,
        stride: usize,
    },
    AlignmentFault {
        type_id: TypeId,
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
                type_id,
                attr,
                stride,
            } => {
                write!(
                    f,
                    "Attribute '{:?}' with size {} bytes, larger than stride with size {}",
                    type_id, attr, stride
                )
            }
            Self::AlignmentFault { type_id, offset } => write!(
                f,
                "Attribute '{:?}' isn't aligned to the byte offset {}",
                type_id, offset
            ),
        }
    }
}

/// Slice base implementation.
///
/// Do not use this type directly, instead:
/// - Use the `slice_attr`, `slice_attr_mut`, `slice`, or `slice_mut` macros
/// - Use the [`Slice`]/[`SliceMut`] types directly
#[derive(Clone, Copy)]
pub struct SliceBase<Attr: Sized + 'static> {
    pub(crate) data: *const u8,
    end: *const u8,
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
                type_id: std::any::TypeId::of::<Attr>(),
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
                type_id: std::any::TypeId::of::<Attr>(),
                offset,
            })
        } else {
            Ok(Self {
                data: ptr,
                end: ptr_range.end,
                stride,
                _phantom: PhantomData,
            })
        }
    }

    pub(crate) fn start(&self) -> *const u8 {
        self.data
    }

    pub(crate) fn end(&self) -> *const u8 {
        let count = self.len();
        if count > 0 {
            self.get_ptr(count - 1).unwrap()
        } else {
            self.data
        }
    }

    pub fn get(&self, index: usize) -> Option<&Attr> {
        self.get_ptr(index)
            .map(|ptr| unsafe { std::mem::transmute::<_, &Attr>(ptr) })
    }

    /// Number of elements in the slice
    pub fn len(&self) -> usize {
        (self.end as usize)
            .checked_sub(self.data as usize)
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
            Some(unsafe { self.data.add(start) })
        } else {
            None
        }
    }

    /// Slice strides, in **bytes**
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
                // `end` is inclusive to avoid adding some chance of making
                // an invalid read, causing undefined behavior.
                if self.start > self.end {
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
