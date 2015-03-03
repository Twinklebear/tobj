//! Tiny OBJ loader, inspired by Syoyo's excellent [tinyobjloader](https://github.com/syoyo/tinyobjloader)

#![allow(dead_code)]

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::cmp::{Eq, Ord, PartialOrd, Ordering};

/// A mesh for some model containing its triangle geometry
/// This object could be a single polygon group or object within a file
/// that defines multiple groups/objects or be the only mesh within the file
/// if it only contains a single mesh
#[derive(Debug)]
pub struct Mesh {
    position: Vec<f32>,
    normals: Option<Vec<f32>>,
    texcoords: Option<Vec<f32>>,
    faces: Vec<u32>,
}

/// A named model within the file
/// This could be a group or object or the single model exported by the file
#[derive(Debug)]
pub struct Model {
    mesh: Mesh,
    name: String,
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
#[derive(Eq, PartialEq, PartialOrd, Ord)]
struct VertexIndices {
    v: u32,
    vt: u32,
    vn: u32,
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
            Some("f") => { println!("Will parse face {}", line); },
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

