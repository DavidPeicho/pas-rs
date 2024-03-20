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

pub struct AttributeSliceBuilder<Attr: Pod> {
    start: *const Attr,
    elt_stride: usize,
}

impl<Attr: Pod> AttributeSliceBuilder<Attr> {
    pub fn new(start: &Attr, elt_stride: usize) -> Self {
        Self {
            start: start as *const Attr,
            elt_stride,
        }
    }
    pub fn build<'a, V: Pod>(&self, data: &'a [V]) -> Slice<'a, Attr> {
        let byte_offset = get_byte_offset(data, self.start as *const u8);
        Slice::new(data, byte_offset, self.elt_stride)
    }
    pub fn build_mut<'a, V: Pod>(&self, data: &'a mut [V]) -> SliceMut<'a, Attr> {
        let byte_offset = get_byte_offset(data, self.start as *const u8);
        SliceMut::new(data, byte_offset, self.elt_stride)
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
///
/// The stride, in element count, can be passed as a first argument:
///
/// ```rust
/// // Stride of 2 vertex
/// let positions = slice_attr!(2, &vertices, [0].position);
/// ````
#[macro_export]
macro_rules! slice_attr {
    ($stride:expr, $data:expr, $( $rest:tt )*) => {
        {
            use strided_slice::AttributeSliceBuilder;

            let r = &($data$($rest)*);
            AttributeSliceBuilder::new(r, $stride).build(&$data)
        }
    };
    ($data:expr, $( $rest:tt )*) => {
        slice_attr!(1, $data, $($rest)*)
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
    ($stride:expr, $data:expr, $( $rest:tt )*) => {
        {
            use strided_slice::{Slice, get_byte_offset};

            let r = &($data$($rest)*);
            let byte_offset = get_byte_offset(&$data, r as *const _ as *const u8);
            Slice::new(&$data, byte_offset, $stride)
        }
    };
    ($data:expr, $( $rest:tt )*) => {
        slice!(1, $data, $($rest)*)
    };
}

/// Similar to [`slice`], but for [`SliceMut`].
#[macro_export]
macro_rules! slice_mut {
    ($stride:expr, $data:expr, $( $rest:tt )*) => {
        {
            use strided_slice::{get_byte_offset, SliceMut};

            let r = &($data$($rest)*);
            let byte_offset = get_byte_offset(&$data, r as *const _ as *const u8);
            SliceMut::new(&mut $data, byte_offset, $stride)
        }
    };
    ($data:expr, $( $rest:tt )*) => {
        slice_mut!(1, $data, $($rest)*)
    };
}

/// Similar to [`slice_attr`], but for [`SliceMut`].
#[macro_export]
macro_rules! slice_attr_mut {
    ($stride:expr, $data:expr, $( $rest:tt )*) => {
        {
            use strided_slice::AttributeSliceBuilder;
            let r = &($data$($rest)*);
            AttributeSliceBuilder::new(r, $stride).build_mut(&mut $data)
        }
    };
    ($data:expr, $( $rest:tt )*) => {
        slice_attr_mut!(1, $data, $($rest)*)
    };
}
