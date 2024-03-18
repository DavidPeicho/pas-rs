# strided-slice-rs

Crate for slicing stuff! Especially strided data.

This crate allows you to:
* Get a slice with a custom stride
* Slice only a part of a struct

## Example

Slicing starting at a reference:

```rust
use strided_slice::Slicer;

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

fn main() {
    let vertices = [
        Vertex {position: [1.0, 0.5, 1.0], uv: [1.0, 1.0]},
        Vertex {position: [1.0, 1.0, 0.5], uv: [0.0, 1.0]},
    ];

    let uvs: Slice<[f32; 2]> = Slicer::new()
        .offset_of(&vertices[0].uv) // Start slice at second vertex
        .build(&vertices);

    println!("Texture Coordinate = {:?}", uvs[0]); // [0.0, 1.0]
}
```

## Safety

While this crate makes use of `unsafe` and `transmute`, it's (_mostly_) safe
to use and comes with runtime checks preventing you to run into undefined behaviors.
