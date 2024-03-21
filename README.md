# strided-slice-rs

Crate for slicing stuff! Especially strided data.

This crate allows you to:
* Get a slice with a custom stride
* Slice only a part of a struct

⚠️ This crate relies on casting between different types ⚠️
* This operation is **endian dependant**
* No default mechanism to **encode**/**decode** types in **big endian** is provided

## Examples

### Macros

#### With Type Inference

Using `slice_attr!` to slice in a `struct` and automatically infer the type:

```rust
use strided_slice::{slice, slice_attr};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
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
    println!("{:?}", positions); // [[1.0, 0.5, 1.0], [1.0, 1.0, 0.5]]

    // Start slice at second vertex, pointing at `uv`.
    let uvs = slice_attr!(vertices, [1].uv);
    println!("{:?}", uvs); // [[0.0, 1.0]]
}
```

#### Without Type Inference

It can be useful to slice at a `struct` attribute, but with a smaller type:

```rust,ignore
let x_positions: Slice<f32> = slice!(vertices, [0].position[0]);
println!("{:?}", x_positions); // [1.0, 1.0]

let y_positions: Slice<f32> = slice!(vertices, [0].position[1]);
println!("{:?}", y_positions); // [0.5, 1.0]

let z_positions: Slice<f32> = slice!(vertices, [0].position[2]);
println!("{:?}", z_positions); // [1.0, 0.5]
```

### Slice and SliceMut

When slicing an array whose type information is known only at runtime, you can use `Slice`/`SliceMut`:

```rust, ignore
let uv_byte_offset = std::mem::size_of::<Vertex>() + std::mem::size_of::<[f32; 3]>();

// Slice starting at the byte offset `32`, with a stride of 1 element.
let uvs: Slice<[f32; 3]> = Slice::new(&vertices, uv_byte_offset, 1);
println!("{:?}", uvs); // [[0.0, 1.0]]
```

## Safety

While this crate makes use of `unsafe` and `transmute`, it's (_mostly_) safe
to use and comes with runtime checks preventing you to run into undefined behaviors.

This crate relies requires your types to implement the [Pod trait](https://docs.rs/bytemuck/latest/bytemuck/trait.Pod.html) from the [bytemuck crate](https://docs.rs/bytemuck/latest/bytemuck/), improving safety with alignment rules, illegal bit patterns, etc...
