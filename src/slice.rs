use bytemuck::Pod;
use std::marker::PhantomData;

pub enum SliceResult {
    OffsetOutOfRange,
    SliceSizeNotMatchingStride,
}

impl std::fmt::Debug for SliceResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OffsetOutOfRange => write!(f, "Offset out of range"),
            Self::SliceSizeNotMatchingStride => {
                write!(f, "Slice byte len isn't a multiple of the stride")
            }
        }
    }
}

/// Immutable slice with custom byte stride.
///
/// # Example
///
/// Creating a slice with a stride equal to the element size:
///
/// ```
/// use strided_slice::Slice;
/// let array = [1.0, 2.0, 3.0];
/// let slice: Slice<f32> = Slice::new(&array);
/// ```
///
/// # Important Notes
///
/// - The struct transmust without checking endianness
#[derive(Clone, Copy)]
pub struct Slice<'a, T: Pod> {
    data: &'a [u8],
    stride: usize,
    _phantom_data: PhantomData<&'a T>,
}

fn try_slice<T>(
    slice_bytes: usize,
    element_bytes: usize,
    offset: usize,
) -> Result<(), SliceResult> {
    if offset + std::mem::size_of::<T>() >= element_bytes {
        Err(SliceResult::OffsetOutOfRange)
    } else if slice_bytes % element_bytes != 0 {
        Err(SliceResult::SliceSizeNotMatchingStride)
    } else {
        Ok(())
    }
}

fn ensure_size_and_alignment<Src, Dst>(offset: usize) -> Result<(), SliceResult> {
    try_slice::<Src>(
        std::mem::size_of::<Dst>(),
        std::mem::size_of::<Dst>(),
        offset,
    )
}

impl<'a, T: Pod> Slice<'a, T> {
    pub fn new<V: Pod>(data: &'a [V]) -> Self {
        Self::new_with_offset(data, 0)
    }

    pub fn new_with_offset<V: Pod>(data: &'a [V], offset: usize) -> Self {
        ensure_size_and_alignment::<T, V>(offset).unwrap();
        let bytes: &[u8] = bytemuck::cast_slice(data);
        Self {
            data: &bytes[offset..],
            stride: std::mem::size_of::<V>(),
            _phantom_data: PhantomData,
        }
    }

    // @todo: Non-Zero stride
    pub fn from_raw(data: &'a [u8], stride: usize, offset: usize) -> Self {
        try_slice::<T>(data.len(), stride, offset).unwrap();
        Self {
            data: &data[offset..],
            stride,
            _phantom_data: PhantomData,
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            return None;
        }
        let start = self.stride * index;
        let ptr = self.data.as_ptr();
        Some(unsafe { std::mem::transmute::<_, &T>(ptr.offset(start as isize)) })
    }

    pub fn len(&self) -> usize {
        self.data.len() / self.stride
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
    data: &'a [u8],
    stride: usize,
    _phantom_data: PhantomData<&'a T>,
}

impl<'a, T: Pod> SliceIterator<'a, T> {
    pub fn new(slice: &'a Slice<'a, T>) -> Self {
        let data = slice.data;
        Self {
            data,
            stride: slice.stride,
            _phantom_data: PhantomData,
        }
    }
}

impl<'a, T: Pod> Iterator for SliceIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.data.len() < std::mem::size_of::<T>() {
            return None;
        }
        let result: &'a T = unsafe { std::mem::transmute::<_, &'a T>(self.data.as_ptr()) };
        self.data = &self.data[usize::min(self.stride, self.data.len())..];
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
    fn slice_count() {
        let vertices = data();

        let slice: Slice<f32> = Slice::new(&[] as &[f32]);
        assert_eq!(slice.len(), 0);

        let slice: Slice<f32> = Slice::new(&vertices);
        assert_eq!(slice.len(), 2);

        let slice: Slice<[f32; 3]> = Slice::new(&vertices);
        assert_eq!(slice.len(), 2);

        let positions = [1.0, 2.0, 3.0];
        let slice: Slice<f32> = Slice::new(&positions);
        assert_eq!(slice.len(), 3);
    }

    #[test]
    #[should_panic]
    fn from_raw_part_invalid_offset() {
        let data: Vec<u8> = vec![0, 200, 100];
        Slice::<f32>::from_raw(&data, 2, 3);
    }

    #[test]
    #[should_panic]
    fn from_raw_part_invalid_stride() {
        let data: Vec<u8> = vec![0, 200, 100, 255];
        Slice::<f32>::from_raw(&data, 8, 0);
    }

    #[test]
    fn immutable_indexing() {
        let vertices = data();
        let slice: Slice<[f32; 3]> = Slice::new(&vertices);
        assert_eq!(slice[0], [1.0, -1.0, 1.0]);
        assert_eq!(slice[1], [-1.0, 1.0, 0.0]);
    }

    #[test]
    fn immutable_iter() {
        let vertices = data();
        {
            let slice: Slice<[f32; 3]> = Slice::new_with_offset(&vertices, 0);
            let mut iter = slice.iter();
            assert_eq!(*iter.next().unwrap(), [1.0, -1.0, 1.0]);
            assert_eq!(*iter.next().unwrap(), [-1.0, 1.0, 0.0]);
            assert_eq!(iter.next(), None);
        }
        {
            let slice: Slice<[f32; 2]> =
                Slice::new_with_offset(&vertices, std::mem::size_of::<[f32; 3]>());
            let mut iter = slice.iter();
            assert_eq!(*iter.next().unwrap(), [0.25, 0.5]);
            assert_eq!(*iter.next().unwrap(), [-1.0, 0.25]);
            assert_eq!(iter.next(), None);
        }
    }
}
