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

    // The list of files that are considered variable files (e.g. configuration files, log files, etc.)
    pub var_files: BTreeSet<String>,

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

        Ok(Self {
            flags,
            var_files: BTreeSet::new(),
            tmpfiles_entries: Vec::new(),
            pkg_name: pkg_name.to_string(),
        })
    }

    pub fn process_path(&mut self, path: &str) -> Option<String> {

        if is_opt_path(path) {
            self.tmpfiles_entries.push(opt_to_tmpfiles(path));
            return None;
        }

        if self.flags.contains(PacmanImporterFlags::SKIP_DOCS) && is_doc_path(path) {
            return None;
        }

        // First step of translation is to check if the path is compliant with ostree, if not, we skip it (if the flag is set) or we bail with an error
        if path_is_ostree_compliant(path) {
            return Some(path.to_string());
        }
        // If the path is not compliant with ostree, we check if the flag to skip not compatible files is set, if it is, we skip it, otherwise we translate it to ostree format
        if self.flags.contains(PacmanImporterFlags::SKIP_NOTCOMPATIBLE) {
            return None;
        }
        
        Some(utils::translate_path_for_ostree(path))

        //Handle /opt files
    }
}

// Check if the given path is compliant with ostree (e.g. it is not a symlink, device file, etc.)
fn path_is_ostree_compliant(path: &str) -> bool {
    if matches!(path, "usr" | "bin" | "sbin" | "lib" | "lib64") {
        return true;
    }

    if path.starts_with("bin/")
        || path.starts_with("sbin/")
        || path.starts_with("lib/")
        || path.starts_with("lib64/")
    {
        return true;
    }

    if path.starts_with("usr/") {
        return true;
    }

    false
}

fn opt_to_tmpfiles(path: &str) -> String {
    let stripped = path.strip_prefix("opt/").unwrap_or(path);

    format!(
        "L /opt/{} - - - - /usr/lib/opt/{}",
        stripped,
        stripped
    )
}

fn is_opt_path(path: &str) -> bool {
    path == "opt" || path.starts_with("opt/")
}

fn is_doc_path(path: &str) -> bool {
    path.starts_with("usr/share/doc/") || 
    path.starts_with("usr/share/info/") || 
    path.starts_with("usr/share/man/") || 
    path.starts_with("usr/share/gtk-doc/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_is_ostree_compliant() {
        assert!(path_is_ostree_compliant("usr"));
        assert!(path_is_ostree_compliant("usr/bin/bash"));
        assert!(path_is_ostree_compliant("bin/sh"));

        assert!(!path_is_ostree_compliant("etc/passwd"));
        assert!(!path_is_ostree_compliant("opt/foo"));
        assert!(!path_is_ostree_compliant("var/lib/pacman"));
    }

    #[test]
    fn test_is_opt_path() {
        assert!(is_opt_path("opt"));
        assert!(is_opt_path("opt/foo"));

        assert!(!is_opt_path("usr/lib"));
        assert!(!is_opt_path("etc/foo"));
    }

    #[test]
    fn test_opt_to_tmpfiles() {
        assert_eq!(
            opt_to_tmpfiles("opt/foo"),
            "L /opt/foo - - - - /usr/lib/opt/foo"
        );

        assert_eq!(
            opt_to_tmpfiles("opt/bar/baz"),
            "L /opt/bar/baz - - - - /usr/lib/opt/bar/baz"
        );
    }

    #[test]
    fn test_process_path() {
        let mut importer =
            PacmanImporter::new("test", PacmanImporterFlags::empty())
                .unwrap();

        assert_eq!(
            importer.process_path("usr/bin/bash"),
            Some("usr/bin/bash".into())
        );

        assert_eq!(
            importer.process_path("etc/pacman.conf"),
            Some("usr/etc/pacman.conf".into())
        );

        assert_eq!(
            importer.process_path("opt/foo"),
            None
        );

        assert_eq!(importer.tmpfiles_entries.len(), 1);
    }
}