//! # Tiny OBJ Loader
//!
//! A tiny OBJ loader, inspired by Syoyo's excellent [`tinyobjloader`](https://github.com/syoyo/tinyobjloader).
//! Aims to be a simple and lightweight option for loading `OBJ` files.
//!
//! Just returns two `Vec`s containing loaded models and materials.
//!
//! ## Triangulation
//!
//! Meshes can be triangulated on the fly or left as-is.
//!
//! Only polygons that are trivially convertible to triangle fans are supported.
//! Arbitrary polygons may not behave as expected. The best solution would be to
//! convert your mesh to solely consist of triangles in your modeling software.
//!
//! ## Optional – Normals & Texture Coordinates
//!
//! It is assumed that all meshes will at least have positions, but normals and
//! texture coordinates are optional.
//!
//! If no normals or texture coordinates are found then the corresponding
//! [`Vec`](Mesh::normals)s for the [`Mesh`] will be empty.
//!
//! ## Flat Data
//!
//! Values are stored packed as [`f32`]s (or [`f64`]s with the use_f64 feature)
//! in flat `Vec`s.
//!
//! For example, the `positions` member of a `Mesh` will contain `[x, y, z, x,
//! y, z, ...]` which you can then use however you like.
//!
//! ## Indices
//!
//! Indices are also loaded and may re-use vertices already existing in the
//! mesh, this data is stored in the [`indices`](Mesh::indices) member.
//!
//! When a `Mesh` contains *per vertex per face* normals or texture coordinates,
//! positions can be duplicated to be *per vertex per face* too via the
//! [`single_index`](LoadOptions::single_index) flag. This potentially changes
//! the topology (faces may become disconnected even though their vertices still
//! share a position in space).
//!
//! By default separate indices for normals and texture coordinates are created.
//! This also guarantees that the topology of the `Mesh` does *not* change when
//! either of the latter are specified *per vertex per face*.
//!
//! ## Materials
//!
//! Standard `MTL` attributes are supported too. Any unrecognized parameters
//! will be stored in a `HashMap` containing the key-value pairs of the
//! unrecognized parameter and its value.
//!
//! ## Example
//!
//! In this simple example we load the classic Cornell Box model that only
//! defines positions and print out its attributes. This example is a slightly
//! trimmed down version of `print_model_info` and `print_material_info`
//! combined together, see them for a version that also prints out normals and
//! texture coordinates if the model has them.
//!
//! The [`LoadOptions`] used are typical for the case when the mesh is going to
//! be sent to a realtime rendering context (game engine, GPU etc.).
//!
//! ```
//! use tobj;
//!
//! let cornell_box = tobj::load_obj("obj/cornell_box.obj", &tobj::GPU_LOAD_OPTIONS);
//! assert!(cornell_box.is_ok());
//!
//! let (models, materials) = cornell_box.expect("Failed to load OBJ file");
//!
//! // Materials might report a separate loading error if the MTL file wasn't found.
//! // If you don't need the materials, you can generate a default here and use that
//! // instead.
//! let materials = materials.expect("Failed to load MTL file");
//!
//! println!("# of models: {}", models.len());
//! println!("# of materials: {}", materials.len());
//!
//! for (i, m) in models.iter().enumerate() {
//!     let mesh = &m.mesh;
//!
//!     println!("model[{}].name = \'{}\'", i, m.name);
//!     println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);
//!
//!     println!(
//!         "Size of model[{}].face_arities: {}",
//!         i,
//!         mesh.face_arities.len()
//!     );
//!
//!     let mut next_face = 0;
//!     for f in 0..mesh.face_arities.len() {
//!         let end = next_face + mesh.face_arities[f] as usize;
//!         let face_indices: Vec<_> = mesh.indices[next_face..end].iter().collect();
//!         println!("    face[{}] = {:?}", f, face_indices);
//!         next_face = end;
//!     }
//!
//!     // Normals and texture coordinates are also loaded, but not printed in this example
//!     println!("model[{}].vertices: {}", i, mesh.positions.len() / 3);
//!
//!     assert!(mesh.positions.len() % 3 == 0);
//!     for v in 0..mesh.positions.len() / 3 {
//!         println!(
//!             "    v[{}] = ({}, {}, {})",
//!             v,
//!             mesh.positions[3 * v],
//!             mesh.positions[3 * v + 1],
//!             mesh.positions[3 * v + 2]
//!         );
//!     }
//! }
//!
//! for (i, m) in materials.iter().enumerate() {
//!     println!("material[{}].name = \'{}\'", i, m.name);
//!     if let Some(ambient) = m.ambient {
//!         println!(
//!             "    material.Ka = ({}, {}, {})",
//!             ambient[0], ambient[1], ambient[2]
//!         );
//!     }
//!     if let Some(diffuse) = m.diffuse {
//!         println!(
//!             "    material.Kd = ({}, {}, {})",
//!             diffuse[0], diffuse[1], diffuse[2]
//!         );
//!     }
//!     if let Some(specular) = m.specular {
//!         println!(
//!             "    material.Ks = ({}, {}, {})",
//!             specular[0], specular[1], specular[2]
//!         );
//!     }
//!     if let Some(shininess) = m.shininess {
//!         println!("    material.Ns = {}", shininess);
//!     }
//!     if let Some(dissolve) = m.dissolve {
//!         println!("    material.d = {}", dissolve);
//!     }
//!     if let Some(ambient_texture) = &m.ambient_texture {
//!         println!("    material.map_Ka = {}", ambient_texture);
//!     }
//!     if let Some(diffuse_texture) = &m.diffuse_texture {
//!         println!("    material.map_Kd = {}", diffuse_texture);
//!     }
//!     if let Some(specular_texture) = &m.specular_texture {
//!         println!("    material.map_Ks = {}", specular_texture);
//!     }
//!     if let Some(shininess_texture) = &m.shininess_texture {
//!         println!("    material.map_Ns = {}", shininess_texture);
//!     }
//!     if let Some(normal_texture) = &m.normal_texture {
//!         println!("    material.map_Bump = {}", normal_texture);
//!     }
//!     if let Some(dissolve_texture) = &m.dissolve_texture {
//!         println!("    material.map_d = {}", dissolve_texture);
//!     }
//!
//!     for (k, v) in &m.unknown_param {
//!         println!("    material.{} = {}", k, v);
//!     }
//! }
//! ```
//!
//! ## Rendering Examples
//!
//! For an example of integration with [glium](https://github.com/tomaka/glium)
//! to make a simple OBJ viewer, check out [`tobj viewer`](https://github.com/Twinklebear/tobj_viewer).
//! Some more sample images can be found in [this gallery](http://imgur.com/a/xsg6v).
//!
//! The Rungholt model shown below is reasonably large (6.7M triangles, 12.3M
//! vertices) and is loaded in ~7.47s using a peak of ~1.1GB of memory on a
//! Windows 10 machine with an i7-4790k and 16GB of 1600Mhz DDR3 RAM with tobj
//! 0.1.1 on rustc 1.6.0. The model can be found on [Morgan McGuire's](http://graphics.cs.williams.edu/data/meshes.xml)
//! meshes page and was originally built by kescha. Future work will focus on
//! improving performance and memory usage.
//!
//! <img src="http://i.imgur.com/wImyNG4.png" alt="Rungholt"
//!     style="display:block; max-width:100%; height:auto">
//!
//! For an example of integration within a ray tracer, check out tray\_rust's
//! [mesh module](https://github.com/Twinklebear/tray_rust/blob/master/src/geometry/mesh.rs).
//! The Stanford Buddha and Dragon from the
//! [Stanford 3D Scanning Repository](http://graphics.stanford.edu/data/3Dscanrep/)
//! both load quite quickly. The Rust logo model was made by [Nylithius on BlenderArtists](http://blenderartists.org/forum/showthread.php?362836-Rust-language-3D-logo).
//! The materials used are from the [MERL BRDF Database](http://www.merl.com/brdf/).
//!
//! <img src="http://i.imgur.com/E1ylrZW.png" alt="Rust logo with friends"
//!     style="display:block; max-width:100%; height:auto">
//!
//! ## Features
//!
//! * [`ahash`](https://crates.io/crates/ahash) – On by default. Use [`AHashMap`](https://docs.rs/ahash/latest/ahash/struct.AHashMap.html)
//!   for hashing when reading files and merging vertices. To disable and use
//!   the slower [`HashMap`](std::collections::HashMap) instead, unset default
//! features in `Cargo.toml`:
//!
//!   ```toml
//!   [dependencies.tobj]
//!   default-features = false
//!   ```
//!
//! * [`merging`](LoadOptions::merge_identical_points) – Adds support for
//!   merging identical vertex positions on disconnected faces during import.
//!
//!   **Warning:** this feature uses *const generics* and thus requires at
//!   least a `beta` toolchain to build.
//!
//! * [`reordering`](LoadOptions::reorder_data) – Adds support for reordering
//!   the normal- and texture coordinate indices.
//!
//! * [`async`](load_obj_buf_async) – Adds support for async loading of obj
//!   files from a buffer, with an async material loader. Useful in environments
//!   that do not support blocking IO (e.g. WebAssembly).
//!
//! * ['use_f64'] - Uses double-precision (f64) instead of single-precision
//!   (f32) floating point types
#![cfg_attr(feature = "merging", allow(incomplete_features))]
#![cfg_attr(feature = "merging", feature(generic_const_exprs))]

#[cfg(test)]
mod tests;

use std::{
    error::Error,
    fmt,
    fs::File,
    io::{prelude::*, BufReader},
    path::Path,
    str::{FromStr, SplitWhitespace},
};

#[cfg(feature = "use_f64")]
type Float = f64;

#[cfg(not(feature = "use_f64"))]
type Float = f32;

#[cfg(feature = "async")]
use std::future::Future;

#[cfg(feature = "merging")]
use std::mem::size_of;

#[cfg(feature = "ahash")]
type HashMap<K, V> = ahash::AHashMap<K, V>;

#[cfg(not(feature = "ahash"))]
type HashMap<K, V> = std::collections::HashMap<K, V>;

/// Typical [`LoadOptions`] for using meshes in a GPU/relatime context.
///
/// Faces are *triangulated*, a *single index* is generated and *degenerate
/// faces* (points & lines) are *discarded*.
pub const GPU_LOAD_OPTIONS: LoadOptions = LoadOptions {
    #[cfg(feature = "merging")]
    merge_identical_points: false,
    #[cfg(feature = "reordering")]
    reorder_data: false,
    single_index: true,
    triangulate: true,
    ignore_points: true,
    ignore_lines: true,
};

/// Typical [`LoadOptions`] for using meshes with an offline rendeder.
///
/// Faces are *kept as they are* (e.g. n-gons) and *normal and texture
/// coordinate data is reordered* so only a single index is needed.
/// Topology remains unchanged except for *degenerate faces* (points & lines)
/// which are *discarded*.
pub const OFFLINE_RENDERING_LOAD_OPTIONS: LoadOptions = LoadOptions {
    #[cfg(feature = "merging")]
    merge_identical_points: true,
    #[cfg(feature = "reordering")]
    reorder_data: true,
    single_index: false,
    triangulate: false,
    ignore_points: true,
    ignore_lines: true,
};

/// A mesh made up of triangles loaded from some `OBJ` file.
///
/// It is assumed that all meshes will at least have positions, but normals and
/// texture coordinates are optional. If no normals or texture coordinates where
/// found then the corresponding `Vec`s in the `Mesh` will be empty. Values are
/// stored packed as [`f32`]s  (or [`f64`]s with the use_f64 feature) in  flat
/// `Vec`s.
///
/// For examples the `positions` member of a loaded mesh will contain `[x, y, z,
/// x, y, z, ...]` which you can then use however you like. Indices are also
/// loaded and may re-use vertices already existing in the mesh. This data is
/// stored in the `indices` member.
///
/// # Example:
/// Load the Cornell box and get the attributes of the first vertex. It's
/// assumed all meshes will have positions (required), but normals and texture
/// coordinates are optional, in which case the corresponding `Vec` will be
/// empty.
///
/// ```
/// let cornell_box = tobj::load_obj("obj/cornell_box.obj", &tobj::GPU_LOAD_OPTIONS);
/// assert!(cornell_box.is_ok());
///
/// let (models, materials) = cornell_box.unwrap();
///
/// let mesh = &models[0].mesh;
/// let i = mesh.indices[0] as usize;
///
/// // pos = [x, y, z]
/// let pos = [
///     mesh.positions[i * 3],
///     mesh.positions[i * 3 + 1],
///     mesh.positions[i * 3 + 2],
/// ];
///
/// if !mesh.normals.is_empty() {
///     // normal = [x, y, z]
///     let normal = [
///         mesh.normals[i * 3],
///         mesh.normals[i * 3 + 1],
///         mesh.normals[i * 3 + 2],
///     ];
/// }
///
/// if !mesh.texcoords.is_empty() {
///     // texcoord = [u, v];
///     let texcoord = [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]];
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    /// Flattened 3 component floating point vectors, storing positions of
    /// vertices in the mesh.
    pub positions: Vec<Float>,
    /// Flattened 3 component floating point vectors, storing the color
    /// associated with the vertices in the mesh.
    ///
    /// Most meshes do not have vertex colors. If no vertex colors are specified
    /// this will be empty.
    pub vertex_color: Vec<Float>,
    /// Flattened 3 component floating point vectors, storing normals of
    /// vertices in the mesh.
    ///
    /// Not all meshes have normals. If no normals are specified this will
    /// be empty.
    pub normals: Vec<Float>,
    /// Flattened 2 component floating point vectors, storing texture
    /// coordinates of vertices in the mesh.
    ///
    /// Not all meshes have texture coordinates. If no texture coordinates are
    /// specified this will be empty.
    pub texcoords: Vec<Float>,
    /// Indices for vertices of each face. If loaded with
    /// [`triangulate`](LoadOptions::triangulate) set to `true` each face in the
    /// mesh is a triangle.
    ///
    /// Otherwise [`face_arities`](Mesh::face_arities) indicates how many
    /// indices are used by each face.
    ///
    /// When [`single_index`](LoadOptions::single_index) is set to `true`,
    /// these indices are for *all* of the data in the mesh. Positions,
    /// normals and texture coordinaes.
    /// Otherwise normals and texture coordinates have *their own* indices,
    /// each.
    pub indices: Vec<u32>,
    /// The number of vertices (arity) of each face. *Empty* if loaded with
    /// `triangulate` set to `true` or if the mesh constists *only* of
    /// triangles.
    ///
    /// The offset for the starting index of a face can be found by iterating
    /// through the `face_arities` until reaching the desired face, accumulating
    /// the number of vertices used so far.
    pub face_arities: Vec<u32>,
    /// The indices for vertex colors. Only present when the
    /// [`merging`](LoadOptions::merge_identical_points) feature is enabled, and
    /// empty unless the corresponding load option is set to `true`.
    #[cfg(feature = "merging")]
    pub vertex_color_indices: Vec<u32>,
    /// The indices for texture coordinates. Can be omitted by setting
    /// `single_index` to `true`.
    pub texcoord_indices: Vec<u32>,
    /// The indices for normals. Can be omitted by setting `single_index` to
    /// `true`.
    pub normal_indices: Vec<u32>,
    /// Optional material id associated with this mesh. The material id indexes
    /// into the Vec of Materials loaded from the associated `MTL` file
    pub material_id: Option<usize>,
}

/// Options for processing the mesh during loading.
///
/// Passed to [`load_obj()`], [`load_obj_buf()`] and [`load_obj_buf_async()`].
///
/// By default, all of these are `false`. With those settings, the data you get
/// represents the original data in the input file/buffer as closely as
/// possible.
///
/// Use the [init struct pattern](https://xaeroxe.github.io/init-struct-pattern/) to set individual options:
/// ```ignore
/// LoadOptions {
///     single_index: true,
///     ..Default::default()
/// }
/// ```
///
/// There are convenience `const`s for the most common cases:
///
/// * [`GPU_LOAD_OPTIONS`] – if you display meshes on the GPU/in realtime.
///
/// * [`OFFLINE_RENDERING_LOAD_OPTIONS`] – if you're rendering meshes with e.g.
///   an offline path tracer or the like.
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Default, Clone, Copy)]
pub struct LoadOptions {
    /// Merge identical positions.
    ///
    /// This is usually what you want if you intend to use the mesh in an
    /// *offline rendering* context or to do further processing with
    /// *topological operators*.
    ///
    /// * This flag is *mutually exclusive* with
    ///   [`single_index`](LoadOptions::single_index) and will lead to a
    ///   [`InvalidLoadOptionConfig`](LoadError::InvalidLoadOptionConfig) error
    ///   if both are set to `true`.
    ///
    /// * If adjacent faces share vertices that have separate `indices` but the
    ///   same position in 3D they will be merged into a single vertex and the
    ///   resp. `indices` changed.
    ///
    /// * Topolgy may change as a result (faces may become *connected* in the
    ///   index).
    #[cfg(feature = "merging")]
    pub merge_identical_points: bool,
    /// Normal & texture coordinates will be reordered to allow omitting their
    /// indices.
    ///
    /// * This flag is *mutually exclusive* with
    ///   [`single_index`](LoadOptions::single_index) and will lead to an
    ///   [`InvalidLoadOptionConfig`](LoadError::InvalidLoadOptionConfig) error
    ///   if both are set to `true`.
    ///
    /// * The resulting [`Mesh`]'s `normal_indices` and/or `texcoord_indices`
    ///   will be empty.
    ///
    /// * *Per-vertex* normals and/or texture_coordinates will be reordered to
    ///   match the `Mesh`'s `indices`.
    ///
    /// * *Per-vertex-per-face*  normals and/or texture coordinates indices will
    ///   be `[0, 1, 2, ..., n]`. I.e.:
    ///
    ///   ```ignore
    ///   // If normals where specified per-vertex-per-face:
    ///   assert!(mesh.indices.len() == mesh.normals.len() / 3);
    ///
    ///   for index in 0..mesh.indices.len() {
    ///       println!("Normal n is {}, {}, {}",
    ///           mesh.normals[index * 3 + 0],
    ///           mesh.normals[index * 3 + 1],
    ///           mesh.normals[index * 3 + 2]
    ///       );
    ///   }
    ///   ```
    #[cfg(feature = "reordering")]
    pub reorder_data: bool,
    /// Create a single index.
    ///
    /// This is usually what you want if you are loading the mesh to display in
    /// a *realtime* (*GPU*) context.
    ///
    /// * This flag is *mutually exclusive* with both
    ///   [`merge_identical_points`](LoadOptions::merge_identical_points) and
    ///   [`reorder_data`](LoadOptions::reorder_data) resp. and will lead to a
    ///   [`InvalidLoadOptionConfig`](LoadError::InvalidLoadOptionConfig) error
    ///   if both it and either of the two other are set to `true`.
    ///
    /// * Vertices may get duplicated to match the granularity
    ///   (*per-vertex-per-face*) of normals and/or texture coordinates.
    ///
    /// * Topolgy may change as a result (faces may become *disconnected* in the
    ///   index).
    ///
    /// * The resulting [`Mesh`]'s [`normal_indices`](Mesh::normal_indices) and
    ///   [`texcoord_indices`](Mesh::texcoord_indices) will be empty.
    pub single_index: bool,
    /// Triangulate all faces.
    ///
    /// * Points (one point) and lines (two points) are blown up to zero area
    ///   triangles via point duplication. Except if `ignore_points` or
    ///   `ignore_lines` is/are set to `true`, resp.
    ///
    /// * The resulting `Mesh`'s [`face_arities`](Mesh::face_arities) will be
    ///   empty as all faces are guranteed to have arity `3`.
    ///
    /// * Only polygons that are trivially convertible to triangle fans are
    ///   supported. Arbitrary polygons may not behave as expected. The best
    ///   solution would be to convert your mesh to solely consist of triangles
    ///   in your modeling software.
    pub triangulate: bool,
    /// Ignore faces containing only a single vertex (points).
    ///
    /// This is usually what you want if you do *not* intend to make special use
    /// of the point data (e.g. as particles etc.).
    ///
    /// Polygon meshes that contain faces with one vertex only usually do so
    /// because of bad topology.
    pub ignore_points: bool,
    /// Ignore faces containing only two vertices (lines).
    ///
    /// This is usually what you want if you do *not* intend to make special use
    /// of the line data (e.g. as wires/ropes etc.).
    ///
    /// Polygon meshes that contains faces with two vertices only usually do so
    /// because of bad topology.
    pub ignore_lines: bool,
}

impl LoadOptions {
    /// Checks if the given `LoadOptions` do not contain mutually exclusive flag
    /// settings.
    ///
    /// This is called by [`load_obj()`]/[`load_obj_buf()`] in any case. This
    /// method is only exposed for scenarios where you want to do this check
    /// yourself.
    pub fn is_valid(&self) -> bool {
        // A = single_index, B = merge_identical_points, C = reorder_data
        // (A ∧ ¬B) ∨ (A ∧ ¬C) -> A ∧ ¬(B ∨ C)
        #[allow(unused_mut)]
        let mut other_flags = false;

        #[cfg(feature = "merging")]
        {
            other_flags = other_flags || self.merge_identical_points;
        }
        #[cfg(feature = "reordering")]
        {
            other_flags = other_flags || self.reorder_data;
        }

        (self.single_index != other_flags) || (!self.single_index && !other_flags)
    }
}

/// A named model within the file.
///
/// Associates some mesh with a name that was specified with an `o` or `g`
/// keyword in the `OBJ` file.
#[derive(Clone, Debug)]
pub struct Model {
    /// [`Mesh`] used by the model containing its geometry.
    pub mesh: Mesh,
    /// Name assigned to this `Mesh`.
    pub name: String,
}

impl Model {
    /// Create a new model, associating a name with a [`Mesh`].
    pub fn new(mesh: Mesh, name: String) -> Model {
        Model { mesh, name }
    }
}

/// A material that may be referenced by one or more [`Mesh`]es.
///
/// Standard `MTL` attributes are supported. Any unrecognized parameters will be
/// stored as key-value pairs in the `unknown_param`
/// [`HashMap`](std::collections::HashMap), which maps the unknown parameter to
/// the value set for it.
///
/// No path is pre-pended to the texture file names specified in the `MTL` file.
#[derive(Clone, Debug, Default)]
pub struct Material {
    /// Material name as specified in the `MTL` file.
    pub name: String,
    /// Ambient color of the material.
    pub ambient: Option<[Float; 3]>,
    /// Diffuse color of the material.
    pub diffuse: Option<[Float; 3]>,
    /// Specular color of the material.
    pub specular: Option<[Float; 3]>,
    /// Material shininess attribute. Also called `glossiness`.
    pub shininess: Option<Float>,
    /// Dissolve attribute is the alpha term for the material. Referred to as
    /// dissolve since that's what the `MTL` file format docs refer to it as.
    pub dissolve: Option<Float>,
    /// Optical density also known as index of refraction. Called
    /// `optical_density` in the `MTL` specc. Takes on a value between 0.001
    /// and 10.0. 1.0 means light does not bend as it passes through
    /// the object.
    pub optical_density: Option<Float>,
    /// Name of the ambient texture file for the material.
    pub ambient_texture: Option<String>,
    /// Name of the diffuse texture file for the material.
    pub diffuse_texture: Option<String>,
    /// Name of the specular texture file for the material.
    pub specular_texture: Option<String>,
    /// Name of the normal map texture file for the material.
    pub normal_texture: Option<String>,
    /// Name of the shininess map texture file for the material.
    pub shininess_texture: Option<String>,
    /// Name of the alpha/opacity map texture file for the material.
    ///
    /// Referred to as `dissolve` to match the `MTL` file format specification.
    pub dissolve_texture: Option<String>,
    /// The illumnination model to use for this material. The different
    /// illumination models are specified in the [`MTL` spec](http://paulbourke.net/dataformats/mtl/).
    pub illumination_model: Option<u8>,
    /// Key value pairs of any unrecognized parameters encountered while parsing
    /// the material.
    pub unknown_param: HashMap<String, String>,
}

/// Possible errors that may occur while loading `OBJ` and `MTL` files.
#[derive(Debug, Clone, Copy, PartialEq)]
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
    InvalidPolygon,
    FaceVertexOutOfBounds,
    FaceTexCoordOutOfBounds,
    FaceNormalOutOfBounds,
    FaceColorOutOfBounds,
    InvalidLoadOptionConfig,
    GenericFailure,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let msg = match *self {
            LoadError::OpenFileFailed => "open file failed",
            LoadError::ReadError => "read error",
            LoadError::UnrecognizedCharacter => "unrecognized character",
            LoadError::PositionParseError => "position parse error",
            LoadError::NormalParseError => "normal parse error",
            LoadError::TexcoordParseError => "texcoord parse error",
            LoadError::FaceParseError => "face parse error",
            LoadError::MaterialParseError => "material parse error",
            LoadError::InvalidObjectName => "invalid object name",
            LoadError::InvalidPolygon => "invalid polygon",
            LoadError::FaceVertexOutOfBounds => "face vertex index out of bounds",
            LoadError::FaceTexCoordOutOfBounds => "face texcoord index out of bounds",
            LoadError::FaceNormalOutOfBounds => "face normal index out of bounds",
            LoadError::FaceColorOutOfBounds => "face vertex color index out of bounds",
            LoadError::InvalidLoadOptionConfig => "mutually exclusive load options",
            LoadError::GenericFailure => "generic failure",
        };

        f.write_str(msg)
    }
}

impl Error for LoadError {}

/// A [`Result`] containing all the models loaded from the file and any
/// materials from referenced material libraries. Or an error that occured while
/// loading.
pub type LoadResult = Result<(Vec<Model>, Result<Vec<Material>, LoadError>), LoadError>;

/// A [`Result`] containing all the materials loaded from the file and a map of
/// `MTL` name to index. Or an error that occured while loading.
pub type MTLLoadResult = Result<(Vec<Material>, HashMap<String, usize>), LoadError>;

/// Struct storing indices corresponding to the vertex.
///
/// Some vertices may not have texture coordinates or normals, 0 is used to
/// indicate this as OBJ indices begin at 1
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug, Copy, Clone)]
struct VertexIndices {
    pub v: usize,
    pub vt: usize,
    pub vn: usize,
}

static MISSING_INDEX: usize = usize::MAX;

impl VertexIndices {
    /// Parse the vertex indices from the face string.
    ///
    /// Valid face strings are those that are valid for a Wavefront `OBJ` file.
    ///
    /// Also handles relative face indices (negative values) which is why
    /// passing the number of positions, texcoords and normals is required.
    ///
    /// Returns `None` if the face string is invalid.
    fn parse(
        face_str: &str,
        pos_sz: usize,
        tex_sz: usize,
        norm_sz: usize,
    ) -> Option<VertexIndices> {
        let mut indices = [MISSING_INDEX; 3];
        for i in face_str.split('/').enumerate() {
            // Catch case of v//vn where we'll find an empty string in one of our splits
            // since there are no texcoords for the mesh.
            if !i.1.is_empty() {
                match isize::from_str(i.1) {
                    Ok(x) => {
                        // Handle relative indices
                        *indices.get_mut(i.0)? = if x < 0 {
                            match i.0 {
                                0 => (pos_sz as isize + x) as _,
                                1 => (tex_sz as isize + x) as _,
                                2 => (norm_sz as isize + x) as _,
                                _ => return None, // Invalid number of elements for a face
                            }
                        } else {
                            (x - 1) as _
                        };
                    }
                    Err(_) => return None,
                }
            }
        }
        Some(VertexIndices {
            v: indices[0],
            vt: indices[1],
            vn: indices[2],
        })
    }
}

/// Enum representing a face, storing indices for the face vertices.
#[derive(Debug)]
enum Face {
    Point(VertexIndices),
    Line(VertexIndices, VertexIndices),
    Triangle(VertexIndices, VertexIndices, VertexIndices),
    Quad(VertexIndices, VertexIndices, VertexIndices, VertexIndices),
    Polygon(Vec<VertexIndices>),
}

/// Parse the float information from the words. Words is an iterator over the
/// float strings. Returns `false` if parsing failed.
fn parse_floatn(val_str: &mut SplitWhitespace, vals: &mut Vec<Float>, n: usize) -> bool {
    let sz = vals.len();
    for p in val_str.take(n) {
        match FromStr::from_str(p) {
            Ok(x) => vals.push(x),
            Err(_) => return false,
        }
    }
    // Require that we found the desired number of floats.
    sz + n == vals.len()
}

/// Parse the a string into a float3 array, returns an error if parsing failed
fn parse_float3(val_str: SplitWhitespace) -> Result<[Float; 3], LoadError> {
    let arr: [Float; 3] = val_str
        .take(3)
        .map(FromStr::from_str)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| LoadError::MaterialParseError)?
        .try_into()
        .unwrap();
    Ok(arr)
}

/// Parse the a string into a float value, returns an error if parsing failed
fn parse_float(val_str: Option<&str>) -> Result<Float, LoadError> {
    val_str
        .map(FromStr::from_str)
        .map_or(Err(LoadError::MaterialParseError), |v| {
            v.map_err(|_| LoadError::MaterialParseError)
        })
}

/// Parse vertex indices for a face and append it to the list of faces passed.
///
/// Also handles relative face indices (negative values) which is why passing
/// the number of positions, texcoords and normals is required.
///
/// Returns `false` if an error occured parsing the face.
fn parse_face(
    face_str: SplitWhitespace,
    faces: &mut Vec<Face>,
    pos_sz: usize,
    tex_sz: usize,
    norm_sz: usize,
) -> bool {
    let mut indices = Vec::new();
    for f in face_str {
        match VertexIndices::parse(f, pos_sz, tex_sz, norm_sz) {
            Some(v) => indices.push(v),
            None => return false,
        }
    }
    // Check what kind face we read and push it on
    match indices.len() {
        1 => faces.push(Face::Point(indices[0])),
        2 => faces.push(Face::Line(indices[0], indices[1])),
        3 => faces.push(Face::Triangle(indices[0], indices[1], indices[2])),
        4 => faces.push(Face::Quad(indices[0], indices[1], indices[2], indices[3])),
        _ => faces.push(Face::Polygon(indices)),
    }
    true
}

/// Add a vertex to a mesh by either re-using an existing index (e.g. it's in
/// the `index_map`) or appending the position, texcoord and normal as
/// appropriate and creating a new vertex.
fn add_vertex(
    mesh: &mut Mesh,
    index_map: &mut HashMap<VertexIndices, u32>,
    vert: &VertexIndices,
    pos: &[Float],
    v_color: &[Float],
    texcoord: &[Float],
    normal: &[Float],
) -> Result<(), LoadError> {
    match index_map.get(vert) {
        Some(&i) => mesh.indices.push(i),
        None => {
            let v = vert.v;
            if v.saturating_mul(3).saturating_add(2) >= pos.len() {
                return Err(LoadError::FaceVertexOutOfBounds);
            }
            // Add the vertex to the mesh
            mesh.positions.push(pos[v * 3]);
            mesh.positions.push(pos[v * 3 + 1]);
            mesh.positions.push(pos[v * 3 + 2]);
            if !texcoord.is_empty() && vert.vt != MISSING_INDEX {
                let vt = vert.vt;
                if vt * 2 + 1 >= texcoord.len() {
                    return Err(LoadError::FaceTexCoordOutOfBounds);
                }
                mesh.texcoords.push(texcoord[vt * 2]);
                mesh.texcoords.push(texcoord[vt * 2 + 1]);
            }
            if !normal.is_empty() && vert.vn != MISSING_INDEX {
                let vn = vert.vn;
                if vn * 3 + 2 >= normal.len() {
                    return Err(LoadError::FaceNormalOutOfBounds);
                }
                mesh.normals.push(normal[vn * 3]);
                mesh.normals.push(normal[vn * 3 + 1]);
                mesh.normals.push(normal[vn * 3 + 2]);
            }
            if !v_color.is_empty() {
                if v * 3 + 2 >= v_color.len() {
                    return Err(LoadError::FaceColorOutOfBounds);
                }
                mesh.vertex_color.push(v_color[v * 3]);
                mesh.vertex_color.push(v_color[v * 3 + 1]);
                mesh.vertex_color.push(v_color[v * 3 + 2]);
            }
            let next = index_map.len() as u32;
            mesh.indices.push(next);
            index_map.insert(*vert, next);
        }
    }
    Ok(())
}

/// Export a list of faces to a mesh and return it, optionally converting quads
/// to tris.
fn export_faces(
    pos: &[Float],
    v_color: &[Float],
    texcoord: &[Float],
    normal: &[Float],
    faces: &[Face],
    mat_id: Option<usize>,
    load_options: &LoadOptions,
) -> Result<Mesh, LoadError> {
    let mut index_map = HashMap::new();
    let mut mesh = Mesh {
        material_id: mat_id,
        ..Default::default()
    };
    let mut is_all_triangles = true;

    for f in faces {
        // Optimized paths for Triangles and Quads, Polygon handles the general case of
        // an unknown length triangle fan.
        match *f {
            Face::Point(ref a) => {
                if !load_options.ignore_points {
                    add_vertex(&mut mesh, &mut index_map, a, pos, v_color, texcoord, normal)?;
                    if load_options.triangulate {
                        add_vertex(&mut mesh, &mut index_map, a, pos, v_color, texcoord, normal)?;
                        add_vertex(&mut mesh, &mut index_map, a, pos, v_color, texcoord, normal)?;
                    } else {
                        is_all_triangles = false;
                        mesh.face_arities.push(1);
                    }
                }
            }
            Face::Line(ref a, ref b) => {
                if !load_options.ignore_lines {
                    add_vertex(&mut mesh, &mut index_map, a, pos, v_color, texcoord, normal)?;
                    add_vertex(&mut mesh, &mut index_map, b, pos, v_color, texcoord, normal)?;
                    if load_options.triangulate {
                        add_vertex(&mut mesh, &mut index_map, b, pos, v_color, texcoord, normal)?;
                    } else {
                        is_all_triangles = false;
                        mesh.face_arities.push(2);
                    }
                }
            }
            Face::Triangle(ref a, ref b, ref c) => {
                add_vertex(&mut mesh, &mut index_map, a, pos, v_color, texcoord, normal)?;
                add_vertex(&mut mesh, &mut index_map, b, pos, v_color, texcoord, normal)?;
                add_vertex(&mut mesh, &mut index_map, c, pos, v_color, texcoord, normal)?;
                if !load_options.triangulate {
                    mesh.face_arities.push(3);
                }
            }
            Face::Quad(ref a, ref b, ref c, ref d) => {
                add_vertex(&mut mesh, &mut index_map, a, pos, v_color, texcoord, normal)?;
                add_vertex(&mut mesh, &mut index_map, b, pos, v_color, texcoord, normal)?;
                add_vertex(&mut mesh, &mut index_map, c, pos, v_color, texcoord, normal)?;

                if load_options.triangulate {
                    add_vertex(&mut mesh, &mut index_map, a, pos, v_color, texcoord, normal)?;
                    add_vertex(&mut mesh, &mut index_map, c, pos, v_color, texcoord, normal)?;
                    add_vertex(&mut mesh, &mut index_map, d, pos, v_color, texcoord, normal)?;
                } else {
                    add_vertex(&mut mesh, &mut index_map, d, pos, v_color, texcoord, normal)?;
                    is_all_triangles = false;
                    mesh.face_arities.push(4);
                }
            }
            Face::Polygon(ref indices) => {
                if load_options.triangulate {
                    let a = indices.first().ok_or(LoadError::InvalidPolygon)?;
                    let mut b = indices.get(1).ok_or(LoadError::InvalidPolygon)?;
                    for c in indices.iter().skip(2) {
                        add_vertex(&mut mesh, &mut index_map, a, pos, v_color, texcoord, normal)?;
                        add_vertex(&mut mesh, &mut index_map, b, pos, v_color, texcoord, normal)?;
                        add_vertex(&mut mesh, &mut index_map, c, pos, v_color, texcoord, normal)?;
                        b = c;
                    }
                } else {
                    for i in indices.iter() {
                        add_vertex(&mut mesh, &mut index_map, i, pos, v_color, texcoord, normal)?;
                    }
                    is_all_triangles = false;
                    mesh.face_arities.push(indices.len() as u32);
                }
            }
        }
    }

    if is_all_triangles {
        // This is a triangle-only mesh.
        mesh.face_arities = Vec::new();
    }

    Ok(mesh)
}

/// Add a vertex to a mesh by either re-using an existing index (e.g. it's in
/// the `index_map`) or appending the position, texcoord and normal as
/// appropriate and creating a new vertex.
#[allow(clippy::too_many_arguments)]
#[inline]
fn add_vertex_multi_index(
    mesh: &mut Mesh,
    index_map: &mut HashMap<usize, u32>,
    normal_index_map: &mut HashMap<usize, u32>,
    texcoord_index_map: &mut HashMap<usize, u32>,
    vert: &VertexIndices,
    pos: &[Float],
    v_color: &[Float],
    texcoord: &[Float],
    normal: &[Float],
) -> Result<(), LoadError> {
    match index_map.get(&vert.v) {
        Some(&i) => mesh.indices.push(i),
        None => {
            let vertex = vert.v;

            if vertex.saturating_mul(3).saturating_add(2) >= pos.len() {
                return Err(LoadError::FaceVertexOutOfBounds);
            }

            // Add the vertex to the mesh.
            mesh.positions.push(pos[vertex * 3]);
            mesh.positions.push(pos[vertex * 3 + 1]);
            mesh.positions.push(pos[vertex * 3 + 2]);

            let next = index_map.len() as u32;
            mesh.indices.push(next);
            index_map.insert(vertex, next);

            // Also add vertex colors to the mesh if present.
            if !v_color.is_empty() {
                let vertex = vert.v;

                if vertex * 3 + 2 >= v_color.len() {
                    return Err(LoadError::FaceColorOutOfBounds);
                }

                mesh.vertex_color.push(v_color[vertex * 3]);
                mesh.vertex_color.push(v_color[vertex * 3 + 1]);
                mesh.vertex_color.push(v_color[vertex * 3 + 2]);
            }
        }
    }

    if !texcoord.is_empty() {
        let texcoord_indices = &mut mesh.texcoord_indices;

        if MISSING_INDEX == vert.vt {
            // Special case: the very first vertex of the mesh has no index.
            if texcoord_indices.is_empty() {
                // We have no choice, simply reference the first vertex.
                mesh.texcoords.push(texcoord[0]);
                mesh.texcoords.push(texcoord[1]);

                texcoord_indices.push(0);
                texcoord_index_map.insert(0, 0);
            // We use the previous index. Not great a fallback but less prone to
            // cause issues. FIXME: we should probably check if the
            // data is per-vertex-per-face and if so calculate the
            // average from adjacent face vertices.
            } else {
                texcoord_indices.push(*texcoord_indices.last().unwrap());
            }
        } else {
            match texcoord_index_map.get(&vert.vt) {
                Some(&index) => mesh.texcoord_indices.push(index as _),
                None => {
                    let vt = vert.vt;

                    if vt * 2 + 1 >= texcoord.len() {
                        return Err(LoadError::FaceTexCoordOutOfBounds);
                    }

                    mesh.texcoords.push(texcoord[vt * 2]);
                    mesh.texcoords.push(texcoord[vt * 2 + 1]);

                    let next = texcoord_index_map.len() as u32;
                    mesh.texcoord_indices.push(next);
                    texcoord_index_map.insert(vt, next);
                }
            }
        }
    }

    if !normal.is_empty() {
        let normal_indices = &mut mesh.normal_indices;
        // The index is sparse – we need to make up a value.
        if MISSING_INDEX == vert.vn {
            // Special case: the very first vertex of the mesh has no index.
            if normal_indices.is_empty() {
                // We have no choice, simply reference the first vertex.
                mesh.normals.push(normal[0]);
                mesh.normals.push(normal[1]);
                mesh.normals.push(normal[2]);

                normal_indices.push(0);
                normal_index_map.insert(0, 0);
            // We use the previous index. Not great a fallback but less prone to
            // cause issues. FIXME: we should probably check if the
            // data is per-vertex-per-face and if so calculate the
            // average from adjacent face vertices.
            } else {
                normal_indices.push(*normal_indices.last().unwrap());
            }
        } else {
            match normal_index_map.get(&vert.vn) {
                Some(&index) => normal_indices.push(index as _),
                None => {
                    let vn = vert.vn;

                    if vn * 3 + 2 >= normal.len() {
                        return Err(LoadError::FaceNormalOutOfBounds);
                    }

                    mesh.normals.push(normal[vn * 3]);
                    mesh.normals.push(normal[vn * 3 + 1]);
                    mesh.normals.push(normal[vn * 3 + 2]);

                    let next = normal_index_map.len() as u32;
                    normal_indices.push(next);
                    normal_index_map.insert(vn, next);
                }
            }
        }
    }

    Ok(())
}

/// Export a list of faces to a mesh and return it, optionally converting quads
/// to tris.
fn export_faces_multi_index(
    pos: &[Float],
    v_color: &[Float],
    texcoord: &[Float],
    normal: &[Float],
    faces: &[Face],
    mat_id: Option<usize>,
    load_options: &LoadOptions,
) -> Result<Mesh, LoadError> {
    let mut index_map = HashMap::new();
    let mut normal_index_map = HashMap::new();
    let mut texcoord_index_map = HashMap::new();

    let mut mesh = Mesh {
        material_id: mat_id,
        ..Default::default()
    };

    let mut is_all_triangles = true;

    for f in faces {
        // Optimized paths for Triangles and Quads, Polygon handles the general case of
        // an unknown length triangle fan
        match *f {
            Face::Point(ref a) => {
                if !load_options.ignore_points {
                    add_vertex_multi_index(
                        &mut mesh,
                        &mut index_map,
                        &mut normal_index_map,
                        &mut texcoord_index_map,
                        a,
                        pos,
                        v_color,
                        texcoord,
                        normal,
                    )?;
                    if load_options.triangulate {
                        add_vertex_multi_index(
                            &mut mesh,
                            &mut index_map,
                            &mut normal_index_map,
                            &mut texcoord_index_map,
                            a,
                            pos,
                            v_color,
                            texcoord,
                            normal,
                        )?;
                        add_vertex_multi_index(
                            &mut mesh,
                            &mut index_map,
                            &mut normal_index_map,
                            &mut texcoord_index_map,
                            a,
                            pos,
                            v_color,
                            texcoord,
                            normal,
                        )?;
                    } else {
                        is_all_triangles = false;
                        mesh.face_arities.push(1);
                    }
                }
            }
            Face::Line(ref a, ref b) => {
                if !load_options.ignore_lines {
                    add_vertex_multi_index(
                        &mut mesh,
                        &mut index_map,
                        &mut normal_index_map,
                        &mut texcoord_index_map,
                        a,
                        pos,
                        v_color,
                        texcoord,
                        normal,
                    )?;
                    add_vertex_multi_index(
                        &mut mesh,
                        &mut index_map,
                        &mut normal_index_map,
                        &mut texcoord_index_map,
                        b,
                        pos,
                        v_color,
                        texcoord,
                        normal,
                    )?;
                    if load_options.triangulate {
                        add_vertex_multi_index(
                            &mut mesh,
                            &mut index_map,
                            &mut normal_index_map,
                            &mut texcoord_index_map,
                            b,
                            pos,
                            v_color,
                            texcoord,
                            normal,
                        )?;
                    } else {
                        is_all_triangles = false;
                        mesh.face_arities.push(2);
                    }
                }
            }
            Face::Triangle(ref a, ref b, ref c) => {
                add_vertex_multi_index(
                    &mut mesh,
                    &mut index_map,
                    &mut normal_index_map,
                    &mut texcoord_index_map,
                    a,
                    pos,
                    v_color,
                    texcoord,
                    normal,
                )?;
                add_vertex_multi_index(
                    &mut mesh,
                    &mut index_map,
                    &mut normal_index_map,
                    &mut texcoord_index_map,
                    b,
                    pos,
                    v_color,
                    texcoord,
                    normal,
                )?;
                add_vertex_multi_index(
                    &mut mesh,
                    &mut index_map,
                    &mut normal_index_map,
                    &mut texcoord_index_map,
                    c,
                    pos,
                    v_color,
                    texcoord,
                    normal,
                )?;
                if !load_options.triangulate {
                    mesh.face_arities.push(3);
                }
            }
            Face::Quad(ref a, ref b, ref c, ref d) => {
                add_vertex_multi_index(
                    &mut mesh,
                    &mut index_map,
                    &mut normal_index_map,
                    &mut texcoord_index_map,
                    a,
                    pos,
                    v_color,
                    texcoord,
                    normal,
                )?;
                add_vertex_multi_index(
                    &mut mesh,
                    &mut index_map,
                    &mut normal_index_map,
                    &mut texcoord_index_map,
                    b,
                    pos,
                    v_color,
                    texcoord,
                    normal,
                )?;
                add_vertex_multi_index(
                    &mut mesh,
                    &mut index_map,
                    &mut normal_index_map,
                    &mut texcoord_index_map,
                    c,
                    pos,
                    v_color,
                    texcoord,
                    normal,
                )?;

                if load_options.triangulate {
                    add_vertex_multi_index(
                        &mut mesh,
                        &mut index_map,
                        &mut normal_index_map,
                        &mut texcoord_index_map,
                        a,
                        pos,
                        v_color,
                        texcoord,
                        normal,
                    )?;
                    add_vertex_multi_index(
                        &mut mesh,
                        &mut index_map,
                        &mut normal_index_map,
                        &mut texcoord_index_map,
                        c,
                        pos,
                        v_color,
                        texcoord,
                        normal,
                    )?;
                    add_vertex_multi_index(
                        &mut mesh,
                        &mut index_map,
                        &mut normal_index_map,
                        &mut texcoord_index_map,
                        d,
                        pos,
                        v_color,
                        texcoord,
                        normal,
                    )?;
                } else {
                    add_vertex_multi_index(
                        &mut mesh,
                        &mut index_map,
                        &mut normal_index_map,
                        &mut texcoord_index_map,
                        d,
                        pos,
                        v_color,
                        texcoord,
                        normal,
                    )?;
                    is_all_triangles = false;
                    mesh.face_arities.push(4);
                }
            }
            Face::Polygon(ref indices) => {
                if load_options.triangulate {
                    let a = indices.first().ok_or(LoadError::InvalidPolygon)?;
                    let mut b = indices.get(1).ok_or(LoadError::InvalidPolygon)?;
                    for c in indices.iter().skip(2) {
                        add_vertex_multi_index(
                            &mut mesh,
                            &mut index_map,
                            &mut normal_index_map,
                            &mut texcoord_index_map,
                            a,
                            pos,
                            v_color,
                            texcoord,
                            normal,
                        )?;
                        add_vertex_multi_index(
                            &mut mesh,
                            &mut index_map,
                            &mut normal_index_map,
                            &mut texcoord_index_map,
                            b,
                            pos,
                            v_color,
                            texcoord,
                            normal,
                        )?;
                        add_vertex_multi_index(
                            &mut mesh,
                            &mut index_map,
                            &mut normal_index_map,
                            &mut texcoord_index_map,
                            c,
                            pos,
                            v_color,
                            texcoord,
                            normal,
                        )?;
                        b = c;
                    }
                } else {
                    for i in indices.iter() {
                        add_vertex_multi_index(
                            &mut mesh,
                            &mut index_map,
                            &mut normal_index_map,
                            &mut texcoord_index_map,
                            i,
                            pos,
                            v_color,
                            texcoord,
                            normal,
                        )?;
                    }
                    is_all_triangles = false;
                    mesh.face_arities.push(indices.len() as u32);
                }
            }
        }
    }

    if is_all_triangles {
        // This is a triangle-only mesh.
        mesh.face_arities = Vec::new();
    }

    #[cfg(feature = "merging")]
    if load_options.merge_identical_points {
        if !mesh.vertex_color.is_empty() {
            mesh.vertex_color_indices = mesh.indices.clone();
            merge_identical_points::<3>(&mut mesh.vertex_color, &mut mesh.vertex_color_indices);
        }
        merge_identical_points::<3>(&mut mesh.positions, &mut mesh.indices);
        merge_identical_points::<3>(&mut mesh.normals, &mut mesh.normal_indices);
        merge_identical_points::<2>(&mut mesh.texcoords, &mut mesh.texcoord_indices);
    }

    #[cfg(feature = "reordering")]
    if load_options.reorder_data {
        reorder_data(&mut mesh);
    }

    Ok(mesh)
}

#[cfg(feature = "reordering")]
#[inline]
fn reorder_data(mesh: &mut Mesh) {
    // If we have per face per vertex data for UVs ...
    if mesh.positions.len() < mesh.texcoords.len() {
        mesh.texcoords = mesh
            .texcoord_indices
            .iter()
            .flat_map(|&index| {
                let index = index as usize * 2;
                IntoIterator::into_iter([mesh.texcoords[index], mesh.texcoords[index + 1]])
            })
            .collect::<Vec<_>>();
    } else {
        assert!(mesh.texcoords.len() == mesh.positions.len());

        let mut new_texcoords = vec![0.0; mesh.positions.len()];
        mesh.texcoord_indices
            .iter()
            .zip(&mesh.indices)
            .for_each(|(&texcoord_index, &index)| {
                let texcoord_index = texcoord_index as usize * 2;
                let index = index as usize * 2;
                new_texcoords[index] = mesh.texcoords[texcoord_index];
                new_texcoords[index + 1] = mesh.texcoords[texcoord_index + 1];
            });

        mesh.texcoords = new_texcoords;
    }

    // Clear indices.
    mesh.texcoord_indices = Vec::new();

    // If we have per face per vertex data for normals ...
    if mesh.positions.len() < mesh.normals.len() {
        mesh.normals = mesh
            .normal_indices
            .iter()
            .flat_map(|&index| {
                let index = index as usize * 2;
                IntoIterator::into_iter([
                    mesh.normals[index],
                    mesh.normals[index + 1],
                    mesh.normals[index + 2],
                ])
            })
            .collect::<Vec<_>>();
    } else {
        assert!(mesh.normals.len() == mesh.positions.len());

        let mut new_normals = vec![0.0; mesh.positions.len()];
        mesh.normal_indices
            .iter()
            .zip(&mesh.indices)
            .for_each(|(&normal_index, &index)| {
                let normal_index = normal_index as usize * 3;
                let index = index as usize * 3;
                new_normals[index] = mesh.normals[normal_index];
                new_normals[index + 1] = mesh.normals[normal_index + 1];
                new_normals[index + 2] = mesh.normals[normal_index + 2];
            });

        mesh.normals = new_normals;
    }

    // Clear indices.
    mesh.normal_indices = Vec::new();
}

/// Merge identical points. A point has dimension N.
#[cfg(feature = "merging")]
#[inline]
fn merge_identical_points<const N: usize>(points: &mut Vec<Float>, indices: &mut Vec<u32>)
where
    [(); size_of::<[Float; N]>()]:,
{
    if indices.is_empty() {
        return;
    }

    let mut compressed_indices = Vec::new();
    let mut canonical_indices = HashMap::<[u8; size_of::<[Float; N]>()], u32>::new();

    let mut index = 0;
    *points = points
        .chunks(N)
        .filter_map(|position| {
            let position: &[Float; N] = &unsafe { *(position.as_ptr() as *const [Float; N]) };

            // Ugly, but floats have no Eq and no Hash.
            let bitpattern = unsafe {
                std::mem::transmute::<&[Float; N], &[u8; size_of::<[Float; N]>()]>(position)
            };

            match canonical_indices.get(bitpattern) {
                Some(&other_index) => {
                    compressed_indices.push(other_index);
                    None
                }
                None => {
                    canonical_indices.insert(*bitpattern, index);
                    compressed_indices.push(index);
                    index += 1;
                    Some(IntoIterator::into_iter(*position))
                }
            }
        })
        .flatten()
        .collect();

    indices
        .iter_mut()
        .for_each(|vertex| *vertex = compressed_indices[*vertex as usize]);
}

/// Load the various objects specified in the `OBJ` file and any associated
/// `MTL` file.
///
/// Returns a pair of `Vec`s containing the loaded models and materials from the
/// file.
///
/// # Arguments
///
/// * `load_options` – Governs on-the-fly processing of the mesh during loading.
///   See [`LoadOptions`] for more information.
pub fn load_obj<P>(file_name: P, load_options: &LoadOptions) -> LoadResult
where
    P: AsRef<Path> + fmt::Debug,
{
    let file = match File::open(file_name.as_ref()) {
        Ok(f) => f,
        Err(_e) => {
            #[cfg(feature = "log")]
            log::error!("load_obj - failed to open {:?} due to {}", file_name, _e);
            return Err(LoadError::OpenFileFailed);
        }
    };
    let mut reader = BufReader::new(file);
    load_obj_buf(&mut reader, load_options, |mat_path| {
        let full_path = if let Some(parent) = file_name.as_ref().parent() {
            parent.join(mat_path)
        } else {
            mat_path.to_owned()
        };

        self::load_mtl(full_path)
    })
}

/// Load the materials defined in a `MTL` file.
///
/// Returns a pair with a `Vec` holding all loaded materials and a `HashMap`
/// containing a mapping of material names to indices in the Vec.
pub fn load_mtl<P>(file_name: P) -> MTLLoadResult
where
    P: AsRef<Path> + fmt::Debug,
{
    let file = match File::open(file_name.as_ref()) {
        Ok(f) => f,
        Err(_e) => {
            #[cfg(feature = "log")]
            log::error!("load_mtl - failed to open {:?} due to {}", file_name, _e);
            return Err(LoadError::OpenFileFailed);
        }
    };
    let mut reader = BufReader::new(file);
    load_mtl_buf(&mut reader)
}

/// Load the various meshes in an `OBJ` buffer.
///
/// This could e.g. be a network stream, a text file already in memory etc.
///
/// # Arguments
///
/// You must pass a `material_loader` function, which will return a material
/// given a name.
///
/// A trivial material loader may just look at the file name and then call
/// `load_mtl_buf` with the in-memory MTL file source.
///
/// Alternatively it could pass an `MTL` file in memory to `load_mtl_buf` to
/// parse materials from some buffer.
///
/// * `load_options` – Governs on-the-fly processing of the mesh during loading.
///   See [`LoadOptions`] for more information.
///
/// # Example
/// The test for `load_obj_buf` includes the OBJ and MTL files as strings
/// and uses a `Cursor` to provide a `BufRead` interface on the buffer.
///
/// ```
/// use std::{env, fs::File, io::BufReader};
///
/// let dir = env::current_dir().unwrap();
/// let mut cornell_box_obj = dir.clone();
/// cornell_box_obj.push("obj/cornell_box.obj");
/// let mut cornell_box_file = BufReader::new(File::open(cornell_box_obj.as_path()).unwrap());
///
/// let mut cornell_box_mtl1 = dir.clone();
/// cornell_box_mtl1.push("obj/cornell_box.mtl");
///
/// let mut cornell_box_mtl2 = dir.clone();
/// cornell_box_mtl2.push("obj/cornell_box2.mtl");
///
/// let m = tobj::load_obj_buf(
///     &mut cornell_box_file,
///     &tobj::LoadOptions {
///         triangulate: true,
///         single_index: true,
///         ..Default::default()
///     },
///     |p| match p.file_name().unwrap().to_str().unwrap() {
///         "cornell_box.mtl" => {
///             let f = File::open(cornell_box_mtl1.as_path()).unwrap();
///             tobj::load_mtl_buf(&mut BufReader::new(f))
///         }
///         "cornell_box2.mtl" => {
///             let f = File::open(cornell_box_mtl2.as_path()).unwrap();
///             tobj::load_mtl_buf(&mut BufReader::new(f))
///         }
///         _ => unreachable!(),
///     },
/// );
/// ```
pub fn load_obj_buf<B, ML>(
    reader: &mut B,
    load_options: &LoadOptions,
    material_loader: ML,
) -> LoadResult
where
    B: BufRead,
    ML: Fn(&Path) -> MTLLoadResult,
{
    if !load_options.is_valid() {
        return Err(LoadError::InvalidLoadOptionConfig);
    }

    let mut models = Vec::new();
    let mut materials = Vec::new();
    let mut mat_map = HashMap::new();

    let mut tmp_pos = Vec::new();
    let mut tmp_v_color = Vec::new();
    let mut tmp_texcoord = Vec::new();
    let mut tmp_normal = Vec::new();
    let mut tmp_faces: Vec<Face> = Vec::new();
    // name of the current object being parsed
    let mut name = "unnamed_object".to_owned();
    // material used by the current object being parsed
    let mut mat_id = None;
    let mut mtlresult = Ok(Vec::new());

    for line in reader.lines() {
        let (line, mut words) = match line {
            Ok(ref line) => (&line[..], line[..].split_whitespace()),
            Err(_e) => {
                #[cfg(feature = "log")]
                log::error!("load_obj - failed to read line due to {}", _e);
                return Err(LoadError::ReadError);
            }
        };
        match words.next() {
            Some("#") | None => continue,
            Some("v") => {
                if !parse_floatn(&mut words, &mut tmp_pos, 3) {
                    return Err(LoadError::PositionParseError);
                }

                // Add inline vertex colors if present.
                parse_floatn(&mut words, &mut tmp_v_color, 3);
            }
            Some("vt") => {
                if !parse_floatn(&mut words, &mut tmp_texcoord, 2) {
                    return Err(LoadError::TexcoordParseError);
                }
            }
            Some("vn") => {
                if !parse_floatn(&mut words, &mut tmp_normal, 3) {
                    return Err(LoadError::NormalParseError);
                }
            }
            Some("f") | Some("l") => {
                if !parse_face(
                    words,
                    &mut tmp_faces,
                    tmp_pos.len() / 3,
                    tmp_texcoord.len() / 2,
                    tmp_normal.len() / 3,
                ) {
                    return Err(LoadError::FaceParseError);
                }
            }
            // Just treating object and group tags identically. Should there be different behavior
            // for them?
            Some("o") | Some("g") => {
                // If we were already parsing an object then a new object name
                // signals the end of the current one, so push it onto our list of objects
                if !tmp_faces.is_empty() {
                    models.push(Model::new(
                        if load_options.single_index {
                            export_faces(
                                &tmp_pos,
                                &tmp_v_color,
                                &tmp_texcoord,
                                &tmp_normal,
                                &tmp_faces,
                                mat_id,
                                load_options,
                            )?
                        } else {
                            export_faces_multi_index(
                                &tmp_pos,
                                &tmp_v_color,
                                &tmp_texcoord,
                                &tmp_normal,
                                &tmp_faces,
                                mat_id,
                                load_options,
                            )?
                        },
                        name,
                    ));
                    tmp_faces.clear();
                }
                let size = line.chars().next().unwrap().len_utf8();
                name = line[size..].trim().to_owned();
                if name.is_empty() {
                    name = "unnamed_object".to_owned();
                }
            }
            Some("mtllib") => {
                // File name can include spaces so we cannot rely on a SplitWhitespace iterator
                let mtllib = line.split_once(' ').unwrap_or_default().1.trim();
                let mat_file = Path::new(mtllib).to_path_buf();
                match material_loader(mat_file.as_path()) {
                    Ok((mut mats, map)) => {
                        // Merge the loaded material lib with any currently loaded ones,
                        // offsetting the indices of the appended
                        // materials by our current length
                        let mat_offset = materials.len();
                        materials.append(&mut mats);
                        for m in map {
                            mat_map.insert(m.0, m.1 + mat_offset);
                        }
                    }
                    Err(e) => {
                        mtlresult = Err(e);
                    }
                }
            }
            Some("usemtl") => {
                let mat_name = line.split_once(' ').unwrap_or_default().1.trim().to_owned();

                if !mat_name.is_empty() {
                    let new_mat = mat_map.get(&mat_name).cloned();
                    // As materials are returned per-model, a new material within an object
                    // has to emit a new model with the same name but different material
                    if mat_id != new_mat && !tmp_faces.is_empty() {
                        models.push(Model::new(
                            if load_options.single_index {
                                export_faces(
                                    &tmp_pos,
                                    &tmp_v_color,
                                    &tmp_texcoord,
                                    &tmp_normal,
                                    &tmp_faces,
                                    mat_id,
                                    load_options,
                                )?
                            } else {
                                export_faces_multi_index(
                                    &tmp_pos,
                                    &tmp_v_color,
                                    &tmp_texcoord,
                                    &tmp_normal,
                                    &tmp_faces,
                                    mat_id,
                                    load_options,
                                )?
                            },
                            name.clone(),
                        ));
                        tmp_faces.clear();
                    }
                    if new_mat.is_none() {
                        #[cfg(feature = "log")]
                        log::warn!("Object {} refers to unfound material: {}", name, mat_name);
                    }
                    mat_id = new_mat;
                } else {
                    return Err(LoadError::MaterialParseError);
                }
            }
            // Just ignore unrecognized characters
            Some(_) => {}
        }
    }

    // For the last object in the file we won't encounter another object name to
    // tell us when it's done, so if we're parsing an object push the last one
    // on the list as well
    models.push(Model::new(
        if load_options.single_index {
            export_faces(
                &tmp_pos,
                &tmp_v_color,
                &tmp_texcoord,
                &tmp_normal,
                &tmp_faces,
                mat_id,
                load_options,
            )?
        } else {
            export_faces_multi_index(
                &tmp_pos,
                &tmp_v_color,
                &tmp_texcoord,
                &tmp_normal,
                &tmp_faces,
                mat_id,
                load_options,
            )?
        },
        name,
    ));

    if !materials.is_empty() {
        mtlresult = Ok(materials);
    }

    Ok((models, mtlresult))
}

/// Load the various materials in a `MTL` buffer.
pub fn load_mtl_buf<B: BufRead>(reader: &mut B) -> MTLLoadResult {
    let mut materials = Vec::new();
    let mut mat_map = HashMap::new();
    // The current material being parsed
    let mut cur_mat = Material::default();
    for line in reader.lines() {
        let (line, mut words) = match line {
            Ok(ref line) => (line.trim(), line[..].split_whitespace()),
            Err(_e) => {
                #[cfg(feature = "log")]
                log::error!("load_obj - failed to read line due to {}", _e);
                return Err(LoadError::ReadError);
            }
        };

        match words.next() {
            Some("#") | None => continue,
            Some("newmtl") => {
                // If we were passing a material save it out to our vector
                if !cur_mat.name.is_empty() {
                    mat_map.insert(cur_mat.name.clone(), materials.len());
                    materials.push(cur_mat);
                }
                cur_mat = Material::default();
                cur_mat.name = line[6..].trim().to_owned();
                if cur_mat.name.is_empty() {
                    return Err(LoadError::InvalidObjectName);
                }
            }
            Some("Ka") => cur_mat.ambient = Some(parse_float3(words)?),
            Some("Kd") => cur_mat.diffuse = Some(parse_float3(words)?),
            Some("Ks") => cur_mat.specular = Some(parse_float3(words)?),
            Some("Ns") => cur_mat.shininess = Some(parse_float(words.next())?),
            Some("Ni") => cur_mat.optical_density = Some(parse_float(words.next())?),
            Some("d") => cur_mat.dissolve = Some(parse_float(words.next())?),
            Some("map_Ka") => match line.get(6..).map(str::trim) {
                Some("") | None => return Err(LoadError::MaterialParseError),
                Some(tex) => cur_mat.ambient_texture = Some(tex.to_owned()),
            },
            Some("map_Kd") => match line.get(6..).map(str::trim) {
                Some("") | None => return Err(LoadError::MaterialParseError),
                Some(tex) => cur_mat.diffuse_texture = Some(tex.to_owned()),
            },
            Some("map_Ks") => match line.get(6..).map(str::trim) {
                Some("") | None => return Err(LoadError::MaterialParseError),
                Some(tex) => cur_mat.specular_texture = Some(tex.to_owned()),
            },
            Some("map_Bump") | Some("map_bump") => match line.get(8..).map(str::trim) {
                Some("") | None => return Err(LoadError::MaterialParseError),
                Some(tex) => cur_mat.normal_texture = Some(tex.to_owned()),
            },
            Some("map_Ns") | Some("map_ns") | Some("map_NS") => {
                match line.get(6..).map(str::trim) {
                    Some("") | None => return Err(LoadError::MaterialParseError),
                    Some(tex) => cur_mat.shininess_texture = Some(tex.to_owned()),
                }
            }
            Some("bump") => match line.get(4..).map(str::trim) {
                Some("") | None => return Err(LoadError::MaterialParseError),
                Some(tex) => cur_mat.normal_texture = Some(tex.to_owned()),
            },
            Some("map_d") => match line.get(5..).map(str::trim) {
                Some("") | None => return Err(LoadError::MaterialParseError),
                Some(tex) => cur_mat.dissolve_texture = Some(tex.to_owned()),
            },
            Some("illum") => {
                if let Some(p) = words.next() {
                    match FromStr::from_str(p) {
                        Ok(x) => cur_mat.illumination_model = Some(x),
                        Err(_) => return Err(LoadError::MaterialParseError),
                    }
                } else {
                    return Err(LoadError::MaterialParseError);
                }
            }
            Some(unknown) => {
                if !unknown.is_empty() {
                    let param = line[unknown.len()..].trim().to_owned();
                    cur_mat.unknown_param.insert(unknown.to_owned(), param);
                }
            }
        }
    }

    // Finalize the last material we were parsing
    if !cur_mat.name.is_empty() {
        mat_map.insert(cur_mat.name.clone(), materials.len());
        materials.push(cur_mat);
    }

    Ok((materials, mat_map))
}

#[cfg(feature = "async")]
/// Load the various meshes in an `OBJ` buffer.
///
/// This could e.g. be a text file already in memory, a file loaded
///  asynchronously over the network etc.
///
/// # Arguments
///
/// You must pass a `material_loader` function, which will return a future
/// that loads a material given a name.
///
/// A trivial material loader may just look at the file name and then call
/// `load_mtl_buf` with the in-memory MTL file source.
///
/// Alternatively it could pass an `MTL` file in memory to `load_mtl_buf` to
/// parse materials from some buffer.
///
/// * `load_options` – Governs on-the-fly processing of the mesh during loading.
///   See [`LoadOptions`] for more information.
///
/// # Example
/// The test for `load_obj_buf` includes the OBJ and MTL files as strings
/// and uses a `Cursor` to provide a `BufRead` interface on the buffer.
///
/// ```
/// async {
///     use std::{env, fs::File, io::BufReader};
///
///     let dir = env::current_dir().unwrap();
///     let mut cornell_box_obj = dir.clone();
///     cornell_box_obj.push("obj/cornell_box.obj");
///     let mut cornell_box_file = BufReader::new(File::open(cornell_box_obj.as_path()).unwrap());
///
///     let m =
///         tobj::load_obj_buf_async(&mut cornell_box_file, &tobj::GPU_LOAD_OPTIONS, move |p| {
///             let dir_clone = dir.clone();
///             async move {
///                 let mut cornell_box_mtl1 = dir_clone.clone();
///                 cornell_box_mtl1.push("obj/cornell_box.mtl");
///
///                 let mut cornell_box_mtl2 = dir_clone.clone();
///                 cornell_box_mtl2.push("obj/cornell_box2.mtl");
///
///                 match p.as_str() {
///                     "cornell_box.mtl" => {
///                         let f = File::open(cornell_box_mtl1.as_path()).unwrap();
///                         tobj::load_mtl_buf(&mut BufReader::new(f))
///                     }
///                     "cornell_box2.mtl" => {
///                         let f = File::open(cornell_box_mtl2.as_path()).unwrap();
///                         tobj::load_mtl_buf(&mut BufReader::new(f))
///                     }
///                     _ => unreachable!(),
///                 }
///             }
///         })
///         .await;
/// };
/// ```
pub async fn load_obj_buf_async<B, ML, MLFut>(
    reader: &mut B,
    load_options: &LoadOptions,
    material_loader: ML,
) -> LoadResult
where
    B: BufRead,
    ML: Fn(String) -> MLFut,
    MLFut: Future<Output = MTLLoadResult>,
{
    if !load_options.is_valid() {
        return Err(LoadError::InvalidLoadOptionConfig);
    }

    let mut models = Vec::new();
    let mut materials = Vec::new();
    let mut mat_map = HashMap::new();

    let mut tmp_pos = Vec::new();
    let mut tmp_v_color = Vec::new();
    let mut tmp_texcoord = Vec::new();
    let mut tmp_normal = Vec::new();
    let mut tmp_faces: Vec<Face> = Vec::new();
    // name of the current object being parsed
    let mut name = "unnamed_object".to_owned();
    // material used by the current object being parsed
    let mut mat_id = None;
    let mut mtlresult = Ok(Vec::new());

    for line in reader.lines() {
        let (line, mut words) = match line {
            Ok(ref line) => (&line[..], line[..].split_whitespace()),
            Err(_e) => {
                #[cfg(feature = "log")]
                log::error!("load_obj - failed to read line due to {}", _e);
                return Err(LoadError::ReadError);
            }
        };
        match words.next() {
            Some("#") | None => continue,
            Some("v") => {
                if !parse_floatn(&mut words, &mut tmp_pos, 3) {
                    return Err(LoadError::PositionParseError);
                }

                // Add inline vertex colors if present.
                parse_floatn(&mut words, &mut tmp_v_color, 3);
            }
            Some("vt") => {
                if !parse_floatn(&mut words, &mut tmp_texcoord, 2) {
                    return Err(LoadError::TexcoordParseError);
                }
            }
            Some("vn") => {
                if !parse_floatn(&mut words, &mut tmp_normal, 3) {
                    return Err(LoadError::NormalParseError);
                }
            }
            Some("f") | Some("l") => {
                if !parse_face(
                    words,
                    &mut tmp_faces,
                    tmp_pos.len() / 3,
                    tmp_texcoord.len() / 2,
                    tmp_normal.len() / 3,
                ) {
                    return Err(LoadError::FaceParseError);
                }
            }
            // Just treating object and group tags identically. Should there be different behavior
            // for them?
            Some("o") | Some("g") => {
                // If we were already parsing an object then a new object name
                // signals the end of the current one, so push it onto our list of objects
                if !tmp_faces.is_empty() {
                    models.push(Model::new(
                        if load_options.single_index {
                            export_faces(
                                &tmp_pos,
                                &tmp_v_color,
                                &tmp_texcoord,
                                &tmp_normal,
                                &tmp_faces,
                                mat_id,
                                load_options,
                            )?
                        } else {
                            export_faces_multi_index(
                                &tmp_pos,
                                &tmp_v_color,
                                &tmp_texcoord,
                                &tmp_normal,
                                &tmp_faces,
                                mat_id,
                                load_options,
                            )?
                        },
                        name,
                    ));
                    tmp_faces.clear();
                }
                name = line[1..].trim().to_owned();
                if name.is_empty() {
                    name = "unnamed_object".to_owned();
                }
            }
            Some("mtllib") => {
                if let Some(mtllib) = words.next() {
                    let mat_file = String::from(mtllib);
                    match material_loader(mat_file).await {
                        Ok((mut mats, map)) => {
                            // Merge the loaded material lib with any currently loaded ones,
                            // offsetting the indices of the appended
                            // materials by our current length
                            let mat_offset = materials.len();
                            materials.append(&mut mats);
                            for m in map {
                                mat_map.insert(m.0, m.1 + mat_offset);
                            }
                        }
                        Err(e) => {
                            mtlresult = Err(e);
                        }
                    }
                } else {
                    return Err(LoadError::MaterialParseError);
                }
            }
            Some("usemtl") => {
                let mat_name = line[7..].trim().to_owned();
                if !mat_name.is_empty() {
                    let new_mat = mat_map.get(&mat_name).cloned();
                    // As materials are returned per-model, a new material within an object
                    // has to emit a new model with the same name but different material
                    if mat_id != new_mat && !tmp_faces.is_empty() {
                        models.push(Model::new(
                            if load_options.single_index {
                                export_faces(
                                    &tmp_pos,
                                    &tmp_v_color,
                                    &tmp_texcoord,
                                    &tmp_normal,
                                    &tmp_faces,
                                    mat_id,
                                    load_options,
                                )?
                            } else {
                                export_faces_multi_index(
                                    &tmp_pos,
                                    &tmp_v_color,
                                    &tmp_texcoord,
                                    &tmp_normal,
                                    &tmp_faces,
                                    mat_id,
                                    load_options,
                                )?
                            },
                            name.clone(),
                        ));
                        tmp_faces.clear();
                    }
                    if new_mat.is_none() {
                        #[cfg(feature = "log")]
                        log::warn!("Object {} refers to unfound material: {}", name, mat_name);
                    }
                    mat_id = new_mat;
                } else {
                    return Err(LoadError::MaterialParseError);
                }
            }
            // Just ignore unrecognized characters
            Some(_) => {}
        }
    }

    // For the last object in the file we won't encounter another object name to
    // tell us when it's done, so if we're parsing an object push the last one
    // on the list as well
    models.push(Model::new(
        if load_options.single_index {
            export_faces(
                &tmp_pos,
                &tmp_v_color,
                &tmp_texcoord,
                &tmp_normal,
                &tmp_faces,
                mat_id,
                load_options,
            )?
        } else {
            export_faces_multi_index(
                &tmp_pos,
                &tmp_v_color,
                &tmp_texcoord,
                &tmp_normal,
                &tmp_faces,
                mat_id,
                load_options,
            )?
        },
        name,
    ));

    if !materials.is_empty() {
        mtlresult = Ok(materials);
    }

    Ok((models, mtlresult))
}
