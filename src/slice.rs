use bytemuck::Pod;
use std::{fmt::Debug, marker::PhantomData, ops::Deref};

use crate::shared_impl::{impl_iterator, SliceBase};

/// Immutable slice with custom stride and start byte offset.
///
/// # Example
///
/// Creating a slice with a stride equal to the element size:
///
/// ```
/// use strided_slice::Slice;
/// let array = [1.0, 2.0, 3.0];
/// let slice: Slice<f64> = Slice::new(&array, 0, 1);
/// ```
///
/// # Important Notes
///
/// - The struct transmust without checking endianness
#[derive(Clone, Copy)]
pub struct Slice<'a, T: Pod> {
    inner: SliceBase<T>,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: Pod> Slice<'a, T> {
    /// Create a strided slice starting at the byte offset `offset`.
    ///
    /// - `offset` represents the byte offset in `V` to start from and **must** be less than
    ///   the size of `V`
    /// - The size of `T` must be smaller or equal to `sizeof::<V>() - offset`
    /// - The slice stride is set to the size of `V`
    ///
    /// # Examples
    ///
    /// ```
    /// use strided_slice::Slice;
    ///
    /// #[repr(C)]
    /// #[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
    /// struct Vertex {
    ///     position: [f32; 3],
    ///     uv: [f32; 2],
    /// }
    ///
    /// let data: [Vertex; 2] = [
    ///     Vertex {
    ///         position: [1.0, 1.0, 1.0],
    ///         uv: [0.5, 0.5]
    ///     },
    ///     Vertex {
    ///         position: [1.0, 1.0, -1.0],
    ///         uv: [1.0, 0.0]
    ///     },
    /// ];
    ///
    /// // `positions` slice starts at byte offset 0, and stride will be 20 bytes (4 * 3 + 4 * 2).
    /// let positions: Slice<[f32; 3]> = Slice::new(&data, 0, 1);
    ///
    /// // `uvs` slice starts at byte offset 4 * 3, and stride will be 20 bytes (4 * 3 + 4 * 2).
    /// let uvs: Slice<[f32; 2]> = Slice::new(&data, std::mem::size_of::<[f32; 3]>(), 1);
    /// ```
    ///
    /// ## Panics
    ///
    /// This method is a wrapper around
    pub fn new<V: Pod>(data: &'a [V], byte_offset: usize, elt_stride: usize) -> Self {
        Self {
            inner: SliceBase::new_typed(data, byte_offset, elt_stride).unwrap(),
            _phantom: PhantomData,
        }
    }

    /// Create a strided slice starting at the byte offset `offset`.
    ///
    /// This is similar to [`Self::new`], but the offset **and** the stride
    /// must be specified in **bytes**, since no type inference can occur.
    ///
    /// This method will be useful when loading 3D models, with the data layout
    /// not known at compile time.
    ///
    /// ## Panics
    ///
    /// Panics in a similar way to [`Self::new`].
    pub fn raw(data: &'a [u8], byte_offset: usize, byte_stride: usize) -> Self {
        let inner =
            SliceBase::new(data.as_ptr_range(), byte_offset, byte_stride, data.len()).unwrap();
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    /// Create a slice where the stride is the same as the attribute size.
    pub fn native(data: &'a [T]) -> Self {
        Self::new(data, 0, 1)
    }

    /// Create a [`SliceIterator`] for this slice.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use strided_slice::Slice;
    ///
    /// let data = [0, 1, 2, 3];
    /// let slice: Slice<u32> = Slice::new(&data, 0, 1);
    /// println!("{:?}", slice.iter().copied());
    /// ```
    pub fn iter(&'a self) -> SliceIterator<'a, T> {
        SliceIterator::new(self)
    }
}

///
/// Traits implementation
///

impl<'a, Attr: Pod> Deref for Slice<'a, Attr> {
    type Target = SliceBase<Attr>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> std::ops::Index<usize> for Slice<'a, T>
where
    T: Pod,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index ouf of bounds")
    }
}

impl<'a, Attr: Pod> From<&'a [Attr]> for Slice<'a, Attr> {
    fn from(item: &'a [Attr]) -> Self {
        Slice::native(item)
    }
}

impl<'a, T: Pod + Debug> std::fmt::Debug for Slice<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

///
/// Iterator
///

/// Iterator for the [`Slice`] type.
#[derive(Clone, Copy)]
pub struct SliceIterator<'a, T: Pod> {
    /// Start pointer, pointing to the first byte of the slice.
    start: *const u8,
    /// End pointer, pointing one byte **after** the end of the slice.
    end: *const u8,
    stride: usize,
    _phantom_data: PhantomData<&'a T>,
}

impl<'a, T: Pod> SliceIterator<'a, T> {
    fn new(slice: &'a Slice<'a, T>) -> Self {
        let data = slice.inner;
        Self {
            start: data.start,
            end: data.end,
            stride: data.stride(),
            _phantom_data: PhantomData,
        }
    }
}
impl_iterator!(SliceIterator -> &'a T);
