/* 
    importer.rs

    This is a structure for translating pacman files (they are readed using libarchive), to ostree format
*/

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use bitflags::bitflags;
use crate::utils;
use anyhow::{Result, bail};


bitflags! {
    #[derive(Debug)]
    pub struct PacmanImporterFlags: u8 {
        // Skip files that are not compatible with ostree (e.g. symlinks, device files, etc.)
        const SKIP_NOTCOMPATIBLE = 0b00000001;

        // Skip documentation files
        const SKIP_DOCS = 0b00000010;
    }
}

#[derive(Debug)]
pub struct PacmanImporter {
    pub flags: PacmanImporterFlags,
    // The list of files that are considered documentation files
    pub docs_files: Option<BTreeSet<String>>,

    // The list of files that are considered variable files (e.g. configuration files, log files, etc.)
    pub var_files: BTreeSet<String>,

    // The list of files that are considered optional files (e.g. files that are not required for the package to work, but are still included in the package)
    pub opt_files: BTreeSet<String>,

    //Created tmpfiles entries for the package, to be used in the ostree commit
    pub tmpfiles_entries: Vec<String>,

    pub pkg_name: String,

}

impl PacmanImporter {

    // Create a new PacmanImporter with the given package name and flags
    pub fn new(pkg_name: &str, flags: PacmanImporterFlags) -> Result<Self> {

        if pkg_name.is_empty() {
            bail!("Package name cannot be empty");
        }

        let doc_files = if flags.contains(PacmanImporterFlags::SKIP_DOCS) {
            Some(BTreeSet::new())
        } else {
            None
        };

        Ok(Self {
            flags,
            docs_files: doc_files,
            var_files: BTreeSet::new(),
            opt_files: BTreeSet::new(),
            tmpfiles_entries: Vec::new(),
            pkg_name: pkg_name.to_string(),
        })
    }

    pub fn process_path(&mut self, path: &str) -> Option<String> {
        // First step of translation is to check if the path is compliant with ostree, if not, we skip it (if the flag is set) or we bail with an error
        if !path_is_ostree_compliant(path) {
            Some(path.to_string())
        }
        // If the path is not compliant with ostree, we check if the flag to skip not compatible files is set, if it is, we skip it, otherwise we translate it to ostree format
        else if self.flags.contains(PacmanImporterFlags::SKIP_NOTCOMPATIBLE) {
            None
        }
        else {
            Some(utils::translate_path_for_ostree(path))
        }

        //Handle /opt files
    }
}

// Check if the given path is compliant with ostree (e.g. it is not a symlink, device file, etc.)
fn path_is_ostree_compliant(path: &str) -> bool {
    if matches!(path, "/" | "/usr" | "/bin" | "/sbin" | "/lib" | "/lib64") {
        return true;
    }

    if path.starts_with("/bin/")
        || path.starts_with("/sbin/")
        || path.starts_with("/lib/")
        || path.starts_with("/lib64/")
    {
        return true;
    }

    if path.starts_with("/usr/") {
        return true;
    }

    false
}