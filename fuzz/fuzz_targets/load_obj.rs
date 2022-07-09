#![no_main]
use std::{io::Cursor, os::unix::prelude::OsStrExt, path::Path};

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: (&[u8], [&[u8]; 4], tobj::LoadOptions)| {
    let (obj, mtls, options) = data;

    let mtl_loader = move |path: &Path| {
        let index: Option<usize> = path
            .as_os_str()
            .as_bytes()
            .first()
            .map(|c| (*c & 0b11).into());

        if let Some(index) = index {
            // load an mtl from one of the mtl bufs
            let mut cursor = Cursor::new(mtls[index]);
            tobj::load_mtl_buf(&mut cursor)
        } else {
            // path was empty, just give a default Ok
            Ok((Vec::new(), ahash::AHashMap::new()))
        }
    };

    let mut cursor = Cursor::new(obj);
    let _ = tobj::load_obj_buf(&mut cursor, &options, mtl_loader);
});
