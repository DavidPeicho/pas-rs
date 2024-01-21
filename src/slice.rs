use bytemuck::Pod;
use std::marker::PhantomData;

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

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn iter(&'a self) -> SliceIterator<'a, T> {
        SliceIterator::new(self)
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
    fn slice_count() {
        let vertices = data();

        let slice: Slice<f32> = Slice::new(&[] as &[f32], 0);
        assert_eq!(slice.len(), 0);

        let slice: Slice<f32> = Slice::new(&vertices, 0);
        assert_eq!(slice.len(), 2);

        let slice: Slice<[f32; 2]> = Slice::new(&vertices, std::mem::size_of::<[f32; 3]>());
        assert_eq!(slice.len(), 2);

        let slice: Slice<[f32; 2]> = Slice::new(&vertices[1..], std::mem::size_of::<[f32; 3]>());
        assert_eq!(slice.len(), 1);

        let positions: [f32; 3] = [1.0, 2.0, 3.0];
        let slice: Slice<f32> = Slice::new(&positions, 0);
        assert_eq!(slice.len(), 3);

        let positions: [f32; 3] = [1.0, 2.0, 3.0];
        let slice: Slice<f32> = Slice::new(&positions, 0);
        assert_eq!(slice.len(), 3);
    }

    #[test]
    fn from_raw_part_invalid_offset() {
        let data: Vec<u8> = vec![0, 200, 100];
        let error = Slice::<f32>::try_raw(&data, 2, 3).unwrap_err();
        assert_eq!(error, SliceError::OffsetOutOfRange);
    }

    #[test]
    fn from_raw_part_invalid_stride() {
        let data: Vec<u8> = vec![0, 200, 100, 255];
        let error = Slice::<f32>::try_raw(&data, 8, 0).unwrap_err();
        assert_eq!(error, SliceError::StrideOOB);
    }

    #[test]
    fn indexing() {
        let vertices = data();
        let slice: Slice<[f32; 3]> = Slice::new(&vertices, 0);
        assert_eq!(slice[0], [1.0, -1.0, 1.0]);
        assert_eq!(slice[1], [-1.0, 1.0, 0.0]);

        let slice: Slice<[f32; 2]> = Slice::new(&vertices[1..], std::mem::size_of::<[f32; 3]>());
        assert_eq!(slice[0], [-1.0, 0.25]);
        assert_eq!(slice.get(1), None);
    }

    #[test]
    fn iter() {
        let vertices = data();
        {
            let slice: Slice<[f32; 3]> = Slice::new(&vertices, 0);
            let mut iter = slice.iter();
            assert_eq!(*iter.next().unwrap(), [1.0, -1.0, 1.0]);
            assert_eq!(*iter.next().unwrap(), [-1.0, 1.0, 0.0]);
            assert_eq!(iter.next(), None);
        }
        {
            let slice: Slice<[f32; 2]> = Slice::new(&vertices, std::mem::size_of::<[f32; 3]>());
            let mut iter = slice.iter();
            assert_eq!(*iter.next().unwrap(), [0.25, 0.5]);
            assert_eq!(*iter.next().unwrap(), [-1.0, 0.25]);
            assert_eq!(iter.next(), None);
        }
    }
}
