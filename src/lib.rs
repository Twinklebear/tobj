//! Tiny OBJ loader, inspired by Syoyo's excellent [tinyobjloader](https://github.com/syoyo/tinyobjloader).
//! Aims to be a simple and lightweight option for loading OBJ files, simply returns two vecs
//! containing loaded models and materials. All models are made of triangles, any quad or polygon faces
//! in an OBJ file will be converted to triangles. Note that only polygons that are trivially
//! convertible to triangle fans are supported, arbitrary polygons may not behave as expected.
//! The best solution would be to re-export your mesh using only triangles in your modeling software.
//!
//! # Example
//! In this simple example we load the classic Cornell Box model that only defines positions and
//! print out its attributes. This example is a slightly trimmed down version of `print_model_info`
//! and `print_material_info` combined together, see them for a version that also prints out
//! normals and texture coordinates if the model has them.
//! 
//! ```
//! use std::path::Path;
//! use tobj;
//!
//! let cornell_box = tobj::load_obj(&Path::new("cornell_box.obj"));
//! assert!(cornell_box.is_ok());
//! let (models, materials) = cornell_box.unwrap();
//!
//! println!("# of models: {}", models.len());
//! println!("# of materials: {}", materials.len());
//! for (i, m) in models.iter().enumerate() {
//! 	let mesh = &m.mesh;
//! 	println!("model[{}].name = \'{}\'", i, m.name);
//! 	println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);
//! 
//! 	println!("Size of model[{}].indices: {}", i, mesh.indices.len());
//! 	for f in 0..mesh.indices.len() / 3 {
//! 		println!("    idx[{}] = {}, {}, {}.", f, mesh.indices[3 * f],
//! 			mesh.indices[3 * f + 1], mesh.indices[3 * f + 2]);
//! 	}
//! 
//! 	// Normals and texture coordinates are also loaded, but not printed in this example
//! 	println!("model[{}].vertices: {}", i, mesh.positions.len() / 3);
//! 	assert!(mesh.positions.len() % 3 == 0);
//! 	for v in 0..mesh.positions.len() / 3 {
//! 		println!("    v[{}] = ({}, {}, {})", v, mesh.positions[3 * v],
//! 			mesh.positions[3 * v + 1], mesh.positions[3 * v + 2]);
//! 	}
//! }
//! for (i, m) in materials.iter().enumerate() {
//! 	println!("material[{}].name = \'{}\'", i, m.name);
//! 	println!("    material.Ka = ({}, {}, {})", m.ambient[0], m.ambient[1], m.ambient[2]);
//! 	println!("    material.Kd = ({}, {}, {})", m.diffuse[0], m.diffuse[1], m.diffuse[2]);
//! 	println!("    material.Ks = ({}, {}, {})", m.specular[0], m.specular[1], m.specular[2]);
//! 	println!("    material.Ns = {}", m.shininess);
//! 	println!("    material.d = {}", m.dissolve);
//! 	println!("    material.map_Ka = {}", m.ambient_texture);
//! 	println!("    material.map_Kd = {}", m.diffuse_texture);
//! 	println!("    material.map_Ks = {}", m.specular_texture);
//! 	println!("    material.map_Ns = {}", m.normal_texture);
//! 	println!("    material.map_d = {}", m.dissolve_texture);
//! 	for (k, v) in &m.unknown_param {
//! 		println!("    material.{} = {}", k, v);
//! 	}
//! }
//! ```
//! 
//! # Rendering Example
//! For an example of integration with [glium](https://github.com/tomaka/glium) to make a simple OBJ viewer, check out
//! [tobj viewer](https://github.com/Twinklebear/tobj_viewer). Some sample images can be found in
//! tobj viewer's readme or in [this gallery](http://imgur.com/a/xsg6v). 
//!
//! The Rungholt model shown below is reasonably large (6.7M triangles, 12.3M vertices) and is loaded in 8.765s (+/- .56s) using a peak
//! of ~1GB of memory on a Windows 8 machine with an i7-4790k and 16GB of 1600Mhz DDR3 RAM on rustc 1.1.0-nightly 97d4e76c2.
//! The model can be found on [Morgan McGuire's](http://graphics.cs.williams.edu/data/meshes.xml) meshes page and
//! was originally built by kescha. Future work will focus on improving performance and memory usage. 
//!
//! <img src="http://i.imgur.com/k2sC05w.png" alt="Rungholt"
//!     style="display:block; max-width:100%; height:auto">

#![allow(dead_code)]
#![cfg_attr(all(test, feature = "unstable"), feature(test))]

#[cfg(all(test, feature = "unstable"))] extern crate test;

use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::fs::File;
use std::collections::HashMap;
use std::str::FromStr;

/// A mesh made up of triangles loaded from some OBJ file
///
/// It is assumed that all meshes will at least have positions, but normals and texture coordinates
/// are optional. If no normals or texture coordinates where found then the corresponding vecs for
/// the mesh will be empty. Values are stored packed as floats in vecs, eg. the positions member of
/// a loaded mesh will contain `[x, y, z, x, y, z, ...]` which you can then use however you like.
/// Indices are also loaded and may re-use vertices already existing in the mesh, this data is
/// stored in the `indices` member.
///
/// # Example:
/// Load the Cornell box and get the attributes of the first vertex. It's assumed all meshes will
/// have positions (required), but normals and texture coordinates are optional, in which case the
/// corresponding Vec will be empty.
///
/// ```
/// use std::path::Path;
///
/// let cornell_box = tobj::load_obj(&Path::new("cornell_box.obj"));
/// assert!(cornell_box.is_ok());
/// let (models, materials) = cornell_box.unwrap();
///
/// let mesh = &models[0].mesh;
/// let i = mesh.indices[0] as usize;
/// // pos = [x, y, z]
/// let pos = [mesh.positions[i * 3], mesh.positions[i * 3 + 1],
///             mesh.positions[i * 3 + 2]];
///
/// if !mesh.normals.is_empty() {
///     // normal = [x, y, z]
///     let normal = [mesh.normals[i * 3], mesh.normals[i * 3 + 1],
///                   mesh.normals[i * 3 + 2]];
/// }
///
/// if !mesh.texcoords.is_empty() {
///     // texcoord = [u, v];
///     let texcoord = [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]];
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Flattened 3 component floating point vectors, storing positions of vertices in the mesh
    pub positions: Vec<f32>,
    /// Flattened 3 component floating point vectors, storing normals of vertices in the mesh. Not
    /// all meshes have normals, if no normals are specified this Vec will be empty
    pub normals: Vec<f32>,
    /// Flattened 2 component floating point vectors, storing texture coordinates of vertices in
    /// the mesh. Not all meshes have normals, if no texture coordinates are specified this Vec
    /// will be empty
    pub texcoords: Vec<f32>,
    /// Indices for vertices of each triangle. Each face in the mesh is a triangle and the indices
    /// specify the position, normal and texture coordinate for each vertex of the face.
    pub indices: Vec<u32>,
    /// Optional material id associated with this mesh. The material id indexes into the Vec of
    /// Materials loaded from the associated MTL file
    pub material_id: Option<usize>,
}

impl Mesh {
    /// Create a new mesh specifying the geometry for the mesh
    pub fn new(pos: Vec<f32>, norm: Vec<f32>, tex: Vec<f32>, indices: Vec<u32>, material_id: Option<usize>)
		-> Mesh {
        Mesh { positions: pos, normals: norm, texcoords: tex, indices: indices, material_id: material_id }
    }
    /// Create a new empty mesh
    pub fn empty() -> Mesh {
        Mesh { positions: Vec::new(), normals: Vec::new(), texcoords: Vec::new(), indices: Vec::new(),
               material_id: None }
    }
}

/// A named model within the file, associates some mesh with a name that was specified with an `o`
/// or `g` keyword in the OBJ file
#[derive(Clone, Debug)]
pub struct Model {
    /// Mesh used by the model containing its geometry
    pub mesh: Mesh,
    /// Name assigned to this mesh
    pub name: String,
}

impl Model {
    /// Create a new model, associating a name with a mesh
    pub fn new(mesh: Mesh, name: String) -> Model {
        Model { mesh: mesh, name: name }
    }
}

/// A material that may be referenced by one or more meshes. Standard MTL attributes are supported.
/// Any unrecognized parameters will be stored as key-value pairs in the `unknown_param` HashMap,
/// which maps the unknown parameter to the value set for it.
#[derive(Clone, Debug)]
pub struct Material {
    /// Material name as specified in the MTL file
    pub name: String,
    /// Ambient color of the material
    pub ambient: [f32; 3],
    /// Diffuse color of the material
    pub diffuse: [f32; 3],
    /// Specular color of the material
    pub specular: [f32; 3],
    /// Material shininess attribute
    pub shininess: f32,
    /// Dissolve attribute is the alpha term for the material. Referred to as dissolve since that's
    /// what the MTL file format docs refer to it as
    pub dissolve: f32,
    /// Name of the ambient texture file for the material. No path is pre-pended to the texture
    /// file names specified in the MTL file
    pub ambient_texture: String,
    /// Name of the diffuse texture file for the material. No path is pre-pended to the texture
    /// file names specified in the MTL file
    pub diffuse_texture: String,
    /// Name of the specular texture file for the material. No path is pre-pended to the texture
    /// file names specified in the MTL file
    pub specular_texture: String,
    /// Name of the normal map texture file for the material. No path is pre-pended to the texture
    /// file names specified in the MTL file
    pub normal_texture: String,
    /// Name of the alpha map texture file for the material. No path is pre-pended to the texture
    /// file names specified in the MTL file. Referred to as dissolve to match the MTL file format
    /// specification
    pub dissolve_texture: String,
    /// Key value pairs of any unrecognized parameters encountered while parsing the material
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

/// Possible errors that may occur while loading OBJ and MTL files
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
    pub v: isize,
    pub vt: isize,
    pub vn: isize,
}

impl VertexIndices {
    /// Parse the vertex indices from the face string
    /// Valid face strings are those that are valid for a Wavefront OBJ file
    /// Also handles relative face indices (negative values) which is why passing the number of
    /// positions, texcoords and normals is required
    /// Returns None if the face string is invalid
    fn parse(face_str: &str, pos_sz: usize, tex_sz: usize, norm_sz: usize) -> Option<VertexIndices> {
        let mut indices = [-1; 3];
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
                                    0 => x + pos_sz as isize,
                                    1 => x + tex_sz as isize,
                                    2 => x + norm_sz as isize,
                                    _ => panic!("Invalid number of elements for a face (> 3)!"),
                                }
                            } else {
                                x - 1
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
    Quad(VertexIndices, VertexIndices, VertexIndices, VertexIndices),
    Polygon(Vec<VertexIndices>),
}

/// Parse the floatn information from the words, words is an iterator over the float strings
/// Returns false if parsing failed
/// TODO: Switch to SplitWhitespace when it is release (1.1)
fn parse_floatn<'a, T: Iterator<Item = &'a str>>(val_str: T, vals: &mut Vec<f32>, n: usize) -> bool {
    let sz = vals.len();
    for p in val_str {
        if sz + n == vals.len() {
            return true;
        }
        match FromStr::from_str(p) {
            Ok(x) => vals.push(x),
            Err(_) => return false,
        }
    }
    // Require that we found the desired number of floats
    sz + n == vals.len()
}

/// Parse the float3 into the array passed, returns false if parsing failed
/// TODO: Switch to SplitWhitespace when it is release (1.1)
fn parse_float3<'a, T: Iterator<Item = &'a str>>(val_str: T, vals: &mut [f32; 3]) -> bool {
    for (i, p) in val_str.enumerate() {
        match FromStr::from_str(p) {
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
/// TODO: Switch to `SplitWhitespace` when it is release (1.1)
fn parse_face<'a, T: Iterator<Item = &'a str>>(face_str: T, faces: &mut Vec<Face>, pos_sz: usize, tex_sz: usize,
		      norm_sz: usize) -> bool {
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
        _ => faces.push(Face::Polygon(indices)),
    }
    true
}

/// Add a vertex to a mesh by either re-using an existing index (eg. it's in the index_map)
/// or appending the position, texcoord and normal as appropriate and creating a new vertex
fn add_vertex(mesh: &mut Mesh, index_map: &mut HashMap<VertexIndices, u32>, vert: &VertexIndices,
              pos: &Vec<f32>, texcoord: &Vec<f32>, normal: &Vec<f32>) {
    match index_map.get(vert) {
        Some(&i) => mesh.indices.push(i),
        None => {
			let v = vert.v as usize;
            // Add the vertex to the mesh
            mesh.positions.push(pos[v * 3]);
            mesh.positions.push(pos[v * 3 + 1]);
            mesh.positions.push(pos[v * 3 + 2]);
            if !texcoord.is_empty() && vert.vt > -1 {
				let vt = vert.vt as usize;
                mesh.texcoords.push(texcoord[vt * 2]);
                mesh.texcoords.push(texcoord[vt * 2 + 1]);
            }
            if !normal.is_empty() && vert.vn > -1 {
				let vn = vert.vn as usize;
                mesh.normals.push(normal[vn * 3]);
                mesh.normals.push(normal[vn * 3 + 1]);
                mesh.normals.push(normal[vn * 3 + 2]);
            }
            let next = index_map.len() as u32;
            mesh.indices.push(next);
            index_map.insert(*vert, next);
        }
    }
}

/// Export a list of faces to a mesh and return it, converting quads to tris
fn export_faces(pos: &Vec<f32>, texcoord: &Vec<f32>, normal: &Vec<f32>, faces: &Vec<Face>,
				mat_id: Option<usize>) -> Mesh {
    let mut index_map = HashMap::new();
    let mut mesh = Mesh::empty();
    mesh.material_id = mat_id;
    // TODO: When drain becomes stable we should use that, since we clear `faces` later anyway
    for f in faces {
        // Optimized paths for Triangles and Quads, Polygon handles the general case of an unknown
        // length triangle fan
        match f {
            &Face::Triangle(ref a, ref b, ref c) => {
                add_vertex(&mut mesh, &mut index_map, a, pos, texcoord, normal);
                add_vertex(&mut mesh, &mut index_map, b, pos, texcoord, normal);
                add_vertex(&mut mesh, &mut index_map, c, pos, texcoord, normal);
            },
            &Face::Quad(ref a, ref b, ref c, ref d) => {
                add_vertex(&mut mesh, &mut index_map, a, pos, texcoord, normal);
                add_vertex(&mut mesh, &mut index_map, b, pos, texcoord, normal);
                add_vertex(&mut mesh, &mut index_map, c, pos, texcoord, normal);

                add_vertex(&mut mesh, &mut index_map, a, pos, texcoord, normal);
                add_vertex(&mut mesh, &mut index_map, c, pos, texcoord, normal);
                add_vertex(&mut mesh, &mut index_map, d, pos, texcoord, normal);
            },
            &Face::Polygon(ref indices) => {
                let a = &indices[0];
                let mut c = &indices[1];
                // TODO: Can we do something nicer with iterators here?
                for i in 2..indices.len() - 1 {
                    let b = c;
                    c = &indices[i];
                    add_vertex(&mut mesh, &mut index_map, a, pos, texcoord, normal);
                    add_vertex(&mut mesh, &mut index_map, b, pos, texcoord, normal);
                    add_vertex(&mut mesh, &mut index_map, c, pos, texcoord, normal);
                }
            },
        }
    }
    mesh
}

/// Load the various objects specified in the OBJ file and any associated MTL file
/// Returns a pair of Vecs containing the loaded models and materials from the file.
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
/// Returns a pair with a Vec holding all loaded materials and a HashMap containing a mapping of
/// material names to indices in the Vec.
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
    let mut name = "unnamed_object".to_string();
    // material used by the current object being parsed
    let mut mat_id = None;
    for line in reader.lines() {
        let (line, mut words) = match line {
            /// TODO: Switch to `split_whitespace` when it is release (1.1)
            Ok(ref line) => (&line[..], line[..].split(char::is_whitespace).filter(|s| !s.is_empty())),
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
                if !parse_face(words, &mut tmp_faces, tmp_pos.len() / 3, tmp_texcoord.len() / 2,
							   tmp_normal.len() / 3) {
					return Err(LoadError::FaceParseError);
                }
            },
            // Just treating object and group tags identically. Should there be different behavior
            // for them?
            Some("o") | Some("g") => {
                // If we were already parsing an object then a new object name
                // signals the end of the current one, so push it onto our list of objects
                if !name.is_empty() && !tmp_faces.is_empty() {
                    models.push(Model::new(export_faces(&tmp_pos, &tmp_texcoord, &tmp_normal,
						                                &tmp_faces, mat_id), name));
                    tmp_faces.clear();
                }
                name = line[1..].trim().to_string();
                if name.is_empty() {
                    return Err(LoadError::InvalidObjectName);
                }
            },
            Some("mtllib") => {
                if let Some(mtllib) = words.next() {
                    let mat_file = match base_path {
                        Some(bp) => bp.join(mtllib),
                        None => Path::new(mtllib).to_path_buf(),
                    };
                    match load_mtl(mat_file.as_path()) {
                        Ok((mats, map)) => {
							// Merge the loaded material lib with any currently loaded ones, offsetting
							// the indices of the appended materials by our current length
							let mat_offset = materials.len();
							// TODO: Switch to append when it's stabilized, some more optimized functionality
							// is coming for this. Alternatively, should I have a material loader that takes
							// the map and such to append to?
                            materials = materials + &mats[..];
							for m in map {
								mat_map.insert(m.0, m.1 + mat_offset);
							}
                        },
                        Err(e) => return Err(e),
                    }
                } else {
                    return Err(LoadError::MaterialParseError);
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
                } else {
                    return Err(LoadError::MaterialParseError);
                }
            },
            // Just ignore unrecognized characters
            Some(_) => {},
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
        let (line, mut words) = match line {
            /// TODO: Switch to `split_whitespace` when it is release (1.1)
            Ok(ref line) => (&line[..], line[..].split(char::is_whitespace).filter(|s| !s.is_empty())),
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
                cur_mat.name = line[6..].trim().to_string();
                if cur_mat.name.is_empty() {
                    return Err(LoadError::InvalidObjectName);
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
                    match FromStr::from_str(p) {
                        Ok(x) => cur_mat.shininess = x,
                        Err(_) => return Err(LoadError::MaterialParseError),
                    }
                } else {
                    return Err(LoadError::MaterialParseError);
                }
            },
            Some("d") => {
                if let Some(p) = words.next() {
                    match FromStr::from_str(p) {
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

/// Print out all loaded properties of some models and associated materials
pub fn print_model_info(models: &Vec<Model>, materials: &Vec<Material>) {
    println!("# of models: {}", models.len());
    println!("# of materials: {}", materials.len());
    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;
        println!("model[{}].name = \'{}\'", i, m.name);
        println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);

        println!("Size of model[{}].indices: {}", i, mesh.indices.len());
        for f in 0..mesh.indices.len() / 3 {
            println!("    idx[{}] = {}, {}, {}.", f, mesh.indices[3 * f], mesh.indices[3 * f + 1],
				mesh.indices[3 * f + 2]);
        }

        println!("model[{}].vertices: {}", i, mesh.positions.len() / 3);
        println!("model[{}].normals: {}", i, mesh.normals.len() / 3);
        println!("model[{}].texcoords: {}", i, mesh.texcoords.len() / 2);
        assert!(mesh.positions.len() % 3 == 0);
        assert!(mesh.normals.len() % 3 == 0);
        assert!(mesh.texcoords.len() % 2 == 0);
        for v in 0..mesh.positions.len() / 3 {
            println!("    v[{}] = ({}, {}, {})", v, mesh.positions[3 * v], mesh.positions[3 * v + 1],
				mesh.positions[3 * v + 2]);
			if !mesh.normals.is_empty() {
				println!("    vn[{}] = ({}, {}, {})", v, mesh.normals[3 * v], mesh.normals[3 * v + 1],
					mesh.normals[3 * v + 2]);
			}
			if !mesh.texcoords.is_empty() {
				println!("    vt[{}] = ({}, {})", v, mesh.texcoords[2 * v], mesh.texcoords[2 * v + 1]);
			}
        }
    }
	print_material_info(materials);
}

/// Print out all loaded properties of some materials
pub fn print_material_info(materials: &Vec<Material>) {
    for (i, m) in materials.iter().enumerate() {
        println!("material[{}].name = \'{}\'", i, m.name);
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

#[cfg(all(test, feature = "unstable"))]
mod benches {
    use test::Bencher;
    use std::path::Path;
    use super::load_obj;

    #[bench]
    fn bench_cornell(b: &mut Bencher) {
        let path = Path::new("cornell_box.obj");
        b.iter(|| {
            let m = load_obj(&path);
            assert!(m.is_ok());
            m.is_ok()
        });
    }
}

