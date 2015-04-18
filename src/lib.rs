//! Tiny OBJ loader, inspired by Syoyo's excellent [tinyobjloader](https://github.com/syoyo/tinyobjloader)

#![allow(dead_code)]

use std::io::prelude::*;
use std::io::BufReader;
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
    pub faces: Vec<u32>,
}

impl Mesh {
    /// Create a new mesh specifying the geometry for the mesh
    pub fn new(pos: Vec<f32>, norm: Vec<f32>, tex: Vec<f32>, faces: Vec<u32>) -> Mesh {
        Mesh { positions: pos, normals: norm, texcoords: tex, faces: faces }
    }
    /// Create a new empty mesh
    pub fn empty() -> Mesh {
        Mesh { positions: Vec::new(), normals: Vec::new(), texcoords: Vec::new(), faces: Vec::new() }
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
    InvalidObjectName,
    GenericFailure,
}

/// LoadResult is a result containing all the models loaded from the file or any
/// error that occured while loading
pub type LoadResult = Result<Vec<Model>, LoadError>;

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
    /// Returns None if the face string is invalid
    pub fn parse(face_str: &str) -> Option<VertexIndices> {
        println!("Parsing face string {}", face_str);
        let mut indices = [0; 3];
        for i in face_str.split('/').enumerate() {
            println!("Index: {}, element index: {}", i.1, i.0);
            // Catch case of v//vn where we'll find an empty string in one of our splits
            // since there are no texcoords for the mesh
            if !i.1.is_empty() {
                match usize::from_str(i.1) {
                    Ok(x) => indices[i.0] = x - 1,
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
        match FromStr::from_str(p) {
            Ok(x) => vals.push(x),
            Err(_) => return false,
        }
    }
    // Require that we found the desired number of floats
    sz + n == vals.len()
}

/// Parse vertex indices for a face and append it to the list of faces passed
/// returns false if an error occured parsing the face
fn parse_face(face_str: Split<char>, faces: &mut Vec<Face>) -> bool {
    let mut indices = Vec::new();
    for f in face_str {
        match VertexIndices::parse(f) {
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
        Some(&i) => mesh.faces.push(i),
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
            mesh.faces.push(next);
            index_map.insert(*vert, next);
        }
    }
}

/// Export a list of faces to a mesh and return it, converting quads to tris
fn export_faces(pos: &Vec<f32>, texcoord: &Vec<f32>, normal: &Vec<f32>, faces: &Vec<Face>) -> Mesh {
    let mut index_map = HashMap::new();
    let mut mesh = Mesh::empty();
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
pub fn load_obj(file_name: &str) -> LoadResult {
    println!("Loading file {}", file_name);
    let file = match File::open(file_name) {
        Ok(f) => f,
        Err(e) => {
            println!("tobj::load_obj - failed to open {} due to {}", file_name, e);
            return Err(LoadError::OpenFileFailed);
        },
    };
    let mut reader = BufReader::new(file);
    load_obj_buf(&mut reader)
}

/// Load the various meshes in an OBJ buffer
pub fn load_obj_buf<B: BufRead>(reader: &mut B) -> LoadResult {
    let mut models = Vec::new();
    let mut tmp_pos = Vec::new();
    let mut tmp_texcoord = Vec::new();
    let mut tmp_normal = Vec::new();
    let mut tmp_faces: Vec<Face> = Vec::new();
    // name of the current object being parsed
    let mut name = String::new();
    // Next index for a new face we might find
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
            Some("#") => { println!("Skipping comment"); continue; },
            Some("v") => {
                println!("Will parse vertex {}", line);
                if !parse_floatn(words, &mut tmp_pos, 3) {
                    println!("Failed to parse 'v'");
                    return Err(LoadError::PositionParseError);
                }
            },
            Some("vt") => {
                println!("Will parse texcoord {}", line);
                if !parse_floatn(words, &mut tmp_texcoord, 2) {
                    return Err(LoadError::TexcoordParseError);
                }
            },
            Some("vn") => {
                println!("Will parse normal {}", line);
                if !parse_floatn(words, &mut tmp_normal, 3) {
                    return Err(LoadError::NormalParseError);
                }
            },
            Some("f") => {
                println!("Will parse face {}", line);
                if !parse_face(words, &mut tmp_faces) {
                    return Err(LoadError::FaceParseError);
                }
            },
            Some("o") => {
                // If we were already parsing an object then a new object name
                // signals the end of the current one, so push it onto our list of objects
                if !name.is_empty() && !tmp_faces.is_empty() {
                    models.push(Model::new(export_faces(&tmp_pos, &tmp_texcoord, &tmp_normal, &tmp_faces), name));
                    println!("Finished parsing {:?}", models[models.len() - 1]);
                    tmp_faces.clear();
                }
                match words.next() {
                    Some(n) => name = n.to_string(),
                    None => return Err(LoadError::InvalidObjectName),
                }
                println!("Beginning to parse new object: {}", name);
            },
            Some("g") => { println!("Will parse group {}", line); },
            Some("mtllib") => { println!("Will parse material lib {}", line); },
            Some("usemtl") => { println!("Will parse usemtl {}", line); },
            None => { println!("Skipping empty line"); continue; },
            // TODO: throw error on unrecognized character. Currently with split we get a newline
            // and incorrectly through so this is off temporarily. Blocked until `words` becomes
            // stable
            Some(c) => { println!("Unrecognized character: {}", c); /*return Err(LoadError::UnrecognizedCharacter) */ },
        }
    }
    // For the last object in the file we won't encounter another object name to tell us when it's
    // done, so if we're parsing an object push the last one on the list as well
    if !name.is_empty() {
        models.push(Model::new(export_faces(&tmp_pos, &tmp_texcoord, &tmp_normal, &tmp_faces), name));
    }
    for m in &models {
        println!("Parsed Model: {:?}", m);
    }
    Ok(models)
}

#[test]
fn test_tri(){
    assert!(load_obj("triangle.obj").is_ok());
}

#[test]
fn test_quad(){
    assert!(load_obj("quad.obj").is_ok());
}

