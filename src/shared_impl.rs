use std::marker::PhantomData;

use bytemuck::Pod;

#[derive(Copy, Clone, PartialEq)]
pub enum SliceError {
    OffsetOutOfRange,
    SliceSizeNotMatchingStride,
    StrideOOB,
    AlignmentFault,
}

impl std::fmt::Debug for SliceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OffsetOutOfRange => write!(f, "Offset out of range"),
            Self::SliceSizeNotMatchingStride => {
                write!(f, "Slice byte len isn't a multiple of the stride")
            }
            Self::StrideOOB => write!(f, "Stride out-of-bounds"),
            Self::AlignmentFault => write!(f, "Alignment fault"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct SliceData<Attr: Sized> {
    data: *const u8,
    len: usize,
    stride: usize,
    _phantom: PhantomData<Attr>,
}

impl<Attr: Sized> SliceData<Attr> {
    pub(crate) fn try_new<V: Pod>(data: &[V], offset: usize) -> Result<Self, SliceError> {
        let ptr = data.as_ptr() as *const u8;
        let stride = std::mem::size_of::<V>();
        let size = stride * data.len();
        Self::try_raw_ptr(ptr, stride, offset, size)
    }

    pub(crate) fn try_raw(data: &[u8], stride: usize, offset: usize) -> Result<Self, SliceError> {
        let ptr = data.as_ptr() as *const u8;
        Self::try_raw_ptr(ptr, stride, offset, data.len())
    }

    pub(crate) fn start(&self) -> *const u8 {
        self.data
    }

    pub(crate) fn end(&self) -> *const u8 {
        if self.len > 0 {
            self.get(self.len - 1).unwrap()
        } else {
            self.data
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, index: usize) -> Option<*const u8> {
        if index < self.len() {
            let start = self.stride * index;
            Some(unsafe { self.data.offset(start as isize) })
        } else {
            None
        }
    }

    pub fn stride(&self) -> usize {
        self.stride
    }

    // @todo: Non-Zero stride
    fn try_raw_ptr(
        ptr: *const u8,
        stride: usize,
        offset: usize,
        bytes: usize,
    ) -> Result<Self, SliceError> {
        // Empty slice are allowed, but we need to ensure that
        // the offset and stride are valid.
        if offset + std::mem::size_of::<Attr>() > stride {
            Err(SliceError::OffsetOutOfRange)
        } else if bytes > 0 && stride > bytes {
            Err(SliceError::StrideOOB)
        } else if unsafe { ptr.add(offset).align_offset(std::mem::align_of::<Attr>()) != 0 } {
            Err(SliceError::AlignmentFault)
        } else if bytes % stride != 0 {
            Err(SliceError::SliceSizeNotMatchingStride)
        } else {
            Ok(Self {
                data: unsafe { ptr.offset(offset as isize) },
                len: bytes / stride,
                stride,
                _phantom: PhantomData,
            })
        }
    }
}

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
                    self.start = self.start.offset(self.stride as isize);
                    ret
                }
            }
        }
    };
}

pub(super) use impl_iterator;
