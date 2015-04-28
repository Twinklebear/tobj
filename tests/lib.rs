extern crate tobj;

use std::path::Path;

#[test]
fn simple_triangle() {
    let m = tobj::load_obj(&Path::new("triangle.obj"));
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
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
	let expect_pos = vec![
		0.0, 0.0, 0.0,
		1.0, 0.0, 0.0,
		0.0, 1.0, 0.0
	];
	assert_eq!(mesh.positions, expect_pos);
	// Verify the indices are loaded properly
	let expect_idx = vec![0, 1, 2];
	assert_eq!(mesh.indices, expect_idx);
}

#[test]
fn multiple_face_formats() {
    let m = tobj::load_obj(&Path::new("quad.obj"));
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
	assert_eq!(models.len(), 3);
	assert!(mats.is_empty());
	
	// Confirm each object in the file was loaded properly
	assert_eq!(models[0].name, "Quad");
	let quad = &models[0].mesh;
	assert!(quad.normals.is_empty());
	assert_eq!(quad.material_id, None);
	let quad_expect_pos = vec![
		0.0, 1.0, 0.0,
		0.0, 0.0, 0.0,
		1.0, 0.0, 0.0,
		1.0, 1.0, 0.0,
	];
	let quad_expect_tex = vec![
		0.0, 1.0,
		0.0, 0.0,
		1.0, 0.0,
		1.0, 1.0,
	];
	let quad_expect_idx = vec![0, 1, 2, 0, 2, 3];
	assert_eq!(quad.positions, quad_expect_pos);
	assert_eq!(quad.texcoords, quad_expect_tex);
	assert_eq!(quad.indices, quad_expect_idx);

	assert_eq!(models[1].name, "Quad_face");
	let quad_face = &models[1].mesh;
	let quad_expect_normals = vec![
		0.0, 0.0, 1.0,
		0.0, 0.0, 1.0,
		0.0, 0.0, 1.0,
		0.0, 0.0, 1.0,
	];
	assert_eq!(quad_face.material_id, None);
	assert_eq!(quad_face.positions, quad_expect_pos);
	assert_eq!(quad_face.texcoords, quad_expect_tex);
	assert_eq!(quad_face.normals, quad_expect_normals);
	assert_eq!(quad_face.indices, quad_expect_idx);

	assert_eq!(models[2].name, "Tri_v_vn");
	let tri = &models[2].mesh;
	let tri_expect_pos = vec![
		0.0, 1.0, 0.0,
		0.0, 0.0, 0.0,
		1.0, 0.0, 0.0,
	];
	let tri_expect_normals = vec![
		0.0, 0.0, 1.0,
		0.0, 0.0, 1.0,
		0.0, 0.0, 1.0,
	];
	let tri_expect_idx = vec![0, 1, 2];
	assert_eq!(tri.material_id, None);
	assert_eq!(tri.positions, tri_expect_pos);
	assert_eq!(tri.normals, tri_expect_normals);
	assert_eq!(tri.indices, tri_expect_idx);
	assert!(tri.texcoords.is_empty());
}

/*
#[test]
fn test_cornell() {
    let m = tobj::load_obj(&Path::new("cornell_box.obj"));
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
    tobj::print_model_info(&models, &mats);
}
*/

