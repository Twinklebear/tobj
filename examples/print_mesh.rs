fn main() {
    let obj_file = std::env::args()
        .nth(1)
        .expect("A .obj file to print is required");

    let (models, materials) =
        tobj::load_obj(obj_file, &tobj::LoadOptions::default()).expect("Failed to OBJ load file");

    // Note: If you don't mind missing the materials, you can generate a default.
    let materials = materials.expect("Failed to load MTL file");

    println!("Number of models          = {}", models.len());
    println!("Number of materials       = {}", materials.len());

    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;
        println!();
        println!("model[{}].name             = \'{}\'", i, m.name);
        println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);

        println!(
            "model[{}].face_count       = {}",
            i,
            mesh.face_arities.len()
        );

        let mut next_face = 0;
        for face in 0..mesh.face_arities.len() {
            let end = next_face + mesh.face_arities[face] as usize;

            let face_indices = &mesh.indices[next_face..end];
            println!(" face[{}].indices          = {:?}", face, face_indices);

            if !mesh.texcoord_indices.is_empty() {
                let texcoord_face_indices = &mesh.texcoord_indices[next_face..end];
                println!(
                    " face[{}].texcoord_indices = {:?}",
                    face, texcoord_face_indices
                );
            }
            if !mesh.normal_indices.is_empty() {
                let normal_face_indices = &mesh.normal_indices[next_face..end];
                println!(
                    " face[{}].normal_indices   = {:?}",
                    face, normal_face_indices
                );
            }

            next_face = end;
        }

        // Normals and texture coordinates are also loaded, but not printed in
        // this example.
        println!(
            "model[{}].positions        = {}",
            i,
            mesh.positions.len() / 3
        );
        assert!(mesh.positions.len() % 3 == 0);

        for vtx in 0..mesh.positions.len() / 3 {
            println!(
                "              position[{}] = ({}, {}, {})",
                vtx,
                mesh.positions[3 * vtx],
                mesh.positions[3 * vtx + 1],
                mesh.positions[3 * vtx + 2]
            );
        }
    }

    for (i, m) in materials.iter().enumerate() {
        println!("material[{}].name = \'{}\'", i, m.name);
        if let Some(ambient) = m.ambient {
            println!(
                "    material.Ka = ({}, {}, {})",
                ambient[0], ambient[1], ambient[2]
            );
        }
        if let Some(diffuse) = m.diffuse {
            println!(
                "    material.Kd = ({}, {}, {})",
                diffuse[0], diffuse[1], diffuse[2]
            );
        }
        if let Some(specular) = m.specular {
            println!(
                "    material.Ks = ({}, {}, {})",
                specular[0], specular[1], specular[2]
            );
        }
        if let Some(shininess) = m.shininess {
            println!("    material.Ns = {}", shininess);
        }
        if let Some(dissolve) = m.dissolve {
            println!("    material.d = {}", dissolve);
        }
        if let Some(ambient_texture) = &m.ambient_texture {
            println!("    material.map_Ka = {}", ambient_texture);
        }
        if let Some(diffuse_texture) = &m.diffuse_texture {
            println!("    material.map_Kd = {}", diffuse_texture);
        }
        if let Some(specular_texture) = &m.specular_texture {
            println!("    material.map_Ks = {}", specular_texture);
        }
        if let Some(shininess_texture) = &m.shininess_texture {
            println!("    material.map_Ns = {}", shininess_texture);
        }
        if let Some(normal_texture) = &m.normal_texture {
            println!("    material.map_Bump = {}", normal_texture);
        }
        if let Some(dissolve_texture) = &m.dissolve_texture {
            println!("    material.map_d = {}", dissolve_texture);
        }

        for (k, v) in &m.unknown_param {
            println!("    material.{} = {}", k, v);
        }
    }
}
