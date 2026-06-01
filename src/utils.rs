/*
    utils.rs

    This is a module for utility functions used throughout the project
*/

pub(crate) fn translate_path_for_ostree(path: &str) -> String {
    translate_path_for_ostree_impl(path).unwrap_or_default()
}

fn translate_path_for_ostree_impl(path: &str) -> Option<String> {
    assert!(!path.starts_with('/'));
    assert!(!path.starts_with("./"));

    // etc/foo -> usr/etc/foo
    if path == "etc" || path.starts_with("etc/") {
        return Some("usr/".to_string() + path);
    }

    // boot/foo -> usr/lib/ostree-boot/foo
    if let Some(prefixless) = path.strip_prefix("boot/") {
        return Some("usr/lib/ostree-boot/".to_string() + prefixless);
    }

    // opt/foo -> usr/lib/opt/foo
    if path == "opt" || path.starts_with("opt/") {
        return Some("usr/lib/".to_string() + path);
    }

    // var/lib/{special}/foo -> usr/lib/{special}/foo
    if let Some(prefixless) = path.strip_prefix("var/lib/") {
        let varlib_cases = &["alternatives", "vagrant"];
        for entry in varlib_cases {
            if prefixless.starts_with(entry) {
                return Some("usr/lib/".to_string() + prefixless);
            }
        }
    }

    // All remaining cases do not need translation.
    None
}