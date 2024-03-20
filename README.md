# strided-slice-rs

Crate for slicing stuff! Especially strided data.

This crate allows you to:
* Get a slice with a custom stride
* Slice only a part of a struct

## Example

### Using Macro

Using `slice_attr!` to slice in a `struct` and automatically infer the type:

```rust
use strided_slice::slice;

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

    // Start slice at first vertex, pointing at `position`.
    let positions = slice_attr!(vertices, [0].position);
    // Start slice at second vertex, pointing at `uv`.
    let uvs = slice_attr!(vertices, [1].uv);

    println!("Positions = {:?}", positions); // [[1.0, 0.5, 1.0], [1.0, 1.0, 0.5]]
    println!("Texture Coordinates = {:?}", uvs); // [0.0, 1.0]
}
```

Sometimes, it's useful to slice at a `struct` attribute, but with a smaller view type. In this case,
you need to tell the compiler what the slice should be:

```rust
let x_positions: Slice<f32> = slice!(vertices, [0].position);
println!("x-axis positions = {:?}", x_positions); // [1.0, 1.0]

let y_positions: Slice<f32> = slice!(vertices, [0].position[1]);
println!("y-axis positions = {:?}", y_positions); // [0.5, 1.0]

let z_positions: Slice<f32> = slice!(vertices, [0].position[2]);
println!("z-axis positions = {:?}", z_positions); // [1.0, 0.5]
```

### Using Stride & Offset

For runtime slicing, you can directly use `Slice` and `SliceMut`:

```rust
// todo
```

## Safety

While this crate makes use of `unsafe` and `transmute`, it's (_mostly_) safe
to use and comes with runtime checks preventing you to run into undefined behaviors.
