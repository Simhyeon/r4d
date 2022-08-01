//! Packaging is about creating a executable script and executing it.

use crate::{Processor, RadResult};
use flate2::bufread::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;

/// Statically packaged (or scriptable) script
#[derive(Serialize, Deserialize)]
pub(crate) struct StaticScript {
    pub header: Vec<u8>,
    pub body: Vec<u8>,
}

impl StaticScript {
    /// Create a new instance
    pub fn new(processor: &Processor, body: Vec<u8>) -> RadResult<Self> {
        let header = processor.serialize_rules()?;
        Ok(Self { header, body })
    }

    /// Unpack binary input into a static script
    pub fn unpack(source: Vec<u8>) -> RadResult<Self> {
        let mut decompressed = Vec::new();
        let mut decoder = GzDecoder::new(&source[..]);
        decoder.read_to_end(&mut decompressed).unwrap();
        let object = bincode::deserialize::<Self>(&decompressed[..]).unwrap();
        Ok(object)
    }

    /// Package static script into binary bytes
    pub fn package(&mut self, file: Option<&std::path::Path>) -> RadResult<Vec<u8>> {
        let serialized = bincode::serialize(&self).unwrap();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&serialized).unwrap();
        let something = encoder.finish().unwrap();
        if let Some(file) = file {
            std::fs::write(file, something).unwrap();
            Ok(vec![])
        } else {
            Ok(something)
        }
    }
}
