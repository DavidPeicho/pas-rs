/// Make the slice point to the referenced element
///
/// The slice generic type will be resolved to the type of
/// the attribute passed as the second macro argument.
///
/// ## Example
///
/// ```rust
/// use strided_slice::slice_attr;
///
/// #[repr(C)]
/// #[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
/// struct Vertex {
///     position: [f32; 3],
///     uv: [f32; 2],
/// }
///
/// let vertices = [
///     Vertex {position: [1.0, 0.5, 1.0], uv: [1.0, 1.0]},
///     Vertex {position: [1.0, 1.0, 0.5], uv: [0.0, 1.0]},
/// ];
/// let positions = slice_attr!(vertices, [0].position); // 2 positions
/// let uvs = slice_attr!(vertices, [1].uv); // 1 uv
/// ```
///
/// The stride, in element count, can be passed as a first argument:
///
/// ```rust
/// // Stride of 2 vertex
/// let positions = slice_attr!(2, vertices, [0].position);
/// ````
#[macro_export]
macro_rules! slice_attr {
    ($stride:expr, $data:expr, $( $rest:tt )*) => {
        {
            use strided_slice::SliceBuilder;

            let r = &($data$($rest)*);
            SliceBuilder::new(r, $stride).build(&$data)
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
/// use strided_slice::{slice, Slice};
///
/// #[repr(C)]
/// #[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
/// struct Vertex {
///     position: [f32; 3],
///     uv: [f32; 2],
/// }
///
/// let vertices = [
///     Vertex {position: [1.0, 0.5, 1.0], uv: [1.0, 1.0]},
///     Vertex {position: [1.0, 1.0, 0.5], uv: [0.0, 1.0]},
/// ];
/// // Only slice the x-axis positions
/// let x_positions: Slice<f32> = slice!(vertices, [0].position[0]);
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
            use strided_slice::SliceBuilder;
            let r = &($data$($rest)*);
            SliceBuilder::new(r, $stride).build_mut(&mut $data)
        }
    };
    ($data:expr, $( $rest:tt )*) => {
        slice_attr_mut!(1, $data, $($rest)*)
    };
}
