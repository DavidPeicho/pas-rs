use bytemuck::Pod;
use std::{fmt::Debug, marker::PhantomData, num::NonZeroUsize, ops::Deref};

use crate::shared_impl::{impl_iterator, SliceBase};

/// Mutable slice
///
/// For more information, have a look at the [`Slice`] type.
pub struct SliceMut<'a, T: Pod> {
    inner: SliceBase<T>,
    _phantom: PhantomData<&'a mut T>,
}

impl<'a, T: Pod + Debug> std::fmt::Debug for SliceMut<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'a, T: Pod> SliceMut<'a, T> {
    /// Mutable version of [`Slice::new()`].
    pub fn new<V: Pod>(data: &'a mut [V], byte_offset: usize, elt_stride: usize) -> Self {
        Self {
            inner: SliceBase::new_typed(data, byte_offset, elt_stride).unwrap(),
            _phantom: PhantomData,
        }
    }

    pub fn raw(data: &'a [u8], offset: usize, stride: NonZeroUsize) -> Self {
        let inner = SliceBase::new(data.as_ptr_range(), offset, stride.get(), data.len()).unwrap();
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.inner
            .get_ptr(index)
            .map(|ptr| unsafe { std::mem::transmute::<_, &mut T>(ptr) })
    }

    pub fn copy_from_slice<V: Pod>(&self, other_data: &[V]) {
        let other_stride = std::mem::size_of::<V>();
        // @todo: Checking the size at compile time would be nice.
        assert!(
            other_stride <= std::mem::size_of::<T>(),
            "`data` type is {} bytes, but slice format expected at most {} bytes",
            std::mem::size_of::<V>(),
            std::mem::size_of::<T>()
        );

        let count = self.len();
        let other_count = other_data.len();
        assert!(
            other_count <= count,
            "`data` too large. Found slice with {} elements, but expected at most {}",
            other_count,
            count
        );

        let bytes: &[u8] = bytemuck::cast_slice(other_data);
        for i in 0..other_count {
            let ptr = self.inner.get_ptr(i).unwrap() as *mut u8;
            let other_ptr = unsafe { bytes.as_ptr().add(i * other_stride) };
            unsafe {
                ptr.copy_from_nonoverlapping(other_ptr, other_stride);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&'a self) -> SliceMutIterator<'a, T> {
        SliceMutIterator::new(self)
    }

    pub fn test<V: Pod>(data: &'a mut [V], elt_stride: usize, t: &mut T) -> SliceMut<'a, T> {
        let r = &t;
        let ptr = r as *const _ as *const u8;
        let offset = (ptr as usize).checked_sub(data.as_ptr() as usize).unwrap();
        Self::new(data, elt_stride, offset)
    }
}

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

///
/// Iterator
///

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
            start: data.start(),
            end: data.end(),
            stride: data.stride(),
            _phantom_data: PhantomData,
        }
    }
}
impl_iterator!(SliceMutIterator -> &'a mut T);
