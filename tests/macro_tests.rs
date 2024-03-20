use strided_slice::{slice, slice_attr, Slice};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

fn data() -> Vec<Vertex> {
    vec![
        Vertex {
            position: [0.0, 1.0, 2.0],
            uv: [3.0, 4.0],
        },
        Vertex {
            position: [5.0, 6.0, 7.0],
            uv: [8.0, 9.0],
        },
        Vertex {
            position: [10.0, 11.0, 12.0],
            uv: [13.0, 14.0],
        },
    ]
}

#[test]
fn slice_first_attribute() {
    let vertices: Vec<Vertex> = data();

    let slice = slice_attr!(vertices, [2].position);
    assert_eq!(slice.len(), 1);
    let slice = slice_attr!(vertices, [1].position);
    assert_eq!(slice.len(), 2);

    let slice = slice_attr!(vertices, [0].position);
    assert_eq!(slice.len(), 3);
    assert!(slice
        .iter()
        .eq([[0.0, 1.0, 2.0], [5.0, 6.0, 7.0], [10.0, 11.0, 12.0]].iter()));
}

#[test]
fn slice_attribute() {
    let vertices: Vec<Vertex> = data();

    let slice = slice_attr!(vertices, [2].uv);
    assert_eq!(slice.len(), 1);
    let slice = slice_attr!(vertices, [1].uv);
    assert_eq!(slice.len(), 2);
    let slice = slice_attr!(vertices, [0].uv);
    assert_eq!(slice.len(), 3);
    assert!(slice
        .iter()
        .eq([[3.0, 4.0], [8.0, 9.0], [13.0, 14.0]].iter()));
}

#[test]
fn slice() {
    let vertices: Vec<Vertex> = data();

    let slice: Slice<f32> = slice!(vertices, [1].position);
    assert_eq!(slice.len(), 2);
    assert!(slice.iter().eq([5.0, 10.0].iter()));

    let slice: Slice<f32> = slice!(vertices, [1].position[1]);
    assert_eq!(slice.len(), 2);
    assert!(slice.iter().eq([6.0, 11.0].iter()));

    let slice: Slice<f32> = slice!(vertices, [0].uv);
    assert_eq!(slice.len(), 3);
    assert!(slice.iter().eq([3.0, 8.0, 13.0].iter()));
}

#[test]
fn slice_with_stride() {
    let values = vec![0.0_f32, 1.0, 2.0, 3.0, 4.0, 5.0];

    let expected_stride1 = [0.0_f32, 1.0, 2.0, 3.0, 4.0, 5.0];
    let slice: Slice<f32> = slice!(1, values, [0]);
    assert!(slice.iter().eq(expected_stride1.iter()));
    let slice = slice_attr!(1, values, [0]);
    assert!(slice.iter().eq(expected_stride1.iter()));

    let expected_stride2 = [1.0_f32, 3.0, 5.0];
    let slice: Slice<f32> = slice!(2, values, [1]);
    assert!(slice.iter().eq(expected_stride2.iter()));
    let slice = slice_attr!(2, values, [1]);
    assert!(slice.iter().eq(expected_stride2.iter()));
}

// TODO: Test mut macro.
