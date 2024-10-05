#![allow(dead_code)]

use std::{
    fs::File,
    io::BufReader,
    path::Path,
};

use color_eyre::eyre::Error;
use serde::Deserialize;

// see: https://github.com/astronexus/HYG-Database/tree/main/hyg

/*
"id","hip","hd"  ,"hr","gl","bf","proper","ra","dec","dist","pmra","pmdec","rv","mag","absmag","spect","ci","x","y","z","vx","vy","vz","rarad","decrad","pmrarad","pmdecrad","bayer","flam","con","comp","comp_primary","base","lum","var","var_min","var_max"
   0,     ,      ,    ,  "",  "",     Sol,0.0,0.0,0.0,0.0,0.0,0.0,-26.7,4.85,G2V,0.656,0.000005,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,"","","",1,0,"",1.0,"",,
   1,    1,224700,    ,  "",  "",      "",0.00006,1.089009,219.7802,-5.2,-1.88,0.0,9.1,2.39,F5,0.482,219.740502,0.003449,4.177065,0.00000004,-0.00000554,-0.000002,0.0000156934097753,0.01900678824815125,-0.0000000252103114,-0.000000009114497,"","",Psc,1,1,"",9.638290236239703,"",,
*/

#[derive(Clone, Debug, Deserialize)]
pub struct Record {
    pub id: u32,
    pub hip: Option<u32>,
    pub hd: Option<u32>,
    pub hr: Option<u32>,
    pub gl: Option<String>,
    pub bf: Option<String>,
    pub proper: Option<String>,
    pub ra: f32,
    pub dec: f32,
    pub dist: f32,
    pub pmra: f32,
    pub pmdec: f32,
    pub rv: Option<f32>,
    pub mag: f32,
    pub absmag: f32,
    pub spect: Option<String>,
    pub ci: Option<String>,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub rarad: f32,
    pub decrad: f32,
    pub pmrarad: f32,
    pub pmdecrad: f32,
    pub bayer: String,
    pub flam: Option<u32>,
    pub con: String,
    pub comp: Option<u32>,
    pub comp_primary: Option<u32>,
    pub base: Option<String>,
    pub lum: f32,
    pub var: Option<String>,
    pub var_min: Option<f32>,
    pub var_max: Option<f32>,
}

pub struct Reader {
    reader: csv::DeserializeRecordsIntoIter<BufReader<File>, Record>,
}

impl Reader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let reader = csv::Reader::from_reader(reader);
        let reader = reader.into_deserialize();
        Ok(Self { reader })
    }

    pub fn read_record(&mut self) -> Result<Option<Record>, Error> {
        self.reader.next().transpose().map_err(Into::into)
    }
}

impl Iterator for Reader {
    type Item = Result<Record, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_record().transpose()
    }
}
