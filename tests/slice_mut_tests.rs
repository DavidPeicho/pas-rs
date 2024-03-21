use strided_slice::SliceMut;

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

#[test]
fn mutable_indexing() {
    let mut vertices = data();

    let mut slice: SliceMut<[u32; 3]> = SliceMut::new(&mut vertices, 0, 1);

    assert_eq!(slice[0], [0, 1, 2]);
    assert_eq!(slice[1], [5, 6, 7]);

    // Changing index 0 doesn't affect other index.
    slice[0] = [20, 21, 22];
    assert_eq!(slice[0], [20, 21, 22]);
    assert_eq!(slice[1], [5, 6, 7]);

    // Changing index 1 doesn't affect other index.
    slice[1] = [100, 101, 102];
    assert_eq!(slice[0], [20, 21, 22]);
    assert_eq!(slice[1], [100, 101, 102]);
}

#[test]
fn copy_from_slice() {
    let mut vertices = data();
    let slice: SliceMut<[u32; 3]> = SliceMut::new(&mut vertices, 0, 1);

    slice.copy_from_slice(&[[20, 21, 22]]);
    assert_eq!(slice[0], [20, 21, 22]);
    slice.copy_from_slice(&[[30, 31, 32], [33, 34, 35]]);
    assert_eq!(slice[0], [30, 31, 32]);
    assert_eq!(slice[1], [33, 34, 35]);

    let slice: SliceMut<[u32; 2]> =
        SliceMut::new(&mut vertices, std::mem::size_of::<[f32; 3]>(), 1);
    slice.copy_from_slice(&[[101, 102]]);
    assert_eq!(slice[0], [101, 102]);
    slice.copy_from_slice(&[[103, 104], [105, 106]]);
    assert_eq!(slice[0], [103, 104]);
    assert_eq!(slice[1], [105, 106]);

    // Positions shouldn't be affected
    let slice: SliceMut<[u32; 3]> = SliceMut::new(&mut vertices, 0, 1);
    assert_eq!(slice[0], [30, 31, 32]);
    assert_eq!(slice[1], [33, 34, 35]);
}
