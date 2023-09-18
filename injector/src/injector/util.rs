use std::path::Path;

pub fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
    const VERBATIM_PREFIX: &str = r#"\\?\"#;

    let p = p.as_ref().display().to_string();
    if let Some(e) = p.strip_prefix(VERBATIM_PREFIX) {
        e.to_string()
    } else {
        p
    }
}
