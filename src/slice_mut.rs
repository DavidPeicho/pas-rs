use bytemuck::Pod;
use std::marker::PhantomData;

use crate::shared_impl::{impl_iterator, SliceData, SliceError};

/// Mutable slice
///
/// # Important Notes
///
/// - The struct transmust without checking endianness
pub struct SliceMut<'a, T: Pod> {
    inner: SliceData<T>,
    _phantom: PhantomData<&'a mut T>,
}

impl<'a, T: Pod> SliceMut<'a, T> {
    pub fn try_new<V: Pod>(data: &'a [V], offset: usize) -> Result<Self, SliceError> {
        Ok(Self {
            inner: SliceData::try_new(data, offset)?,
            _phantom: PhantomData,
        })
    }

    // @todo: Non-Zero stride
    pub fn try_raw(data: &'a [u8], stride: usize, offset: usize) -> Result<Self, SliceError> {
        Ok(Self {
            inner: SliceData::try_raw(data, stride, offset)?,
            _phantom: PhantomData,
        })
    }

    pub fn new<V: Pod>(data: &'a [V], offset: usize) -> Self {
        Self::try_new(data, offset).unwrap()
    }

    pub fn new_raw(data: &'a [u8], stride: usize, offset: usize) -> Self {
        Self::try_raw(data, stride, offset).unwrap()
    }

    pub fn get(&self, index: usize) -> Option<&'a T> {
        if let Some(ptr) = self.inner.get(index) {
            Some(unsafe { std::mem::transmute::<_, &T>(ptr) })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if let Some(ptr) = self.inner.get(index) {
            Some(unsafe { std::mem::transmute::<_, &mut T>(ptr) })
        } else {
            None
        }
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
            let ptr = self.inner.get(i).unwrap() as *mut u8;
            let other_ptr = unsafe { bytes.as_ptr().offset((i * other_stride) as isize) };
            unsafe {
                ptr.copy_from_nonoverlapping(other_ptr, other_stride);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn iter(&'a self) -> SliceMutIterator<'a, T> {
        SliceMutIterator::new(self)
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

///
/// Test
///

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C)]
    #[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
    struct Vertex {
        position: [f32; 3],
        uv: [f32; 2],
    }

    fn data() -> Vec<Vertex> {
        vec![
            Vertex {
                position: [1.0, -1.0, 1.0],
                uv: [0.25, 0.5],
            },
            Vertex {
                position: [-1.0, 1.0, 0.0],
                uv: [-1.0, 0.25],
            },
        ]
    }

    #[test]
    fn slice_count() {}

    #[test]
    fn mutable_indexing() {
        let mut vertices = data();

        let mut slice: SliceMut<[f32; 3]> = SliceMut::new(&mut vertices, 0);
        assert_eq!(slice[0], [1.0, -1.0, 1.0]);
        assert_eq!(slice[1], [-1.0, 1.0, 0.0]);

        // Changing index 0 doesn't affect other index.
        slice[0] = [4.0, 3.0, 1.0];
        assert_eq!(slice[0], [4.0, 3.0, 1.0]);
        assert_eq!(slice[1], [-1.0, 1.0, 0.0]);

        // Changing index 1 doesn't affect other index.
        slice[1] = [11.0, 10.0, 9.0];
        assert_eq!(slice[0], [4.0, 3.0, 1.0]);
        assert_eq!(slice[1], [11.0, 10.0, 9.0]);
    }

    #[test]
    fn mutable_iter() {
        let mut vertices = data();
        let slice: SliceMut<[f32; 3]> = SliceMut::new(&mut vertices, 0);
        {
            let mut iter = slice.iter();
            assert_eq!(*iter.next().unwrap(), [1.0, -1.0, 1.0]);
            assert_eq!(*iter.next().unwrap(), [-1.0, 1.0, 0.0]);
            assert_eq!(iter.next(), None);
        }

        let slice: SliceMut<[f32; 2]> =
            SliceMut::new(&mut vertices, std::mem::size_of::<[f32; 3]>());
        {
            let mut iter = slice.iter();
            assert_eq!(*iter.next().unwrap(), [0.25, 0.5]);
            assert_eq!(*iter.next().unwrap(), [-1.0, 0.25]);
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn copy_from_slice() {
        let mut vertices = data();
        let slice: SliceMut<[f32; 3]> = SliceMut::new(&mut vertices, 0);

        slice.copy_from_slice(&[[0.1_f32, 0.2, 0.3]]);
        assert_eq!(slice[0], [0.1_f32, 0.2, 0.3]);
        slice.copy_from_slice(&[[0.9_f32, 0.8, 0.7], [0.6, 0.5, 0.4]]);
        assert_eq!(slice[0], [0.9, 0.8, 0.7]);
        assert_eq!(slice[1], [0.6, 0.5, 0.4]);

        let slice: SliceMut<[f32; 2]> =
            SliceMut::new(&mut vertices, std::mem::size_of::<[f32; 3]>());
        slice.copy_from_slice(&[[0.1_f32, 0.2]]);
        assert_eq!(slice[0], [0.1, 0.2]);
        slice.copy_from_slice(&[[0.1_f32, 0.2], [0.3, 0.4]]);
        assert_eq!(slice[0], [0.1, 0.2]);
        assert_eq!(slice[1], [0.3, 0.4]);
    }
}
