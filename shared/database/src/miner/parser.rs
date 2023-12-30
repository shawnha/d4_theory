use std::collections::{BTreeMap, BTreeSet};
use serde::{Serialize, Deserialize};

pub enum Error {
    /// Failed to open data file
    OpenFile(std::io::Error),

    /// Failed to parse data file
    ParseFile(std::io::Error),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::OpenFile(e) =>
                write!(f, "Failed to open data file: {}", e),
            Error::ParseFile(e) =>
                write!(f, "Failed to parse data file: {}", e),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StlFile {
    pub fields: BTreeMap<String, String>,
}

pub struct AffFile {
    pub values: Vec<String>,
}

pub struct SklFile {

}

pub struct Parser {
    pub stl_files: BTreeSet<StlFile>,
    pub aff_files: BTreeSet<AffFile>,
    pub skl_files: BTreeSet<SklFile>,
}

impl Parser {
    fn parse(&self, &str) -> Result<Self> {

    }
}
