use std::{num::NonZeroUsize, ops::Range};

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

    /// Make the slice point to the referenced element
    ///
    /// ## Example
    ///
    /// ## Safety
    ///
    /// It's possible to use a reference that doesn't belong to the
    /// allocation getting sliced:
    ///
    /// ```rust
    /// let vertices = [
    ///     Vertex {position: [1.0, 0.5, 1.0], uv: [1.0, 1.0]},
    ///     Vertex {position: [1.0, 1.0, 0.5], uv: [0.0, 1.0]},
    /// ];
    /// let dummy = [0.0, 0.0, 0.0];
    ///
    /// // Will panic
    /// let slice: Slice<[f32; 3]> = Slicer::new().offset_of(&dummy).build(&vertices);
    /// ```
    ///
    /// The offset will be checked at runtime via using the slice pointer range.
    pub fn offset_of<T: Sized>(mut self, element: &T) -> Self {
        self.ptr_offset = Some(element as *const _ as *const u8);
        self
    }

    pub fn try_build<'a, Attr: Pod, V: Pod>(
        self,
        data: &'a [V],
    ) -> Result<Slice<'a, Attr>, SliceError> {
        let byte_offset = self.slice_to_offset(data)?;
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
        let byte_offset = self.slice_to_offset(data)?;
        Ok(SliceMut::strided(
            data,
            byte_offset,
            NonZeroUsize::new(self.element_stride).unwrap(),
        ))
    }

    pub fn build_mut<'a, Attr: Pod, V: Pod>(self, data: &'a mut [V]) -> SliceMut<'a, Attr> {
        self.try_build_mut(data).unwrap()
    }

    fn slice_to_offset<V: Sized>(&self, data: &[V]) -> Result<usize, SliceError> {
        if let None = self.ptr_offset {
            return Ok(0);
        }
        let ptr_start = self.ptr_offset.unwrap();
        let ptr_range = data.as_ptr_range();
        let ptr_range: Range<*const u8> = ptr_range.start as *const u8..ptr_range.end as *const u8;
        match ptr_range.contains(&ptr_start) {
            true => Ok((ptr_start as usize)
                .checked_sub(data.as_ptr() as usize)
                .unwrap()),
            false => Err(SliceError::OffsetOutOfBounds),
        }
    }
}
