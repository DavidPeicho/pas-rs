use strided_slice::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
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

#[test]
fn slice_count() {}

#[test]
fn mutable_indexing() {
    let mut vertices = data();

    let mut slice: SliceMut<[f32; 3]> = SliceMut::new(&mut vertices, 1, 0);

    assert_eq!(slice[0], [1.0, -1.0, 1.0]);
    assert_eq!(slice[1], [-1.0, 1.0, 0.0]);

    // Changing index 0 doesn't affect other index.
    slice[0] = [4.0, 3.0, 1.0];
    assert_eq!(slice[0], [4.0, 3.0, 1.0]);
    assert_eq!(slice[1], [-1.0, 1.0, 0.0]);

    // Changing index 1 doesn't affect other index.
    slice[1] = [11.0, 10.0, 9.0];
    assert_eq!(slice[0], [4.0, 3.0, 1.0]);
    assert_eq!(slice[1], [11.0, 10.0, 9.0]);
}

#[test]
fn mutable_iter() {
    let mut vertices = data();
    let slice: SliceMut<[f32; 3]> = SliceMut::new(&mut vertices, 1, 0);
    {
        let mut iter = slice.iter();
        assert_eq!(*iter.next().unwrap(), [1.0, -1.0, 1.0]);
        assert_eq!(*iter.next().unwrap(), [-1.0, 1.0, 0.0]);
        assert_eq!(iter.next(), None);
    }

    let slice: SliceMut<[f32; 2]> =
        SliceMut::new(&mut vertices, 1, std::mem::size_of::<[f32; 3]>());
    {
        let mut iter = slice.iter();
        assert_eq!(*iter.next().unwrap(), [0.25, 0.5]);
        assert_eq!(*iter.next().unwrap(), [-1.0, 0.25]);
        assert_eq!(iter.next(), None);
    }
}

#[test]
fn copy_from_slice() {
    let mut vertices = data();
    let slice: SliceMut<[f32; 3]> = SliceMut::new(&mut vertices, 1, 0);

    slice.copy_from_slice(&[[0.1_f32, 0.2, 0.3]]);
    assert_eq!(slice[0], [0.1_f32, 0.2, 0.3]);
    slice.copy_from_slice(&[[0.9_f32, 0.8, 0.7], [0.6, 0.5, 0.4]]);
    assert_eq!(slice[0], [0.9, 0.8, 0.7]);
    assert_eq!(slice[1], [0.6, 0.5, 0.4]);

    let slice: SliceMut<[f32; 2]> =
        SliceMut::new(&mut vertices, 1, std::mem::size_of::<[f32; 3]>());
    slice.copy_from_slice(&[[0.1_f32, 0.2]]);
    assert_eq!(slice[0], [0.1, 0.2]);
    slice.copy_from_slice(&[[0.1_f32, 0.2], [0.3, 0.4]]);
    assert_eq!(slice[0], [0.1, 0.2]);
    assert_eq!(slice[1], [0.3, 0.4]);
}

#[test]
fn slicer() {
    let mut vertices = data();

    let slice: SliceMut<[f32; 3]> = slice_mut!(vertices, [0].position);
    assert_eq!(slice.len(), 2);
    assert_eq!(slice[0], [1.0_f32, -1.0, 1.0]);
    assert_eq!(slice[1], [-1.0_f32, 1.0, 0.0]);

    let slice: SliceMut<[f32; 3]> = slice_mut!(vertices, [1].position);
    assert_eq!(slice.len(), 1);
    assert_eq!(slice[0], [-1.0_f32, 1.0, 0.0]);
}

// #[test]
// fn slicer_stride() {
//     let mut data = [0.0_f32, 1.0, 2.0, 3.0, 4.0, 5.0];
//     let slice = Slicer::new().stride(3).build_mut::<[f32; 3], _>(&mut data);
//     assert_eq!(slice.len(), 2);
//     assert_eq!(slice[0], [0.0_f32, 1.0, 2.0]);
//     assert_eq!(slice[1], [3.0_f32, 4.0, 5.0]);
// }
