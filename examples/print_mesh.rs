extern crate tobj;

use std::env;

fn main() {
    let obj_file = env::args().skip(1).next().expect("A .obj file to print is required");
    let (models, materials) = tobj::load_obj(&obj_file, false).expect("Failed to load file");

    println!("# of models: {}", models.len());
    println!("# of materials: {}", materials.len());
    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;
        println!("model[{}].name = \'{}\'", i, m.name);
        println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);

        println!("Size of model[{}].num_face_indices: {}", i, mesh.num_face_indices.len());
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
            println!("    v[{}] = ({}, {}, {})", v, mesh.positions[3 * v],
            mesh.positions[3 * v + 1], mesh.positions[3 * v + 2]);
        }
    }

    for (i, m) in materials.iter().enumerate() {
        println!("material[{}].name = \'{}\'", i, m.name);
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

}
