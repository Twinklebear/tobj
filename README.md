tobj - Tiny OBJ Loader
===
Tiny OBJ loader, inspired by Syoyo's excellent [tinyobjloader](https://github.com/syoyo/tinyobjloader).
Aims to be a simple and lightweight option for loading OBJ files, simply returns two vecs
containing loaded models and materials. All models are made of triangles, any quad faces in an
OBJ file will be converted to two triangles.

Documentation
---
Rust doc can be found [here](http://www.willusher.io/tobj/tobj/).

Installation
---
Add the [crate](https://crates.io/crates/tobj) as a dependency in your Cargo.toml and you're all set!

Example
---
In this simple example we load the classic Cornell Box model that only defines positions and
print out its attributes.

```rust
let cornell_box = load_obj(&Path::new("cornell_box.obj"));
assert!(cornell_box.is_ok());
let (models, materials) = cornell_box.unwrap();

println!("# of models: {}", models.len());
println!("# of materials: {}", materials.len());
for (i, m) in models.iter().enumerate() {
	let mesh = &m.mesh;
	println!("model[{}].name = {}", i, m.name);
	println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);

	println!("Size of model[{}].indices: {}", i, mesh.indices.len());
	for f in 0..(mesh.indices.len() / 3) {
		println!("    idx[{}] = {}, {}, {}.", f, mesh.indices[3 * f],
			mesh.indices[3 * f + 1], mesh.indices[3 * f + 2]);
	}

	// Normals and texture coordinates are also loaded, but not printed
	// in this example
	println!("model[{}].vertices: {}", i, mesh.positions.len());
	assert!(mesh.positions.len() % 3 == 0);
	for v in 0..(mesh.positions.len() / 3) {
		println!("    v[{}] = ({}, {}, {})", v, mesh.positions[3 * v],
			mesh.positions[3 * v + 1], mesh.positions[3 * v + 2]);
	}
	print_material_info(materials);
}
for (i, m) in materials.iter().enumerate() {
	println!("material[{}].name = {}", i, m.name);
	println!("    material.Ka = ({}, {}, {})", m.ambient[0], m.ambient[1],
		m.ambient[2]);
	println!("    material.Kd = ({}, {}, {})", m.diffuse[0], m.diffuse[1],
		m.diffuse[2]);
	println!("    material.Ks = ({}, {}, {})", m.specular[0], m.specular[1],
		m.specular[2]);
	println!("    material.Ns = {}", m.shininess);
	println!("    material.d = {}", m.dissolve);
	println!("    material.map_Ka = {}", m.ambient_texture);
	println!("    material.map_Kd = {}", m.diffuse_texture);
	println!("    material.map_Ks = {}", m.specular_texture);
	println!("    material.map_Ns = {}", m.normal_texture);
	println!("    material.map_d = {}", m.dissolve_texture);
	for (k, v) in &m.unknown_param {
		println!("    material.{} = {}", k, v);
	}
}

