use std::borrow::{Borrow, BorrowMut};

use pas::{Slice, SliceMut};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
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

// Test for [`Slice`] and [`SliceMut`] as well as [`SliceIterator`] and [`SliceIteratorMut`].
macro_rules! tests {
    ($slice: ident, $name: ident, $borrow: ident) => { paste::expr! {
        #[test]
        fn [<slice_len_$name>]() {
            #[allow(unused_mut)]
            let mut vertices = data();

            #[allow(unused_mut)]
            let mut empty: Vec<u32> = Vec::new();
            let slice: $slice<u32> = $slice::new(empty.$borrow(), 0, 1);
            assert_eq!(slice.len(), 0);

            let slice: $slice<f32> = $slice::new(vertices.$borrow(), 0, 1);
            assert_eq!(slice.len(), 3);

            let slice: $slice<[f32; 2]> = $slice::new(vertices.$borrow(), std::mem::size_of::<[f32; 3]>(), 1);
            assert_eq!(slice.len(), 3);

            let slice: $slice<[f32; 2]> = $slice::new(vertices.$borrow(), std::mem::size_of::<[f32; 3]>(), 2);
            assert_eq!(slice.len(), 2);
        }

        #[test]
        fn [<strided_$name>]() {
            #[allow(unused_mut)]
            let mut data: [u32; 12] = [
                1, 2, 3,
                4, 5, 6,
                7, 8, 9,
                10, 11, 12,
            ];
            let stride: usize = 6;

            let positions: $slice<[u32; 3]> = $slice::new(data.$borrow(), 0, stride);
            assert!(positions.iter().eq([[1, 2, 3], [7, 8, 9]].iter()));

            let normals: $slice<[u32; 3]> = $slice::new(data.$borrow(), 3 * std::mem::size_of::<u32>(), stride);
            assert!(normals.iter().eq([[4, 5, 6], [10, 11, 12]].iter()));
        }

        #[test]
        fn [<indexing_$name>]() {
            #[allow(unused_mut)]
            let mut vertices = data();
            let slice: $slice<[u32; 3]> = $slice::new(vertices.$borrow(), 0, 1);
            assert_eq!(slice[0], [0, 1, 2]);
            assert_eq!(slice[1], [5, 6, 7]);

            // Point to third uv
            let slice: $slice<[u32; 2]> = $slice::new(vertices.$borrow(),
                2 * std::mem::size_of::<Vertex>() + std::mem::size_of::<[f32; 3]>(), 1);
            assert_eq!(slice[0], [13, 14]);
            assert_eq!(slice.get(1), None);
        }

        #[test]
        fn [<iter_$name>]() {
            #[allow(unused_mut)]
            let mut vertices = data();

            let slice: $slice<[u32; 3]> = $slice::new(vertices.$borrow(), 0, 1);
            let mut iter = slice.iter();
            assert_eq!(*iter.next().unwrap(), [0, 1, 2]);
            assert_eq!(*iter.next().unwrap(), [5, 6, 7]);
            assert_eq!(*iter.next().unwrap(), [10, 11, 12]);
            assert_eq!(iter.next(), None);

            let slice: $slice<[u32; 2]> = $slice::new(vertices.$borrow(), std::mem::size_of::<[f32; 3]>(), 1);
            let mut iter = slice.iter();
            assert_eq!(*iter.next().unwrap(), [3, 4]);
            assert_eq!(*iter.next().unwrap(), [8, 9]);
            assert_eq!(*iter.next().unwrap(), [13, 14]);
            assert_eq!(iter.next(), None);
        }

        #[test]
        #[should_panic]
        fn [<attr_larger_than_stride_$name>]() {
            #[allow(unused_mut)]
            let mut positions = [0, 1, 2, 3];
            let _: $slice<Vertex> = $slice::new(positions.$borrow(), 0, 1);
        }

        #[test]
        #[should_panic]
        fn [<offset_larger_than_slice_$name>]() {
            #[allow(unused_mut)]
            let mut vertices = data();
            let bytes = std::mem::size_of_val(&*vertices);
            let _: $slice<[u32; 3]> = $slice::new(vertices.$borrow(), bytes, 1);
        }

        #[test]
        #[should_panic]
        fn [<unaligned_attr_$name>]() {
            #[allow(unused_mut)]
            let mut positions = [0, 1, 2, 3];
            let _: $slice<u32> = $slice::new(positions.$borrow(), 1, 1);
        }
    }};
}

tests!(Slice, immutable, borrow);
tests!(SliceMut, mutable, borrow_mut);
