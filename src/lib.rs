//! Tiny OBJ loader, inspired by Syoyo's excellent [tinyobjloader](https://github.com/syoyo/tinyobjloader)

#![allow(dead_code)]

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::cmp::{Eq, Ord, PartialOrd, Ordering};
use std::collections::BTreeMap;
use std::str::{FromStr, Words};

/// A mesh for some model containing its triangle geometry
/// This object could be a single polygon group or object within a file
/// that defines multiple groups/objects or be the only mesh within the file
/// if it only contains a single mesh
#[derive(Debug)]
pub struct Mesh {
    positions: Vec<f32>,
    normals: Option<Vec<f32>>,
    texcoords: Option<Vec<f32>>,
    faces: Vec<u32>,
}

impl Mesh {
    /// Create a new mesh specifying the geometry for the mesh
    pub fn new(pos: Vec<f32>, norm: Option<Vec<f32>>, tex: Option<Vec<f32>>, faces: Vec<u32>) -> Mesh {
        Mesh { positions: pos, normals: norm, texcoords: tex, faces: faces }
    }
}

/// A named model within the file
/// This could be a group or object or the single model exported by the file
#[derive(Debug)]
pub struct Model {
    mesh: Mesh,
    name: String,
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
    GenericFailure,
}

/// LoadResult is a result containing all the models loaded from the file or any
/// error that occured while loading
pub type LoadResult = Result<Vec<Model>, LoadError>;

/// Struct storing indices corresponding to the vertex
/// Some vertices may not have texcoords or normals, 0 is used to indicate this
/// as OBJ indices begin at 1
/// TODO: Should use std::btree_map::BTreeMap to store the mapping of VertexIndices -> index
#[derive(Eq, PartialEq, PartialOrd, Ord, Debug)]
struct VertexIndices {
    v: u32,
    vt: u32,
    vn: u32,
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
            match FromStr::from_str(i.1) {
                Ok(x) => indices[i.0] = x,
                Err(_) => return None,
            }
        }
        Some(VertexIndices { v: indices[0], vt: indices[1], vn: indices[2] })
    }
}

/// Parse vertex indices for a face and write them to the index buffer
/// will push new positions, normals and texcoords and create a new index if the vertex hasn't been created
/// returns false if parsing a face failed
fn parse_face(face_str: Words, pos: &mut Vec<f32>, normals: &mut Vec<f32>, texcoords: &mut Vec<f32>,
              indices: &mut Vec<u32>, vertex_map: &mut BTreeMap<VertexIndices, u32>) -> bool {
    true
}

/// Load the various meshes in an OBJ file
pub fn load_obj(file_name: &str) -> LoadResult {
    println!("Loading file {}", file_name);
    let mut file = match File::open(file_name) {
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
    /*
    let mut tmp_pos = Vec::new();
    let mut tmp_normals = Vec::new();
    let mut tmp_texcoords = Vec::new();
    */
    for line in reader.lines() {
        // We just need the line for debugging for a bit
        let (line, mut words) = match line {
            Ok(ref line) => (&line[..], line[..].words()),
            Err(e) => {
                println!("tobj::load_obj - failed to read line due to {}", e);
                return Err(LoadError::ReadError);
            },
        };
        match words.next() {
            Some("#") => { println!("Skipping comment"); continue; },
            Some("v") => { println!("Will parse vertex {}", line); },
            Some("vt") => { println!("Will parse texcoord {}", line); },
            Some("vn") => { println!("Will parse normal {}", line); },
            Some("f") => {
                println!("Will parse face {}", line);
                let face = match words.next() {
                    Some(f) => VertexIndices::parse(f),
                    None => None,
                };
                match face {
                    Some(f) => println!("Valid face, {:?}", f),
                    None => println!("Invalid face string"),
                }
            },
            Some("o") => { println!("Will parse object {}", line); },
            Some("g") => { println!("Will parse group {}", line); },
            Some("mtllib") => { println!("Will parse material lib {}", line); },
            Some("usemtl") => { println!("Will parse usemtl {}", line); },
            None => { println!("Skipping empty line"); continue; },
            Some(_) => { println!("Unrecognized character"); return Err(LoadError::UnrecognizedCharacter) },
        }
    }
    Ok(models)
}

#[test]
fn test_basic(){
    assert!(load_obj("triangle.obj").is_ok());
}

