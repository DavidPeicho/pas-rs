use std::num::NonZeroUsize;

use bytemuck::Pod;

use crate::{Slice, SliceError, SliceMut};

pub struct Slicer {
    ptr_offset: Option<*const u8>,
    element_stride: usize,
}

impl Slicer {
    pub fn new() -> Self {
        Self {
            ptr_offset: None,
            element_stride: 1,
        }
    }

    pub fn stride(mut self, count: usize) -> Self {
        self.element_stride = count;
        self
    }

    pub fn offset_of<T: Sized>(mut self, element: &T) -> Self {
        self.ptr_offset = Some(&element as *const _ as *const u8);
        self
    }

    pub fn try_build<'a, Attr: Pod, V: Pod>(
        self,
        data: &'a [V],
    ) -> Result<Slice<'a, Attr>, SliceError> {
        println!("DKKDWKFKKWDKWDKWDK");
        let byte_offset = if let Some(ptr) = self.ptr_offset {
            let ptr_range = data.as_ptr_range();
            let ptr_range = ptr_range.start as *const u8..ptr_range.end as *const u8;
            println!(
                "{}, {}, {}",
                ptr as usize, ptr_range.start as usize, ptr_range.end as usize
            );
            if !ptr_range.contains(&ptr) {
                return Err(SliceError::OffsetOutOfBounds);
            }
            Ok((ptr as usize).checked_sub(data.as_ptr() as usize).unwrap())
        } else {
            Ok(0)
        }?;

        Ok(Slice::strided(
            data,
            byte_offset,
            NonZeroUsize::new(self.element_stride).unwrap(),
        ))
    }

    pub fn build<'a, Attr: Pod, V: Pod>(self, data: &'a [V]) -> Slice<'a, Attr> {
        self.try_build(data).unwrap()
    }

    pub fn try_build_mut<'a, Attr: Pod, V: Pod>(
        self,
        data: &'a mut [V],
    ) -> Result<SliceMut<'a, Attr>, SliceError> {
        let byte_offset = if let Some(ptr) = self.ptr_offset {
            let ptr_range = data.as_ptr_range();
            let ptr_range = ptr_range.start as *const u8..ptr_range.end as *const u8;
            if !ptr_range.contains(&ptr) {
                return Err(SliceError::OffsetOutOfBounds);
            }
            Ok((ptr as usize).checked_sub(data.as_ptr() as usize).unwrap())
        } else {
            Ok(0)
        }?;

        Ok(SliceMut::strided(
            data,
            byte_offset,
            NonZeroUsize::new(self.element_stride).unwrap(),
        ))
    }

    pub fn build_mut<'a, Attr: Pod, V: Pod>(self, data: &'a mut [V]) -> SliceMut<'a, Attr> {
        self.try_build_mut(data).unwrap()
    }
}
