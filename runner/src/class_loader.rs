use std::fmt::Debug;
use crate::class::{Class, self};
use reader::base::{ClassFile, ParsingError};

/// Runtime representation of a class loader.
///
/// This is the structure that will be used to load classes at runtime, and
/// ensure that each class is loaded only once, and correcly (in order).
#[derive(Debug)]
pub struct ClassLoader {
    pub class_path: ClassPath,
}

impl ClassLoader {
    /// Create a new class loader.
    pub fn new() -> Self {
        Self {
            class_path: ClassPath::new(),
        }
    }

    /// Register a new class path entry to this class loader.
    pub fn add_class_path_entry(&mut self, entry: Box<dyn ClassPathEntry>) {
        self.class_path.add_entry(entry);
    }

    /// Load a class from this class loader.
    pub fn load_classfile(&mut self, class_name: &str) -> Result<ClassFile, ClassLoadingError> {
        let bytes = self.class_path.read_class(class_name)?;
        match ClassFile::from_bytes(&bytes) {
            Ok(classfile) => Ok(classfile),
            Err(e) => Err(ClassLoadingError::DocodingError(e)),
        }
    }
}

/// Runtime representation of a class path.
///
/// This is the structure that will be used to search for classes at runtime,
/// and retrieve their classfile.
#[derive(Debug, Default)]
pub struct ClassPath {
    entries: Vec<Box<dyn ClassPathEntry>>,
}

impl ClassPath {
    /// Create a new empty class path.
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Add a new class path entry to this class path.
    pub fn add_entry(&mut self, entry: Box<dyn ClassPathEntry>) {
        self.entries.push(entry);
    }

    /// Read a classfile from this class path.
    ///
    /// Returns the bytes of the classfile, or an error if the classfile could not be found or loaded.
    pub fn read_class(&self, name: &str) -> Result<Vec<u8>, ClassLoadingError> {
        for entry in &self.entries {
            match entry.read_class(name) {
                Ok(bytes) => return Ok(bytes),
                Err(ClassLoadingError::NotFound) => continue,
                Err(e) => return Err(e),
            }
        }
        Err(ClassLoadingError::NotFound)
    }
}

/// Class path entry trait.
///
/// This trait is used to represent a class path entry, which is a way to
/// register a loader that can load classes from a specific location (from File, from Jar Archive, ...).
pub trait ClassPathEntry: Debug {
    /// Read a classfile from this class path entry.
    ///
    /// Returns the bytes of the classfile, or an error if the classfile could not be found or loaded.
    fn read_class(&self, name: &str) -> Result<Vec<u8>, ClassLoadingError>;
}

#[derive(Debug)]
pub enum ClassLoadingError {
    NotFound,
    IOError(std::io::Error),
    DocodingError(ParsingError),
    Unknown,
}

/// Class path entry for a directory.
///
/// This is a class path entry that will load classes (in .class files) from a directory, or subdirectory.
#[derive(Debug)]
pub struct ClassPathDirEntry {
    /// The path of the root directory.
    path: std::path::PathBuf,
}

impl ClassPathEntry for ClassPathDirEntry {
    fn read_class(&self, name: &str) -> Result<Vec<u8>, ClassLoadingError> {
        let mut path = self.path.clone();
        for part in name.split('.') {
            path.push(part);
        }
        path.set_extension("class");
        match std::fs::read(path) {
            Ok(bytes) => Ok(bytes),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Err(ClassLoadingError::NotFound),
                _ => Err(ClassLoadingError::IOError(e)),
            },
        }
    }
}
