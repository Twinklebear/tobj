use float_eq::assert_float_eq;
use std::{
    env,
    fs::File,
    io::{BufReader, Cursor},
};

use crate as tobj;

const CORNELL_BOX_OBJ: &str = include_str!("../obj/cornell_box.obj");
const CORNELL_BOX_MTL1: &str = include_str!("../obj/cornell_box.mtl");
const CORNELL_BOX_MTL2: &str = include_str!("../obj/cornell_box2.mtl");

// Set the tolerance for float comparison
use crate::Float;
const TOL: Float = 0.0000001;

#[test]
fn simple_triangle() {
    let m = tobj::load_obj(
        "obj/triangle.obj",
        &tobj::LoadOptions {
            single_index: true,
            ..Default::default()
        },
    );
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
    let mats = mats.unwrap();
    // We expect a single model with no materials
    assert_eq!(models.len(), 1);
    assert!(mats.is_empty());
    // Confirm our triangle is loaded correctly
    assert_eq!(models[0].name, "Triangle");
    let mesh = &models[0].mesh;
    assert!(mesh.normals.is_empty());
    assert!(mesh.texcoords.is_empty());
    assert_eq!(mesh.material_id, None);

    // Verify each position is loaded properly
    let expect_pos = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    assert_float_eq!(mesh.positions, expect_pos, r2nd_all <= TOL);
    // Verify the indices are loaded properly
    let expect_idx = vec![0, 1, 2];
    assert_eq!(mesh.indices, expect_idx);

    // Verify that there are no vertex colors
    assert!(mesh.vertex_color.is_empty());
}

#[test]
fn simple_triangle_colored() {
    let m = tobj::load_obj(
        "obj/triangle_colored.obj",
        &tobj::LoadOptions {
            single_index: true,
            ..Default::default()
        },
    );
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
    let mats = mats.unwrap();
    // We expect a single model with no materials
    assert_eq!(models.len(), 1);
    assert!(mats.is_empty());
    // Confirm our triangle is loaded correctly
    assert_eq!(models[0].name, "Triangle");
    let mesh = &models[0].mesh;
    assert!(mesh.normals.is_empty());
    assert!(mesh.texcoords.is_empty());
    assert_eq!(mesh.material_id, None);

    // Verify each position is loaded properly
    let expect_pos = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    assert_float_eq!(mesh.positions, expect_pos, r2nd_all <= TOL);
    // Verify the indices are loaded properly
    let expect_idx = vec![0, 1, 2];
    assert_eq!(mesh.indices, expect_idx);

    // Verify vertex colors are loaded
    let expect_vertex_color = vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    assert_float_eq!(mesh.vertex_color, expect_vertex_color, r2nd_all <= TOL);
}

#[test]
#[cfg(feature = "merging")]
fn simple_quad_colored_merge() {
    let m = tobj::load_obj(
        "obj/quad_colored_merge.obj",
        &tobj::LoadOptions {
            merge_identical_points: true,
            ..Default::default()
        },
    );
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
    let mats = mats.unwrap();
    // We expect a single model with no materials
    assert_eq!(models.len(), 1);
    assert!(mats.is_empty());
    // Confirm our quad is loaded correctly
    assert_eq!(models[0].name, "Quad");
    let mesh = &models[0].mesh;
    assert_eq!(mesh.normals.len(), 3);
    assert!(mesh.texcoords.is_empty());
    assert_eq!(mesh.material_id, None);

    // Verify each position is loaded properly and merged, with the exception of
    // the 2nd and 4th vertices which have different vertex colors.
    #[rustfmt::skip]
    let expect_pos = vec![
        0.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        1.0, 1.0, 0.0,
    ];
    assert_float_eq!(mesh.positions, expect_pos, r2nd_all <= TOL);
    // Verify the indices are loaded properly
    let expect_idx = vec![0, 1, 2, 1, 2, 3];
    assert_eq!(mesh.indices, expect_idx);

    // Verify vertex colors are loaded and correctly indexed.
    #[rustfmt::skip]
    let expect_vertex_color = vec![
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 0.0,
        1.0, 1.0, 1.0,
    ];
    assert_float_eq!(mesh.vertex_color, expect_vertex_color, r2nd_all <= TOL);
    let expect_vertex_color_index = vec![0, 1, 2, 3, 2, 4];
    assert_eq!(mesh.vertex_color_indices, expect_vertex_color_index);
}

#[test]
fn empty_name_triangle() {
    let m = tobj::load_obj(
        "obj/empty_name_triangle.obj",
        &tobj::LoadOptions {
            single_index: true,
            ..Default::default()
        },
    );
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
    let mats = mats.unwrap();
    // We expect a single model with no materials
    assert_eq!(models.len(), 1);
    assert!(mats.is_empty());
    // Confirm our triangle is loaded correctly
    assert_eq!(models[0].name, "unnamed_object");
    let mesh = &models[0].mesh;
    assert!(mesh.normals.is_empty());
    assert!(mesh.texcoords.is_empty());
    assert_eq!(mesh.material_id, None);

    // Verify each position is loaded properly
    let expect_pos = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    assert_float_eq!(mesh.positions, expect_pos, r2nd_all <= TOL);
    // Verify the indices are loaded properly
    let expect_idx = vec![0, 1, 2];
    assert_eq!(mesh.indices, expect_idx);
}

#[test]
fn test_lines() {
    let m = tobj::load_obj(
        "obj/lines.obj",
        &tobj::LoadOptions {
            single_index: true,
            ..Default::default()
        },
    );
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
    let mats = mats.unwrap();
    // We expect a single model with no materials
    assert_eq!(models.len(), 1);
    assert!(mats.is_empty());
    // Confirm our line list is loaded correctly
    assert_eq!(models[0].name, "Lines");
    let mesh = &models[0].mesh;
    assert!(mesh.normals.is_empty());
    assert!(mesh.texcoords.is_empty());
    assert_eq!(mesh.material_id, None);

    // Verify each position is loaded properly
    let expect_pos = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    assert_float_eq!(mesh.positions, expect_pos, r2nd_all <= TOL);
    // Verify the indices are loaded properly
    let expect_idx = vec![0, 1, 1, 2, 2, 0];
    assert_eq!(mesh.indices, expect_idx);
}

#[test]
fn non_triangulated_quad() {
    let m = tobj::load_obj(
        "obj/quad.obj",
        &tobj::LoadOptions {
            single_index: true,
            ..Default::default()
        },
    );
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
    let mats = mats.unwrap();
    assert_eq!(models.len(), 3);
    assert!(mats.is_empty());

    // First one is a quad formed by two triangles
    // so face_arities is empty (all trinagles)
    assert!(models[0].mesh.face_arities.is_empty());

    // Second is a quad face
    assert_eq!(models[1].mesh.face_arities.len(), 1);
    assert_eq!(models[1].mesh.face_arities[0], 4);
    let expect_quad_indices = vec![0, 1, 2, 3];
    assert_eq!(models[1].mesh.indices, expect_quad_indices);

    // Third is a triangle
    assert!(models[2].mesh.face_arities.is_empty());
}

#[test]
fn multiple_face_formats() {
    let m = tobj::load_obj(
        "obj/quad.obj",
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    );
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
    let mats = mats.unwrap();
    assert_eq!(models.len(), 3);
    assert!(mats.is_empty());

    // Confirm each object in the file was loaded properly
    assert_eq!(models[0].name, "Quad");
    let quad = &models[0].mesh;
    assert!(quad.normals.is_empty());
    assert_eq!(quad.material_id, None);
    let quad_expect_pos = vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
    let quad_expect_tex = vec![0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0];
    let quad_expect_idx = vec![0, 1, 2, 0, 2, 3];
    assert_float_eq!(quad.positions, quad_expect_pos, r2nd_all <= TOL);
    assert_float_eq!(quad.texcoords, quad_expect_tex, r2nd_all <= TOL);
    assert_eq!(quad.indices, quad_expect_idx);

    assert_eq!(models[1].name, "Quad_face");
    let quad_face = &models[1].mesh;
    let quad_expect_normals = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
    assert_eq!(quad_face.material_id, None);
    assert_float_eq!(quad_face.positions, quad_expect_pos, r2nd_all <= TOL);
    assert_float_eq!(quad_face.texcoords, quad_expect_tex, r2nd_all <= TOL);
    assert_float_eq!(quad_face.normals, quad_expect_normals, r2nd_all <= TOL);
    assert_eq!(quad_face.indices, quad_expect_idx);

    assert_eq!(models[2].name, "Tri_v_vn");
    let tri = &models[2].mesh;
    let tri_expect_pos = vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    let tri_expect_normals = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
    let tri_expect_idx = vec![0, 1, 2];
    assert_eq!(tri.material_id, None);
    assert_float_eq!(tri.positions, tri_expect_pos, r2nd_all <= TOL);
    assert_float_eq!(tri.normals, tri_expect_normals, r2nd_all <= TOL);
    assert_eq!(tri.indices, tri_expect_idx);
    assert!(tri.texcoords.is_empty());
}

#[allow(clippy::excessive_precision)]
fn validate_cornell(models: Vec<tobj::Model>, mats: Vec<tobj::Material>) {
    // Verify the floor loaded properly
    assert_eq!(models[0].name, "floor");
    let mesh = &models[0].mesh;
    assert_eq!(mesh.material_id, Some(0));
    let expect_indices = vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11];
    let expect_verts = vec![
        552.799988, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000,
        559.200012, 549.599976, 0.000000, 559.200012, 290.000000, 0.000000, 114.000000, 240.000000,
        0.000000, 272.000000, 82.000000, 0.000000, 225.000000, 130.000000, 0.000000, 65.000000,
        472.000000, 0.000000, 406.000000, 314.000000, 0.000000, 456.000000, 265.000000, 0.000000,
        296.000000, 423.000000, 0.000000, 247.000000,
    ];
    assert_eq!(mesh.indices, expect_indices);
    assert_float_eq!(mesh.positions, expect_verts, r2nd_all <= TOL);

    // Verify the light loaded properly
    assert_eq!(models[1].name, "light");
    let mesh = &models[1].mesh;
    assert_eq!(mesh.material_id, Some(3));
    let expect_indices = vec![0, 1, 2, 0, 2, 3];
    let expect_verts = vec![
        343.000000, 548.000000, 227.000000, 343.000000, 548.000000, 332.000000, 213.000000,
        548.000000, 332.000000, 213.000000, 548.000000, 227.000000,
    ];
    assert_eq!(mesh.indices, expect_indices);
    assert_float_eq!(mesh.positions, expect_verts, r2nd_all <= TOL);

    // Verify the ceiling loaded properly
    assert_eq!(models[2].name, "ceiling");
    let mesh = &models[2].mesh;
    assert_eq!(mesh.material_id, Some(0));
    let expect_verts = vec![
        556.000000, 548.799988, 0.000000, 556.000000, 548.799988, 559.200012, 0.000000, 548.799988,
        559.200012, 0.000000, 548.799988, 0.000000,
    ];
    assert_eq!(mesh.indices, expect_indices);
    assert_float_eq!(mesh.positions, expect_verts, r2nd_all <= TOL);

    // Verify the back wall loaded properly
    assert_eq!(models[3].name, "back_wall");
    let mesh = &models[3].mesh;
    assert_eq!(mesh.material_id, Some(0));
    let expect_verts = vec![
        549.599976, 0.000000, 559.200012, 0.000000, 0.000000, 559.200012, 0.000000, 548.799988,
        559.200012, 556.000000, 548.799988, 559.200012,
    ];
    assert_eq!(mesh.indices, expect_indices);
    assert_float_eq!(mesh.positions, expect_verts, r2nd_all <= TOL);

    // Verify the green wall loaded properly
    assert_eq!(models[4].name, "green_wall");
    let mesh = &models[4].mesh;
    assert_eq!(mesh.material_id, Some(4));
    let expect_verts = vec![
        0.000000, 0.000000, 559.200012, 0.000000, 0.000000, 0.000000, 0.000000, 548.799988,
        0.000000, 0.000000, 548.799988, 559.200012,
    ];
    assert_eq!(mesh.indices, expect_indices);
    assert_float_eq!(mesh.positions, expect_verts, r2nd_all <= TOL);

    // Verify the red wall loaded properly
    assert_eq!(models[5].name, "red_wall");
    let mesh = &models[5].mesh;
    assert_eq!(mesh.material_id, Some(1));
    let expect_verts = vec![
        552.799988, 0.000000, 0.000000, 549.599976, 0.000000, 559.200012, 556.000000, 548.799988,
        559.200012, 556.000000, 548.799988, 0.000000,
    ];
    assert_eq!(mesh.indices, expect_indices);
    assert_float_eq!(mesh.positions, expect_verts, r2nd_all <= TOL);

    // Verify the short block loaded properly
    assert_eq!(models[6].name, "short_block");
    let mesh = &models[6].mesh;
    assert_eq!(mesh.material_id, Some(0));
    let expect_indices = vec![
        0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16, 17,
        18, 16, 18, 19,
    ];
    let expect_verts = vec![
        130.000000, 165.000000, 65.000000, 82.000000, 165.000000, 225.000000, 240.000000,
        165.000000, 272.000000, 290.000000, 165.000000, 114.000000, 290.000000, 0.000000,
        114.000000, 290.000000, 165.000000, 114.000000, 240.000000, 165.000000, 272.000000,
        240.000000, 0.000000, 272.000000, 130.000000, 0.000000, 65.000000, 130.000000, 165.000000,
        65.000000, 290.000000, 165.000000, 114.000000, 290.000000, 0.000000, 114.000000, 82.000000,
        0.000000, 225.000000, 82.000000, 165.000000, 225.000000, 130.000000, 165.000000, 65.000000,
        130.000000, 0.000000, 65.000000, 240.000000, 0.000000, 272.000000, 240.000000, 165.000000,
        272.000000, 82.000000, 165.000000, 225.000000, 82.000000, 0.000000, 225.000000,
    ];
    assert_eq!(mesh.indices, expect_indices);
    assert_float_eq!(mesh.positions, expect_verts, r2nd_all <= TOL);

    // Verify the tall block loaded properly
    assert_eq!(models[7].name, "tall_block");
    let mesh = &models[7].mesh;
    assert_eq!(mesh.material_id, Some(0));
    let expect_indices = vec![
        0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16, 17,
        18, 16, 18, 19,
    ];
    let expect_verts = vec![
        423.000000, 330.000000, 247.000000, 265.000000, 330.000000, 296.000000, 314.000000,
        330.000000, 456.000000, 472.000000, 330.000000, 406.000000, 423.000000, 0.000000,
        247.000000, 423.000000, 330.000000, 247.000000, 472.000000, 330.000000, 406.000000,
        472.000000, 0.000000, 406.000000, 472.000000, 0.000000, 406.000000, 472.000000, 330.000000,
        406.000000, 314.000000, 330.000000, 456.000000, 314.000000, 0.000000, 456.000000,
        314.000000, 0.000000, 456.000000, 314.000000, 330.000000, 456.000000, 265.000000,
        330.000000, 296.000000, 265.000000, 0.000000, 296.000000, 265.000000, 0.000000, 296.000000,
        265.000000, 330.000000, 296.000000, 423.000000, 330.000000, 247.000000, 423.000000,
        0.000000, 247.000000,
    ];
    assert_eq!(mesh.indices, expect_indices);
    assert_float_eq!(mesh.positions, expect_verts, r2nd_all <= TOL);

    // Verify white material loaded properly
    assert_eq!(mats[0].name, "white");
    let mat = &mats[0];
    assert_float_eq!(mat.ambient.unwrap(), [0.0, 0.0, 0.0], r2nd_all <= TOL);
    assert_float_eq!(mat.diffuse.unwrap(), [1.0, 1.0, 1.0], r2nd_all <= TOL);
    assert_float_eq!(mat.specular.unwrap(), [0.0, 0.0, 0.0], r2nd_all <= TOL);
    assert_eq!(
        mat.unknown_param.get("Ke").map(|s| s.as_ref()),
        Some("1 1 1")
    );
    assert_eq!(mat.illumination_model, None);

    // Verify red material loaded properly
    assert_eq!(mats[1].name, "red");
    let mat = &mats[1];
    assert_float_eq!(mat.ambient.unwrap(), [0.0, 0.0, 0.0], r2nd_all <= TOL);
    assert_float_eq!(mat.diffuse.unwrap(), [1.0, 0.0, 0.0], r2nd_all <= TOL);
    assert_float_eq!(mat.specular.unwrap(), [0.0, 0.0, 0.0], r2nd_all <= TOL);
    assert_eq!(mat.illumination_model, Some(2));
    assert_eq!(
        mat.ambient_texture,
        Some("this ambient texture has spaces.jpg".to_owned())
    );
    assert_eq!(
        mat.diffuse_texture,
        Some("this diffuse texture has spaces.jpg".to_owned())
    );
    assert_eq!(
        mat.specular_texture,
        Some("this specular texture has spaces.jpg".to_owned())
    );
    assert_eq!(
        mat.normal_texture,
        Some("this normal texture has spaces.jpg".to_owned())
    );
    assert_eq!(
        mat.shininess_texture,
        Some("this shininess texture has spaces.jpg".to_owned())
    );
    assert_eq!(
        mat.dissolve_texture,
        Some("this dissolve texture has spaces.jpg".to_owned())
    );

    // Verify blue material loaded properly
    assert_eq!(mats[2].name, "blue");
    let mat = &mats[2];
    assert_float_eq!(mat.ambient.unwrap(), [0.0, 0.0, 0.0], r2nd_all <= TOL);
    assert_float_eq!(mat.diffuse.unwrap(), [0.0, 0.0, 1.0], r2nd_all <= TOL);
    assert_float_eq!(mat.specular.unwrap(), [0.0, 0.0, 0.0], r2nd_all <= TOL);
    assert_eq!(mat.shininess, Some(10.0));
    assert_eq!(mat.unknown_param.len(), 1);
    assert_eq!(
        mat.unknown_param.get("crazy_unknown"),
        Some(&"Wierd stuff here".to_string())
    );

    // Verify light material loaded properly
    assert_eq!(mats[3].name, "light");
    let mat = &mats[3];
    assert_float_eq!(mat.ambient.unwrap(), [20.0, 20.0, 20.0], r2nd_all <= TOL);
    assert_float_eq!(mat.diffuse.unwrap(), [1.0, 1.0, 1.0], r2nd_all <= TOL);
    assert_float_eq!(mat.specular.unwrap(), [0.0, 0.0, 0.0], r2nd_all <= TOL);
    assert_eq!(mat.dissolve, Some(0.8));
    assert_eq!(mat.optical_density, Some(1.25));

    // Verify green material loaded properly
    assert_eq!(mats[4].name, "green");
    let mat = &mats[4];
    assert_float_eq!(mat.ambient.unwrap(), [0.0, 0.0, 0.0], r2nd_all <= TOL);
    assert_float_eq!(mat.diffuse.unwrap(), [0.0, 1.0, 0.0], r2nd_all <= TOL);
    assert_float_eq!(mat.specular.unwrap(), [0.0, 0.0, 0.0], r2nd_all <= TOL);
    assert_eq!(mat.ambient_texture, Some("dummy_texture.png".to_owned()));
    assert_eq!(mat.diffuse_texture, Some("dummy_texture.png".to_owned()));
    assert_eq!(mat.specular_texture, Some("dummy_texture.png".to_owned()));
    assert_eq!(mat.normal_texture, Some("dummy_texture.png".to_owned()));
    assert_eq!(mat.shininess_texture, Some("dummy_texture.png".to_owned()));
    assert_eq!(mat.dissolve_texture, Some("dummy_texture.png".to_owned()));
}

#[test]
fn test_cornell() {
    let m = tobj::load_obj(
        "obj/cornell_box.obj",
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    );
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
    let mats = mats.unwrap();
    assert_eq!(models.len(), 8);
    assert_eq!(mats.len(), 5);
    validate_cornell(models, mats);
}

#[test]
fn test_custom_material_loader() {
    let m = tobj::load_obj_buf(
        &mut Cursor::new(CORNELL_BOX_OBJ),
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| match p.to_str().unwrap() {
            "cornell_box.mtl" => tobj::load_mtl_buf(&mut Cursor::new(CORNELL_BOX_MTL1)),
            "cornell_box2.mtl" => tobj::load_mtl_buf(&mut Cursor::new(CORNELL_BOX_MTL2)),
            _ => unreachable!(),
        },
    );
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
    let mats = mats.unwrap();
    assert_eq!(models.len(), 8);
    assert_eq!(mats.len(), 5);
    validate_cornell(models, mats);
}

#[cfg(feature = "async")]
#[test]
fn test_async_custom_material_loader() {
    let m = tokio_test::block_on(tobj::load_obj_buf_async(
        &mut Cursor::new(CORNELL_BOX_OBJ),
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            match p.as_str() {
                "cornell_box.mtl" => tobj::load_mtl_buf(&mut Cursor::new(CORNELL_BOX_MTL1)),
                "cornell_box2.mtl" => tobj::load_mtl_buf(&mut Cursor::new(CORNELL_BOX_MTL2)),
                _ => unreachable!(),
            }
        },
    ));
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
    let mats = mats.unwrap();
    assert_eq!(models.len(), 8);
    assert_eq!(mats.len(), 5);
    validate_cornell(models, mats);
}

#[test]
fn test_custom_material_loader_files() {
    let dir = env::current_dir().unwrap();
    let mut cornell_box_obj = dir.clone();
    cornell_box_obj.push("obj/cornell_box.obj");
    let mut cornell_box_file = BufReader::new(File::open(cornell_box_obj.as_path()).unwrap());

    let mut cornell_box_mtl1 = dir.clone();
    cornell_box_mtl1.push("obj/cornell_box.mtl");

    let mut cornell_box_mtl2 = dir.clone();
    cornell_box_mtl2.push("obj/cornell_box2.mtl");

    let m = tobj::load_obj_buf(
        &mut cornell_box_file,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| match p.file_name().unwrap().to_str().unwrap() {
            "cornell_box.mtl" => {
                let f = File::open(cornell_box_mtl1.as_path()).unwrap();
                tobj::load_mtl_buf(&mut BufReader::new(f))
            }
            "cornell_box2.mtl" => {
                let f = File::open(cornell_box_mtl2.as_path()).unwrap();
                tobj::load_mtl_buf(&mut BufReader::new(f))
            }
            _ => unreachable!(),
        },
    );
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
    let mats = mats.unwrap();
    assert_eq!(models.len(), 8);
    assert_eq!(mats.len(), 5);
    validate_cornell(models, mats);
}

#[test]
fn test_invalid_index() {
    let m = tobj::load_obj(
        "obj/invalid_index.obj",
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    );
    assert!(m.is_err());
    let err = m.err().unwrap();
    assert_eq!(err, tobj::LoadError::FaceVertexOutOfBounds);
}
