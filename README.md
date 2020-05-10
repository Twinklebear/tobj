tobj - Tiny OBJ Loader
===
A tiny OBJ loader, inspired by Syoyo's excellent [tinyobjloader](https://github.com/syoyo/tinyobjloader).
Aims to be a simple and lightweight option for loading OBJ files, just returns two vecs
containing loaded models and materials. All models are made of triangles, any quad or polygon faces in an
OBJ file will be converted to triangles. Note that only polygons that are trivially
convertible to triangle fans are supported, arbitrary polygons may not behave as expected.
The best solution would be to re-export your mesh using only triangles in your modeling software.

It is assumed that all meshes will at least have positions, but normals and texture coordinates
are optional. If no normals or texture coordinates were found then the corresponding vecs for
the mesh will be empty. Values are stored packed as floats in vecs, eg. the positions member of
a loaded mesh will contain `[x, y, z, x, y, z, ...]` which you can then use however you like.
Indices are also loaded and may re-use vertices already existing in the mesh, this data is
stored in the `indices` member.

Standard MTL attributes are supported as well and any unrecognized parameters will be stored in a
HashMap containing the key-value pairs of the unrecognized parameter and its value.

Documentation
---
Rust doc can be found [here](http://www.willusher.io/tobj/tobj/).

Installation
---
Add the [crate](https://crates.io/crates/tobj) as a dependency in your Cargo.toml and you're all set!

[![Crate](https://img.shields.io/crates/v/tobj.svg)](https://crates.io/crates/tobj)
[![Build Status](https://travis-ci.org/Twinklebear/tobj.svg?branch=master)](https://travis-ci.org/Twinklebear/tobj)

Example
---
The [print mesh example](examples/print_mesh.rs) (also below) loads an OBJ file from
the command line and prints out some information about its faces, vertices, and materials.

```rust
extern crate tobj;

use std::env;

fn main() {
    let obj_file = env::args()
        .skip(1)
        .next()
        .expect("A .obj file to print is required");
    let (models, materials) = tobj::load_obj(&obj_file, false).expect("Failed to load file");

    println!("# of models: {}", models.len());
    println!("# of materials: {}", materials.len());
    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;
        println!("model[{}].name = \'{}\'", i, m.name);
        println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);

        println!(
            "Size of model[{}].num_face_indices: {}",
            i,
            mesh.num_face_indices.len()
        );
        let mut next_face = 0;
        for f in 0..mesh.num_face_indices.len() {
            let end = next_face + mesh.num_face_indices[f] as usize;
            let face_indices: Vec<_> = mesh.indices[next_face..end].iter().collect();
            println!("    face[{}] = {:?}", f, face_indices);
            next_face = end;
        }

        // Normals and texture coordinates are also loaded, but not printed in this example
        println!("model[{}].vertices: {}", i, mesh.positions.len() / 3);
        assert!(mesh.positions.len() % 3 == 0);
        for v in 0..mesh.positions.len() / 3 {
            println!(
                "    v[{}] = ({}, {}, {})",
                v,
                mesh.positions[3 * v],
                mesh.positions[3 * v + 1],
                mesh.positions[3 * v + 2]
            );
        }
    }

    for (i, m) in materials.iter().enumerate() {
        println!("material[{}].name = \'{}\'", i, m.name);
        println!(
            "    material.Ka = ({}, {}, {})",
            m.ambient[0], m.ambient[1], m.ambient[2]
        );
        println!(
            "    material.Kd = ({}, {}, {})",
            m.diffuse[0], m.diffuse[1], m.diffuse[2]
        );
        println!(
            "    material.Ks = ({}, {}, {})",
            m.specular[0], m.specular[1], m.specular[2]
        );
        println!("    material.Ns = {}", m.shininess);
        println!("    material.d = {}", m.dissolve);
        println!("    material.map_Ka = {}", m.ambient_texture);
        println!("    material.map_Kd = {}", m.diffuse_texture);
        println!("    material.map_Ks = {}", m.specular_texture);
        println!("    material.map_Ns = {}", m.shininess_texture);
        println!("    material.map_Bump = {}", m.normal_texture);
        println!("    material.map_d = {}", m.dissolve_texture);
        for (k, v) in &m.unknown_param {
            println!("    material.{} = {}", k, v);
        }
    }
}
```

Rendering Examples
---
For an example of integration with [glium](https://github.com/tomaka/glium) to make a simple OBJ viewer, check out
[tobj viewer](https://github.com/Twinklebear/tobj_viewer). A sample image from the viewer is shown below,
the [Rungholt](http://graphics.cs.williams.edu/data/meshes.xml) model can be found on Morgan McGuire's meshes
page and was originally built by kescha.

The Rungholt model is reasonably large (6.7M triangles, 12.3M vertices) and is loaded in ~7.47s
using a peak of ~1.1GB of memory on a Windows 10 machine with an i7-4790k and 16GB of 1600Mhz DDR3 RAM with
tobj 0.1.1 on rustc 1.6.0.

![Rungholt](http://i.imgur.com/wImyNG4.png)

For an example of integration within a ray tracer, check out tray\_rust's
[mesh module](https://github.com/Twinklebear/tray_rust/blob/master/src/geometry/mesh.rs).
The Stanford Buddha and Dragon from the [Stanford 3D Scanning Repository](http://graphics.stanford.edu/data/3Dscanrep/) both load quite quickly.
The Rust logo model was made by
[Nylithius on BlenderArtists](http://blenderartists.org/forum/showthread.php?362836-Rust-language-3D-logo). 
The materials used are from the [MERL BRDF Database](http://www.merl.com/brdf/).

![Rust Logo with friends](http://i.imgur.com/E1ylrZW.png)

