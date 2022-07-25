use crate::{Processor, RadResult};
use flate2::bufread::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;

#[derive(Serialize, Deserialize)]
pub(crate) struct StaticScript {
    pub header: Vec<u8>,
    pub body: Vec<u8>,
}

impl StaticScript {
    pub fn new(processor: &Processor, body: Vec<u8>) -> RadResult<Self> {
        let header = processor.serialize_rules()?;
        Ok(Self { header, body })
    }

    pub fn unpack(source: Vec<u8>) -> RadResult<Self> {
        let mut decompressed = Vec::new();
        let mut decoder = GzDecoder::new(&source[..]);
        decoder.read_to_end(&mut decompressed).unwrap();
        let object = bincode::deserialize::<Self>(&decompressed[..]).unwrap();
        Ok(object)
    }

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
