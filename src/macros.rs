use bytemuck::Pod;

use crate::{Slice, SliceMut};

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

pub struct AttributeSliceBuilder<Attr: Pod>(*const Attr);

impl<Attr: Pod> AttributeSliceBuilder<Attr> {
    pub fn new(elt: &Attr) -> Self {
        Self(elt as *const Attr)
    }
    pub fn build<'a, V: Pod>(&self, data: &'a [V]) -> Slice<'a, Attr> {
        let byte_offset = get_byte_offset(data, self.0 as *const u8);
        Slice::new(data, 1, byte_offset)
    }
    pub fn build_mut<'a, V: Pod>(&self, data: &'a mut [V]) -> SliceMut<'a, Attr> {
        let byte_offset = get_byte_offset(data, self.0 as *const u8);
        SliceMut::new(data, 1, byte_offset)
    }
}

/// Make the slice point to the referenced element
///
/// The slice generic type will be resolved to the type of
/// the attribute passed as the second macro argument.
///
/// ## Example
///
/// ```rust
/// let vertices = [
///     Vertex {position: [1.0, 0.5, 1.0], uv: [1.0, 1.0]},
///     Vertex {position: [1.0, 1.0, 0.5], uv: [0.0, 1.0]},
/// ];
/// let positions = slice_attr!(&vertices, [0].position); // 2 positions
/// let uvs = slice_attr!(&vertices, [1].uv); // 1 uv
/// ```
#[macro_export]
macro_rules! slice_attr {
    ($data:expr, [$index:expr].$( $rest:ident ).*) => {
        {
            use strided_slice::AttributeSliceBuilder;

            let r = &($data[$index].$($rest).*);
            AttributeSliceBuilder::new(r).build(&$data)
        }
    };
}

/// Similar to [`slice_attr`].
///
/// At the opposite of [`slice_attr`], this macro doesn't infer the slice generic.
/// This allows to get a view on a type that has a smaller size than the target attribute.
///
/// ## Example
///
/// ```rust
/// let vertices = [
///     Vertex {position: [1.0, 0.5, 1.0], uv: [1.0, 1.0]},
///     Vertex {position: [1.0, 1.0, 0.5], uv: [0.0, 1.0]},
/// ];
/// // Only slice the x-axis positions
/// let x_positions: Slice<f32> = slice!(&vertices, [0].position[0]);
/// ```
#[macro_export]
macro_rules! slice {
    ($data:expr, [$index:expr].$( $rest:ident ).*) => {
        {
            use strided_slice::get_byte_offset;

            let r = &($data[$index].$($rest).*);
            let byte_offset = get_byte_offset(&$data, r as *const _ as *const u8);
            Slice::new(&$data, 1, byte_offset)
        }
    };
}

/// Similar to [`slice`], but for [`SliceMut`].
#[macro_export]
macro_rules! slice_mut {
    ($data:expr, [$index:expr].$( $rest:ident ).*) => {
        {
            use strided_slice::get_byte_offset;

            let r = &($data[$index].$($rest).*);
            let byte_offset = get_byte_offset(&$data, r as *const _ as *const u8);
            SliceMut::new(&mut $data, 1, byte_offset)
        }
    };
}

/// Similar to [`slice_attr`], but for [`SliceMut`].
#[macro_export]
macro_rules! slice_attr_mut {
    ($data:expr, [$index:expr].$( $rest:ident ).*) => {
        {
            use strided_slice::AttributeSliceBuilder;
            let r = &mut ($data[$index].$($rest).*);
            AttributeSliceBuilder::new(r).build_mut(&mut $data)
        }
    };
}
