use bytemuck::Pod;
use std::{fmt::Debug, marker::PhantomData, num::NonZeroUsize, ops::Deref};

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

impl<'a, T: Pod + Debug> std::fmt::Debug for Slice<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
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
    /// let positions: Slice<[f32; 3]> = Slice::new(&data, 1, 0).unwrap();
    ///
    /// // `uvs` slice starts at byte offset 4 * 3, and stride will be 20 bytes (4 * 3 + 4 * 2).
    /// let uvs: Slice<[f32; 2]> = Slice::try_new(&data, 1, std::mem::size_of::<[f32; 3]>()).unwrap();
    /// ```
    ///
    /// ## Panics
    ///
    /// TODO
    pub fn new<V: Pod>(data: &'a [V], byte_offset: usize, elt_stride: usize) -> Self {
        Self {
            inner: SliceBase::new_typed(data, byte_offset, elt_stride).unwrap(),
            _phantom: PhantomData,
        }
    }

    // @todo: Non-Zero stride
    pub fn raw(data: &'a [u8], offset: usize, stride: NonZeroUsize) -> Self {
        Self {
            inner: SliceBase::new(data.as_ptr_range(), offset, stride.get(), data.len()).unwrap(),
            _phantom: PhantomData,
        }
    }

    pub fn iter(&'a self) -> SliceIterator<'a, T> {
        SliceIterator::new(self)
    }
}

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

///
/// Iterator
///

#[derive(Clone, Copy)]
pub struct SliceIterator<'a, T: Pod> {
    start: *const u8,
    end: *const u8,
    stride: usize,
    _phantom_data: PhantomData<&'a T>,
}

impl<'a, T: Pod> SliceIterator<'a, T> {
    fn new(slice: &'a Slice<'a, T>) -> Self {
        let data = slice.inner;
        Self {
            start: data.start(),
            end: data.end(),
            stride: data.stride(),
            _phantom_data: PhantomData,
        }
    }
}
impl_iterator!(SliceIterator -> &'a T);
