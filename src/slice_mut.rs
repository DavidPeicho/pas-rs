use bytemuck::Pod;
use std::{fmt::Debug, marker::PhantomData, ops::Deref};

use crate::shared_impl::{impl_iterator, SliceBase};

/// Mutable slice
///
/// For more information, have a look at the [`crate::Slice`] type.
pub struct SliceMut<'a, Attr: Pod> {
    inner: SliceBase<Attr>,
    _phantom: PhantomData<&'a mut Attr>,
}

impl<'a, Attr: Pod + Debug> std::fmt::Debug for SliceMut<'a, Attr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'a, Attr: Pod> SliceMut<'a, Attr> {
    /// Mutable version of [`crate::Slice::new()`].
    pub fn new<V: Pod>(data: &'a mut [V], byte_offset: usize, elt_stride: usize) -> Self {
        Self {
            inner: SliceBase::new_typed(data, byte_offset, elt_stride).unwrap(),
            _phantom: PhantomData,
        }
    }

    /// Mutable version of [`crate::Slice::raw()`].
    pub fn raw(data: &'a [u8], byte_offset: usize, byte_stride: usize) -> Self {
        let inner =
            SliceBase::new(data.as_ptr_range(), byte_offset, byte_stride, data.len()).unwrap();
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    /// Create a mutable slice where the stride is the same as the attribute size.
    pub fn native(data: &'a mut [Attr]) -> Self {
        Self::new(data, 0, 1)
    }

    /// Mutable version of [`crate::SliceBase::get()`].
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Attr> {
        self.inner
            .get_ptr(index)
            .map(|ptr| unsafe { std::mem::transmute::<_, &mut Attr>(ptr) })
    }

    /// Copies all elements from `src`` into `self``, using a memcpy.
    ///
    /// At the opposite of the std `copy_from_slice`:
    /// * The length of `src` **doesn't** need to match the length of `self`
    /// * The `src` parameter can contain elements whose byte-size is smaller than `Attr`
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use strided_slice::SliceMut;
    ///
    /// let mut dest = [0_u32, 0, 0, 0];
    /// let slice: SliceMut<u32> = SliceMut::new(&mut dest, 0, 1);
    ///
    /// slice.copy_from_slice(&[1_u8, 2]);
    /// println!("{:?}", slice); // Prints `[1, 2]`
    ///
    /// slice.copy_from_slice(&[3_u8, 4, 5, 6]);
    /// println!("{:?}", slice); // Prints `[3, 4, 5, 6]`
    /// ```
    ///
    /// ## Panics
    ///
    /// * Panics if the length of `src` is bigger than the length of `self`
    /// * Panics if the `src` inner format is bigger than the slice attribute format
    pub fn copy_from_slice<V: Pod>(&self, src: &[V]) {
        let other_stride = std::mem::size_of::<V>();
        // @todo: Checking the size at compile time would be nice.
        assert!(
            other_stride <= std::mem::size_of::<Attr>(),
            "`data` type is {} bytes, but slice format expected at most {} bytes",
            std::mem::size_of::<V>(),
            std::mem::size_of::<Attr>()
        );

        let count = self.len();
        let other_count = src.len();
        assert!(
            other_count <= count,
            "`data` too large. Found slice with {} elements, but expected at most {}",
            other_count,
            count
        );

        let bytes: &[u8] = bytemuck::cast_slice(src);
        for i in 0..other_count {
            let ptr = self.inner.get_ptr(i).unwrap() as *mut u8;
            let other_ptr = unsafe { bytes.as_ptr().add(i * other_stride) };
            unsafe {
                ptr.copy_from_nonoverlapping(other_ptr, other_stride);
            }
        }
    }

    /// Create a [`SliceMutIterator`] for this slice.
    pub fn iter(&'a self) -> SliceMutIterator<'a, Attr> {
        SliceMutIterator::new(self)
    }
}

///
/// Traits implementation
///

impl<'a, Attr: Pod> Deref for SliceMut<'a, Attr> {
    type Target = SliceBase<Attr>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> std::ops::Index<usize> for SliceMut<'a, T>
where
    T: Pod,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index ouf of bounds")
    }
}

impl<'a, T> std::ops::IndexMut<usize> for SliceMut<'a, T>
where
    T: Pod,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("index ouf of bounds")
    }
}

impl<'a, Attr: Pod> From<&'a mut [Attr]> for SliceMut<'a, Attr> {
    fn from(item: &'a mut [Attr]) -> Self {
        SliceMut::native(item)
    }
}

///
/// Iterator
///

/// Iterator for the [`SliceMut`] type.
#[derive(Clone, Copy)]
pub struct SliceMutIterator<'a, T: Pod> {
    start: *const u8,
    end: *const u8,
    stride: usize,
    _phantom_data: PhantomData<&'a mut T>,
}

impl<'a, T: Pod> SliceMutIterator<'a, T> {
    fn new(slice: &'a SliceMut<'a, T>) -> Self {
        let data = slice.inner;
        Self {
            start: data.start,
            end: data.end,
            stride: data.stride(),
            _phantom_data: PhantomData,
        }
    }
}
impl_iterator!(SliceMutIterator -> &'a mut T);
