use bytemuck::Pod;

use crate::{Slice, SliceMut};

#[doc(hidden)]
/// Get the offset between the start of a slice and a pointer.
///
/// # Panics
///
/// Panics if the `start` argument pointer isn't in the range
/// of the slice start and end pointers.
pub fn get_byte_offset<V: Sized>(data: &[V], start: *const u8) -> usize {
    let ptr_range = data.as_ptr_range();
    let ptr_range = ptr_range.start as *const u8..ptr_range.end as *const u8;
    if !ptr_range.contains(&start) {
        panic!(
            "referenced attribute at address {} doesn't belong in slice at adress range ({}, {})",
            start as usize, ptr_range.start as usize, ptr_range.end as usize
        );
    }
    let end: usize = start as *const _ as usize;
    end.checked_sub(data.as_ptr() as usize).unwrap()
}

#[doc(hidden)]
/// Slice builder.
///
/// This is used internally by the [`crate::slice_attr!`] an [`crate::slice_attr_mut!`]
/// for type inference.
pub struct SliceBuilder<Attr: Pod> {
    start: *const Attr,
}

impl<Attr: Pod> SliceBuilder<Attr> {
    pub fn new(start: &Attr) -> Self {
        Self {
            start: start as *const Attr,
        }
    }
    pub fn build<'a, V: Pod>(&self, data: &'a [V]) -> Slice<'a, Attr> {
        let byte_offset = get_byte_offset(data, self.start as *const u8);
        Slice::new(data, byte_offset)
    }
    pub fn build_mut<'a, V: Pod>(&self, data: &'a mut [V]) -> SliceMut<'a, Attr> {
        let byte_offset = get_byte_offset(data, self.start as *const u8);
        SliceMut::new(data, byte_offset)
    }
}
