//! Tiny OBJ loader, inspired by Syoyo's excellent [tinyobjloader](https://github.com/syoyo/tinyobjloader)

#![allow(dead_code)]

use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::fs::File;
use std::collections::HashMap;
use std::str::{FromStr, Split};

/// A mesh for some model containing its triangle geometry
/// This object could be a single polygon group or object within a file
/// that defines multiple groups/objects or be the only mesh within the file
/// if it only contains a single mesh
#[derive(Debug, Clone)]
pub struct Mesh {
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub texcoords: Vec<f32>,
    pub indices: Vec<u32>,
    pub material_id: Option<usize>,
}

impl Mesh {
    /// Create a new mesh specifying the geometry for the mesh
    pub fn new(pos: Vec<f32>, norm: Vec<f32>, tex: Vec<f32>, indices: Vec<u32>, material_id: Option<usize>) -> Mesh {
        Mesh { positions: pos, normals: norm, texcoords: tex, indices: indices, material_id: material_id }
    }
    /// Create a new empty mesh
    pub fn empty() -> Mesh {
        Mesh { positions: Vec::new(), normals: Vec::new(), texcoords: Vec::new(), indices: Vec::new(),
               material_id: None }
    }
}

/// A named model within the file
/// This could be a group or object or the single model exported by the file
#[derive(Debug)]
pub struct Model {
    pub mesh: Mesh,
    pub name: String,
}

impl Model {
    /// Create a new model, associating a name with a mesh
    pub fn new(mesh: Mesh, name: String) -> Model {
        Model { mesh: mesh, name: name }
    }
}


/// A material that may be referenced by one or more meshes. Standard MTL attributes are supported
/// and any unrecognized ones will be stored as Strings in the `unknown_param` HashMap
#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
    pub dissolve: f32,
    pub ambient_texture: String,
    pub diffuse_texture: String,
    pub specular_texture: String,
    pub normal_texture: String,
    pub dissolve_texture: String,
    pub unknown_param: HashMap<String, String>,
}

impl Material {
    pub fn empty() -> Material {
        Material { name: String::new(), ambient: [0.0; 3], diffuse: [0.0; 3], specular: [0.0; 3],
                   shininess: 0.0, dissolve: 1.0, ambient_texture: String::new(),
                   diffuse_texture: String::new(), specular_texture: String::new(),
                   normal_texture: String::new(), dissolve_texture: String::new(),
                   unknown_param: HashMap::new() }
    }
}

/// TODO: Decide on various errors we'll return
#[derive(Debug)]
pub enum LoadError {
    OpenFileFailed,
    ReadError,
    UnrecognizedCharacter,
    PositionParseError,
    NormalParseError,
    TexcoordParseError,
    FaceParseError,
    MaterialParseError,
    InvalidObjectName,
    GenericFailure,
}

/// LoadResult is a result containing all the models loaded from the file and any materials from
/// referenced material libraries, or an error that occured while loading
pub type LoadResult = Result<(Vec<Model>, Vec<Material>), LoadError>;

/// MTLLoadResult is a result containing all the materials loaded from the file and a map of MTL
/// name to index or the error that occured while loading
pub type MTLLoadResult = Result<(Vec<Material>, HashMap<String, usize>), LoadError>;

/// Struct storing indices corresponding to the vertex
/// Some vertices may not have texcoords or normals, 0 is used to indicate this
/// as OBJ indices begin at 1
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug, Copy, Clone)]
struct VertexIndices {
    pub v: usize,
    pub vt: usize,
    pub vn: usize,
}

impl VertexIndices {
    /// Parse the vertex indices from the face string
    /// Valid face strings are those that are valid for a Wavefront OBJ file
    /// Also handles relative face indices (negative values) which is why passing the number of
    /// positions, texcoords and normals is required
    /// Returns None if the face string is invalid
    pub fn parse(face_str: &str, pos_sz: usize, tex_sz: usize, norm_sz: usize) -> Option<VertexIndices> {
        let mut indices = [0; 3];
        for i in face_str.split('/').enumerate() {
            // Catch case of v//vn where we'll find an empty string in one of our splits
            // since there are no texcoords for the mesh
            if !i.1.is_empty() {
                match isize::from_str(i.1) {
                    Ok(x) => {
                        // Handle relative indices
                        indices[i.0] =
                            if x < 0 {
                                match i.0 {
                                    0 => (x + pos_sz as isize) as usize,
                                    1 => (x + tex_sz as isize) as usize,
                                    2 => (x + norm_sz as isize) as usize,
                                    _ => panic!("Invalid number of elements for a face (> 3)!"),
                                }
                            } else {
                                (x - 1) as usize
                            };
                    },
                    Err(_) => return None,
                }
            }
        }
        Some(VertexIndices { v: indices[0], vt: indices[1], vn: indices[2] })
    }
}

/// Enum representing either a quad or triangle face, storing indices for the face vertices
#[derive(Debug)]
enum Face {
    Triangle(VertexIndices, VertexIndices, VertexIndices),
    Quad(VertexIndices, VertexIndices, VertexIndices, VertexIndices)
}

/// Parse the floatn information from the words, words is an iterator over the float strings
/// Returns false if parsing failed
fn parse_floatn(val_str: Split<char>, vals: &mut Vec<f32>, n: usize) -> bool {
    let sz = vals.len();
    for p in val_str {
        if p.is_empty() {
            continue;
        }
        if sz + n == vals.len() {
            return true;
        }
        // This stupid trim is only needed b/c words isn't stable
        match FromStr::from_str(p.trim()) {
            Ok(x) => vals.push(x),
            Err(_) => return false,
        }
    }
    // Require that we found the desired number of floats
    sz + n == vals.len()
}

/// Parse the float3 into the array passed, returns false if parsing failed
fn parse_float3(val_str: Split<char>, vals: &mut [f32; 3]) -> bool {
    for (i, p) in val_str.enumerate() {
        // This trim is only needed b/c words isn't stable
        match FromStr::from_str(p.trim()) {
            Ok(x) => vals[i] = x,
            Err(_) => return false,
        }
    }
    true
}

/// Parse vertex indices for a face and append it to the list of faces passed
/// Also handles relative face indices (negative values) which is why passing the number of
/// positions, texcoords and normals is required
/// returns false if an error occured parsing the face
fn parse_face(face_str: Split<char>, faces: &mut Vec<Face>, pos_sz: usize, tex_sz: usize, norm_sz: usize) -> bool {
    let mut indices = Vec::new();
    for f in face_str {
        match VertexIndices::parse(f, pos_sz, tex_sz, norm_sz) {
            Some(v) => indices.push(v),
            None => return false,
        }
    }
    // Check if we read a triangle or a quad face and push it on
    match indices.len() {
        3 => faces.push(Face::Triangle(indices[0], indices[1], indices[2])),
        4 => faces.push(Face::Quad(indices[0], indices[1], indices[2], indices[3])),
        _ => return false,
    }
    true
}

/// Add a vertex to a mesh by either re-using an existing index (eg. it's in the index_map)
/// or appending the position, texcoord and normal as appropriate and creating a new vertex
fn add_vertex(mesh: &mut Mesh, index_map: &mut HashMap<VertexIndices, u32>, vert: &VertexIndices,
              pos: &Vec<f32>, texcoord: &Vec<f32>, normal: &Vec<f32>) {
    match index_map.get(vert){
        Some(&i) => mesh.indices.push(i),
        None => {
            // Add the vertex to the mesh
            mesh.positions.push(pos[vert.v * 3]);
            mesh.positions.push(pos[vert.v * 3 + 1]);
            mesh.positions.push(pos[vert.v * 3 + 2]);
            if !texcoord.is_empty() {
                mesh.texcoords.push(texcoord[vert.vt * 2]);
                mesh.texcoords.push(texcoord[vert.vt * 2 + 1]);
            }
            if !normal.is_empty() {
                mesh.normals.push(normal[vert.vn * 3]);
                mesh.normals.push(normal[vert.vn * 3 + 1]);
                mesh.normals.push(normal[vert.vn * 3 + 2]);
            }
            let next = index_map.len() as u32;
            mesh.indices.push(next);
            index_map.insert(*vert, next);
        }
    }
}

/// Export a list of faces to a mesh and return it, converting quads to tris
fn export_faces(pos: &Vec<f32>, texcoord: &Vec<f32>, normal: &Vec<f32>, faces: &Vec<Face>, mat_id: Option<usize>) -> Mesh {
    let mut index_map = HashMap::new();
    let mut mesh = Mesh::empty();
    mesh.material_id = mat_id;
    // TODO: When drain becomes stable we should use that, since we clear `faces` later anyway
    for f in faces {
        match *f {
            Face::Triangle(ref a, ref b, ref c) => {
                add_vertex(&mut mesh, &mut index_map, a, pos, texcoord, normal);
                add_vertex(&mut mesh, &mut index_map, b, pos, texcoord, normal);
                add_vertex(&mut mesh, &mut index_map, c, pos, texcoord, normal);
            },
            Face::Quad(ref a, ref b, ref c, ref d) => {
                add_vertex(&mut mesh, &mut index_map, a, pos, texcoord, normal);
                add_vertex(&mut mesh, &mut index_map, b, pos, texcoord, normal);
                add_vertex(&mut mesh, &mut index_map, c, pos, texcoord, normal);

                add_vertex(&mut mesh, &mut index_map, a, pos, texcoord, normal);
                add_vertex(&mut mesh, &mut index_map, c, pos, texcoord, normal);
                add_vertex(&mut mesh, &mut index_map, d, pos, texcoord, normal);
            }
        }
    }
    mesh
}

/// Load the various meshes in an OBJ file
pub fn load_obj(file_name: &Path) -> LoadResult {
    let file = match File::open(file_name) {
        Ok(f) => f,
        Err(e) => {
            println!("tobj::load_obj - failed to open {:?} due to {}", file_name, e);
            return Err(LoadError::OpenFileFailed);
        },
    };
    let mut reader = BufReader::new(file);
    load_obj_buf(&mut reader, file_name.parent())
}

/// Load the materials defined in a MTL file
pub fn load_mtl(file_name: &Path) -> MTLLoadResult {
    let file = match File::open(file_name) {
        Ok(f) => f,
        Err(e) => {
            println!("tobj::load_mtl - failed to open {:?} due to {}", file_name, e);
            return Err(LoadError::OpenFileFailed);
        },
    };
    let mut reader = BufReader::new(file);
    load_mtl_buf(&mut reader)
}

/// Load the various meshes in an OBJ buffer. `base_path` specifies the path prefix to apply to
/// referenced material libs
fn load_obj_buf<B: BufRead>(reader: &mut B, base_path: Option<&Path>) -> LoadResult {
    let mut models = Vec::new();
    let mut materials = Vec::new();
    let mut mat_map = HashMap::new();

    let mut tmp_pos = Vec::new();
    let mut tmp_texcoord = Vec::new();
    let mut tmp_normal = Vec::new();
    let mut tmp_faces: Vec<Face> = Vec::new();
    // name of the current object being parsed
    let mut name = String::new();
    // material used by the current object being parsed
    let mut mat_id = None;
    for line in reader.lines() {
        // We just need the line for debugging for a bit
        // TODO: Switch back to using `words` when it becomes stable
        let (line, mut words) = match line {
            Ok(ref line) => (&line[..], line[..].trim().split(' ')),
            Err(e) => {
                println!("tobj::load_obj - failed to read line due to {}", e);
                return Err(LoadError::ReadError);
            },
        };
        match words.next() {
            Some("#") | None => continue,
            Some("v") => {
                if !parse_floatn(words, &mut tmp_pos, 3) {
                    return Err(LoadError::PositionParseError);
                }
            },
            Some("vt") => {
                if !parse_floatn(words, &mut tmp_texcoord, 2) {
                    return Err(LoadError::TexcoordParseError);
                }
            },
            Some("vn") => {
                if !parse_floatn(words, &mut tmp_normal, 3) {
                    return Err(LoadError::NormalParseError);
                }
            },
            Some("f") => {
                if !parse_face(words, &mut tmp_faces, tmp_pos.len() / 3, tmp_texcoord.len() / 2, tmp_normal.len() / 3) {
                    return Err(LoadError::FaceParseError);
                }
            },
            // Just treating object and group tags identically. Should there be different behavior
            // for them?
            Some("o") | Some("g") => {
                // If we were already parsing an object then a new object name
                // signals the end of the current one, so push it onto our list of objects
                if !name.is_empty() && !tmp_faces.is_empty() {
                    models.push(Model::new(export_faces(&tmp_pos, &tmp_texcoord, &tmp_normal, &tmp_faces, mat_id), name));
                    tmp_faces.clear();
                }
                match words.next() {
                    Some(n) => name = n.to_string(),
                    None => return Err(LoadError::InvalidObjectName),
                }
                //println!("Beginning to parse new object/group: {}", name);
            },
            Some("mtllib") => {
                if let Some(mtllib) = words.next() {
                    let mat_file = match base_path {
                        Some(bp) => bp.join(mtllib),
                        None => Path::new(mtllib).to_path_buf(),
                    };
                  //  println!("Will parse material lib {:?}", mat_file);
                    match load_mtl(mat_file.as_path()) {
                        Ok((mats, map)) => {
                            materials = mats;
                            mat_map = map;
                        },
                        Err(e) => return Err(e),
                    }
                }
            },
            Some("usemtl") => {
                if let Some(mat_name) = words.next() {
                    match mat_map.get(mat_name) {
                        Some(m) => mat_id = Some(*m),
                        None => {
                            mat_id = None;
                            println!("Warning: Object {} refers to unfound material: {}", name, mat_name);
                        }
                    }
                }
            },
            // TODO: throw error on unrecognized character? Currently with split we get a newline
            // and incorrectly through so this is off temporarily. Blocked until `words` becomes
            // stable
            Some(_) => { /*return Err(LoadError::UnrecognizedCharacter) */ },
        }
    }
    // For the last object in the file we won't encounter another object name to tell us when it's
    // done, so if we're parsing an object push the last one on the list as well
    if !name.is_empty() {
        models.push(Model::new(export_faces(&tmp_pos, &tmp_texcoord, &tmp_normal, &tmp_faces, mat_id), name));
    }
    Ok((models, materials))
}

/// Load the various materials in a MTL buffer
fn load_mtl_buf<B: BufRead>(reader: &mut B) -> MTLLoadResult {
    let mut materials = Vec::new();
    let mut mat_map = HashMap::new();
    // The current material being parsed
    let mut cur_mat = Material::empty();
    for line in reader.lines() {
        // We just need the line for debugging for a bit
        // TODO: Switch back to using `words` when it becomes stable
        let (line, mut words) = match line {
            Ok(ref line) => (&line[..], line[..].trim().split(' ')),
            Err(e) => {
                println!("tobj::load_obj - failed to read line due to {}", e);
                return Err(LoadError::ReadError);
            },
        };
        match words.next() {
            Some("#") | None => continue,
            Some("newmtl") => {
                // If we were passing a material save it out to our vector
                if !cur_mat.name.is_empty() {
                    mat_map.insert(cur_mat.name.clone(), materials.len());
                    materials.push(cur_mat);
                }
                cur_mat = Material::empty();
                match words.next() {
                    Some(n) => cur_mat.name = n.to_string(),
                    None => return Err(LoadError::InvalidObjectName),
                }
            },
            Some("Ka") => {
                if !parse_float3(words, &mut cur_mat.ambient) {
                    return Err(LoadError::MaterialParseError);
                }
            },
            Some("Kd") => {
                if !parse_float3(words, &mut cur_mat.diffuse) {
                    return Err(LoadError::MaterialParseError);
                }
            },
            Some("Ks") => {
                if !parse_float3(words, &mut cur_mat.specular) {
                    return Err(LoadError::MaterialParseError);
                }
            },
            Some("Ns") => {
                if let Some(p) = words.next() {
                    // This trim is only needed b/c words isn't stable
                    match FromStr::from_str(p.trim()) {
                        Ok(x) => cur_mat.shininess = x,
                        Err(_) => return Err(LoadError::MaterialParseError),
                    }
                } else {
                    return Err(LoadError::MaterialParseError);
                }
            },
            Some("d") => {
                if let Some(p) = words.next() {
                    // This trim is only needed b/c words isn't stable
                    match FromStr::from_str(p.trim()) {
                        Ok(x) => cur_mat.dissolve = x,
                        Err(_) => return Err(LoadError::MaterialParseError),
                    }
                } else {
                    return Err(LoadError::MaterialParseError);
                }
            },
            Some("map_Ka") => {
                match words.next() {
                    Some(tex) => cur_mat.ambient_texture = tex.to_string(),
                    None => return Err(LoadError::MaterialParseError),
                }
            },
            Some("map_Kd") => {
                match words.next() {
                    Some(tex) => cur_mat.diffuse_texture = tex.to_string(),
                    None => return Err(LoadError::MaterialParseError),
                }
            },
            Some("map_Ks") => {
                match words.next() {
                    Some(tex) => cur_mat.specular_texture = tex.to_string(),
                    None => return Err(LoadError::MaterialParseError),
                }
            },
            Some("map_Ns") => {
                match words.next() {
                    Some(tex) => cur_mat.normal_texture = tex.to_string(),
                    None => return Err(LoadError::MaterialParseError),
                }
            },
            Some("map_d") => {
                match words.next() {
                    Some(tex) => cur_mat.dissolve_texture = tex.to_string(),
                    None => return Err(LoadError::MaterialParseError),
                }
            },
            Some(unknown) => {
                if !unknown.is_empty() {
                    let param = line[unknown.len()..].trim().to_string();
                    cur_mat.unknown_param.insert(unknown.to_string(), param);
                }
            },
        }
    }
    // Finalize the last material we were parsing
    if !cur_mat.name.is_empty() {
        mat_map.insert(cur_mat.name.clone(), materials.len());
        materials.push(cur_mat);
    }
    Ok((materials, mat_map))
}

/// Print out all loaded properties of some models and associated materials (once mats are added)
pub fn print_model_info(models: &Vec<Model>, materials: &Vec<Material>) {
    println!("# of models: {}", models.len());
    println!("# of materials: {}", materials.len());
    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;
        println!("model[{}].name = {}", i, m.name);
        println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);

        println!("Size of model[{}].indices: {}", i, mesh.indices.len());
        for f in 0..(mesh.indices.len() / 3) {
            println!("    idx[{}] = {}, {}, {}.", f, mesh.indices[3 * f], mesh.indices[3 * f + 1], mesh.indices[3 * f + 2]);
        }

        println!("model[{}].vertices: {}", i, mesh.positions.len());
        assert!(mesh.positions.len() % 3 == 0);
        for v in 0..(mesh.positions.len() / 3) {
            println!("    v[{}] = ({}, {}, {})", v, mesh.positions[3 * v], mesh.positions[3 * v + 1], mesh.positions[3 * v + 2]);
        }
        print_material_info(materials);
    }
}

/// Print out all loaded properties of some materials
fn print_material_info(materials: &Vec<Material>) {
    for (i, m) in materials.iter().enumerate() {
        println!("material[{}].name = {}", i, m.name);
        println!("    material.Ka = ({}, {}, {})", m.ambient[0], m.ambient[1], m.ambient[2]);
        println!("    material.Kd = ({}, {}, {})", m.diffuse[0], m.diffuse[1], m.diffuse[2]);
        println!("    material.Ks = ({}, {}, {})", m.specular[0], m.specular[1], m.specular[2]);
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

#[test]
fn test_tri() {
    let m = load_obj(&Path::new("triangle.obj"));
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
    print_model_info(&models, &mats);
}

#[test]
fn test_quad() {
    let m = load_obj(&Path::new("quad.obj"));
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
    print_model_info(&models, &mats);
}

#[test]
fn test_cornell() {
    let m = load_obj(&Path::new("cornell_box.obj"));
    assert!(m.is_ok());
    let (models, mats) = m.unwrap();
    print_model_info(&models, &mats);
}

