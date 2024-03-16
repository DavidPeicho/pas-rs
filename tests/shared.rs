use std::num::NonZeroUsize;
use strided_slice::*;

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

macro_rules! tests {
    ($slice: ident, $name: ident) => { paste::expr! {
        #[test]
        fn [<slice_count_$name>]() {
            let vertices = data();

            let slice: $slice<f32> = $slice::new(&[] as &[f32], 0);
            assert_eq!(slice.len(), 0);

            let slice: $slice<f32> = $slice::new(&vertices, 0);
            assert_eq!(slice.len(), 2);

            let slice: $slice<[f32; 2]> = $slice::new(&vertices, std::mem::size_of::<[f32; 3]>());
            assert_eq!(slice.len(), 2);

            let slice: $slice<[f32; 2]> =
                $slice::new(&vertices[1..], std::mem::size_of::<[f32; 3]>());
            assert_eq!(slice.len(), 1);

            let positions: [f32; 3] = [1.0, 2.0, 3.0];
            let slice: $slice<f32> = $slice::new(&positions, 0);
            assert_eq!(slice.len(), 3);

            let positions: [f32; 3] = [1.0, 2.0, 3.0];
            let slice: $slice<f32> = $slice::new(&positions, 0);
            assert_eq!(slice.len(), 3);
        }

        #[test]
        fn [<from_raw_part_invalid_offset_$name>]() {
            let data: Vec<u8> = vec![0, 200, 100];
            let error = $slice::<f32>::try_raw(&data, 2, NonZeroUsize::new(3).unwrap()).unwrap_err();
            assert_eq!(error, SliceError::OffsetOutOfBounds);
        }

        #[test]
        fn [<from_raw_part_invalid_stride_$name>]() {
            let data: Vec<u8> = vec![0, 200, 100, 255];
            let error = $slice::<f32>::try_raw(&data, 8, NonZeroUsize::new(100).unwrap()).unwrap_err();
            assert_eq!(error, SliceError::StrideOutOfBounds);
        }

        #[test]
        fn [<strided_$name>]() {
            let data: [f32; 12] = [
                1.0, 1.0, 1.0, // position
                0.0, 1.0, 0.0, // normal
                1.0, 1.0, -1.0, // position
                0.0, 1.0, 0.0, // normal
            ];
            let stride = NonZeroUsize::new(6).unwrap();

            let positions: $slice<[f32; 3]> = $slice::strided(&data, 0, stride);
            assert_eq!(positions[0], [1.0_f32, 1.0, 1.0]);
            assert_eq!(positions[1], [1.0_f32, 1.0, -1.0]);

            let normals: $slice<[f32; 3]> = $slice::strided(&data[3..], 0, stride);
            assert_eq!(normals[0], [0.0_f32, 1.0, 0.0]);
            assert_eq!(normals[1], [0.0_f32, 1.0, 0.0]);
        }

        #[test]
        fn [<indexing_$name>]() {
            let vertices = data();
            let slice: $slice<[f32; 3]> = $slice::new(&vertices, 0);
            assert_eq!(slice[0], [1.0, -1.0, 1.0]);
            assert_eq!(slice[1], [-1.0, 1.0, 0.0]);

            let slice: $slice<[f32; 2]> =
                $slice::new(&vertices[1..], std::mem::size_of::<[f32; 3]>());
            assert_eq!(slice[0], [-1.0, 0.25]);
            assert_eq!(slice.get(1), None);
        }

        #[test]
        fn [<iter_$name>]() {
            let vertices = data();
            {
                let slice: $slice<[f32; 3]> = $slice::new(&vertices, 0);
                let mut iter = slice.iter();
                assert_eq!(*iter.next().unwrap(), [1.0, -1.0, 1.0]);
                assert_eq!(*iter.next().unwrap(), [-1.0, 1.0, 0.0]);
                assert_eq!(iter.next(), None);
            }
            {
                let slice: $slice<[f32; 2]> =
                    $slice::new(&vertices, std::mem::size_of::<[f32; 3]>());
                let mut iter = slice.iter();
                assert_eq!(*iter.next().unwrap(), [0.25, 0.5]);
                assert_eq!(*iter.next().unwrap(), [-1.0, 0.25]);
                assert_eq!(iter.next(), None);
            }
        }
    }};
}

tests!(Slice, immutable);
tests!(SliceMut, mutable);
