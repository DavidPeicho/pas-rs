use bytemuck::Pod;
use std::marker::PhantomData;

/// Mutable slice
///
/// # Important Notes
///
/// - The struct transmust without checking endianness
pub struct SliceMut<'a, T: Pod> {
    data: &'a mut [u8],
    stride: usize,
    _phantom_data: PhantomData<&'a mut T>,
}

impl<'a, T: Pod> SliceMut<'a, T> {
    pub fn new(data: &'a mut [u8], stride: usize, offset: usize) -> Self {
        Self {
            data: &mut data[offset..],
            stride,
            _phantom_data: PhantomData,
        }
    }

    pub fn from_slice_offset<V: Pod>(data: &'a mut [V], offset: usize) -> Self {
        let bytes: &mut [u8] = bytemuck::cast_slice_mut(data);
        Self::new(bytes, std::mem::size_of::<V>(), offset)
    }

    pub fn from_slice<V: Pod>(data: &'a mut [V]) -> Self {
        Self::from_slice_offset(data, 0)
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            return None;
        }
        let start = self.stride * index;
        let ptr = self.data.as_ptr();
        Some(unsafe { std::mem::transmute::<_, &T>(ptr.offset(start as isize)) })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len() {
            return None;
        }
        let start = self.stride * index;
        let ptr = self.data.as_mut_ptr();
        Some(unsafe { std::mem::transmute::<_, &mut T>(ptr.offset(start as isize)) })
    }

    pub fn set<V: Pod>(&mut self, other_data: &[V]) {
        let other_stride = std::mem::size_of::<V>();
        assert!(
            other_stride <= std::mem::size_of::<T>(),
            "`data` type is {} bytes, but slice format expected at most {} bytes",
            std::mem::size_of::<V>(),
            std::mem::size_of::<T>()
        );

        let count = self.len();
        let other_count = other_data.len();
        assert!(
            count <= self.len(),
            "`data` too large. Found slice with {} elements, but expected at most {}",
            other_count,
            count
        );

        let ptr = self.data.as_mut_ptr();
        let bytes: &[u8] = bytemuck::cast_slice(other_data);
        let other_ptr = bytes.as_ptr();
        for i in 0..count {
            let dst_start = self.stride * i;
            let src_start = i * other_stride;
            unsafe {
                ptr.offset(dst_start as isize)
                    .copy_from_nonoverlapping(other_ptr.offset(src_start as isize), other_stride);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.data.len() / self.stride
    }

    pub fn iter(&'a mut self) -> SliceMutIterator<'a, T> {
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
    data: &'a mut [u8],
    index: usize,
    stride: usize,
    _phantom_data: PhantomData<&'a mut T>,
}

impl<'a, T: Pod> SliceMutIterator<'a, T> {
    pub fn new(slice: &'a mut SliceMut<'a, T>) -> Self {
        Self {
            data: &mut slice.data,
            index: 0,
            stride: slice.stride,
            _phantom_data: PhantomData,
        }
    }
}

impl<'a, T: Pod> Iterator for SliceMutIterator<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        let offset = self.index * self.stride;
        if offset >= self.data.len() {
            return None;
        }
        self.index = self.index + 1;
        let ptr = self.data.as_ptr();
        let result: &mut T =
            unsafe { std::mem::transmute::<_, &mut T>(ptr.offset(offset as isize)) };
        Some(result)
    }
}

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

        let mut slice: SliceMut<[f32; 3]> = SliceMut::from_slice_offset(&mut vertices, 0);
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
        let mut slice: SliceMut<[f32; 3]> = SliceMut::from_slice_offset(&mut vertices, 0);
        {
            let mut iter = slice.iter();
            assert_eq!(*iter.next().unwrap(), [1.0, -1.0, 1.0]);
            assert_eq!(*iter.next().unwrap(), [-1.0, 1.0, 0.0]);
            assert_eq!(iter.next(), None);
        }
        {
            let mut slice: SliceMut<[f32; 2]> =
                SliceMut::from_slice_offset(&mut vertices, std::mem::size_of::<[f32; 3]>());
            let mut iter = slice.iter();
            assert_eq!(*iter.next().unwrap(), [0.25, 0.5]);
            assert_eq!(*iter.next().unwrap(), [-1.0, 0.25]);
            assert_eq!(iter.next(), None);
        }
    }
}
