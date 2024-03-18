use bytemuck::Pod;
use std::{marker::PhantomData, num::NonZeroUsize, ops::Deref};

use crate::shared_impl::{impl_iterator, SliceData, SliceError};

/// Immutable slice with custom byte stride.
///
/// # Example
///
/// Creating a slice with a stride equal to the element size:
///
/// ```
/// use strided_slice::Slice;
/// let array = [1.0, 2.0, 3.0];
/// let slice: Slice<f32> = Slice::new(&array, 0);
/// ```
///
/// # Important Notes
///
/// - The struct transmust without checking endianness
#[derive(Clone, Copy)]
pub struct Slice<'a, T: Pod> {
    inner: SliceData<T>,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: Pod> std::fmt::Debug for Slice<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slice({})", self.len())
    }
}

impl<'a, T: Pod> Slice<'a, T> {
    /// Try to create a strided slice starting at the byte offset `offset`.
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
    /// let positions: Slice<[f32; 3]> = Slice::try_new(&data, 0).unwrap();
    ///
    /// // `uvs` slice starts at byte offset 4 * 3, and stride will be 20 bytes (4 * 3 + 4 * 2).
    /// let uvs: Slice<[f32; 2]> = Slice::try_new(&data, std::mem::size_of::<[f32; 3]>()).unwrap();
    /// ```
    pub fn try_new<V: Pod>(data: &'a [V], byte_offset: usize) -> Result<Self, SliceError> {
        Ok(Self {
            inner: SliceData::new_typed(data, byte_offset, 1)?,
            _phantom: PhantomData,
        })
    }

    /// Wrapper around [`Self::try_new()`].
    pub fn new<V: Pod>(data: &'a [V], byte_offset: usize) -> Self {
        Self::try_new(data, byte_offset).unwrap()
    }

    /// Similar to [`Self::try_new()`], but allows to pass a number of elements for the stride.
    ///
    /// Note: The stride is expressed in **count** of elements, and **not** in bytes.
    pub fn try_strided<V: Pod>(
        data: &'a [V],
        byte_offset: usize,
        elt_stride: NonZeroUsize,
    ) -> Result<Self, SliceError> {
        Ok(Self {
            inner: SliceData::new_typed(data, byte_offset, elt_stride.get())?,
            _phantom: PhantomData,
        })
    }

    /// Wrapper around [`Self::try_strided()`].
    pub fn strided<V: Pod>(data: &'a [V], byte_offset: usize, elt_stride: NonZeroUsize) -> Self {
        Self::try_strided(data, byte_offset, elt_stride).unwrap()
    }

    // @todo: Non-Zero stride
    pub fn try_raw(
        data: &'a [u8],
        offset: usize,
        stride: NonZeroUsize,
    ) -> Result<Self, SliceError> {
        Ok(Self {
            inner: SliceData::new(data.as_ptr_range(), offset, stride.get(), data.len())?,
            _phantom: PhantomData,
        })
    }

    pub fn raw(data: &'a [u8], offset: usize, stride: NonZeroUsize) -> Self {
        Self::try_raw(data, offset, stride).unwrap()
    }

    pub fn iter(&'a self) -> SliceIterator<'a, T> {
        SliceIterator::new(self)
    }
}

impl<'a, Attr: Pod> Deref for Slice<'a, Attr> {
    type Target = SliceData<Attr>;

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
