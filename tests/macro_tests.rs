use strided_slice::{slice, slice_attr, slice_attr_mut, slice_mut, Slice, SliceMut};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [u32; 3],
    pub uv: [u32; 2],
}

pub fn data() -> Vec<Vertex> {
    vec![
        Vertex {
            position: [0, 1, 2],
            uv: [3, 4],
        },
        Vertex {
            position: [5, 6, 7],
            uv: [8, 9],
        },
        Vertex {
            position: [10, 11, 12],
            uv: [13, 14],
        },
    ]
}

// Tests for [`slice_attr`] and [`slice_attr_mut`].
macro_rules! slice_attr_tests {
    ($slice_attr:ident) => {
        paste::paste! {
            #[test]
            fn [<$slice_attr>]() {
                #[allow(unused_mut)]
                let mut vertices = data();

                let slice = $slice_attr!(vertices, [0].position);
                assert!(slice.iter().eq([[0, 1, 2], [5, 6, 7], [10, 11, 12]].iter()));
                let slice = $slice_attr!(vertices, [1].position);
                assert!(slice.iter().eq([[5, 6, 7], [10, 11, 12]].iter()));
                let slice = $slice_attr!(vertices, [2].position);
                assert!(slice.iter().eq([[10, 11, 12]].iter()));

                let slice = slice_attr!(vertices, [0].uv);
                assert!(slice.iter().eq([[3, 4], [8, 9], [13, 14]].iter()));
                let slice = slice_attr!(vertices, [1].uv);
                assert!(slice.iter().eq([[8, 9], [13, 14]].iter()));
                let slice = slice_attr!(vertices, [2].uv);
                assert!(slice.iter().eq([[13, 14]].iter()));
            }

            #[test]
            fn [<strided_$slice_attr>]() {
                #[allow(unused_mut)]
                let mut values = vec![0, 1, 2, 3, 4, 5];

                let slice = $slice_attr!(1, values, [0]);
                assert!(slice.iter().eq([0, 1, 2, 3, 4, 5].iter()));

                let slice = $slice_attr!(2, values, [1]);
                assert!(slice.iter().eq([1, 3, 5].iter()));
            }
        }
    };
}

// Tests for [`slice`] and [`slice_mut`].
macro_rules! slice_tests {
    ($slice:ident, $type:ty) => {
        paste::paste! {
            #[test]
            fn [<$slice>]() {
                #[allow(unused_mut)]
                let mut vertices = data();

                let slice: $type<u32> = $slice!(vertices, [1].position);
                assert_eq!(slice.len(), 2);
                assert!(slice.iter().eq([5, 10].iter()));

                let slice: $type<u32> = $slice!(vertices, [1].position[1]);
                assert_eq!(slice.len(), 2);
                assert!(slice.iter().eq([6, 11].iter()));

                let slice: $type<u32> = $slice!(vertices, [0].uv);
                assert_eq!(slice.len(), 3);
                assert!(slice.iter().eq([3, 8, 13].iter()));
            }

            #[test]
            fn [<strided_$slice>]() {
                #[allow(unused_mut)]
                let mut values = vec![0, 1, 2, 3, 4, 5];

                let slice: $type<u32> = $slice!(1, values, [0]);
                assert!(slice.iter().eq([0, 1, 2, 3, 4, 5].iter()));

                let slice: $type<u32> = $slice!(2, values, [1]);
                assert!(slice.iter().eq([1, 3, 5].iter()));
            }
        }
    };
}

slice_attr_tests!(slice_attr);
slice_attr_tests!(slice_attr_mut);
slice_tests!(slice, Slice);
slice_tests!(slice_mut, SliceMut);
