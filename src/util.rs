use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn ensure_file_exists(
    path: &Path,
    default_content: Option<&str>,
) -> Result<bool, std::io::Error> {
    // Make sure the path exists
    path.parent().map(|p| std::fs::create_dir_all(p));

    // Make sure the file exists
    match OpenOptions::new().create_new(true).write(true).open(path) {
        Ok(mut f) => {
            // Write the default content
            if let Some(content) = default_content {
                f.write_all(content.as_bytes())?;
            }
            Ok(true)
        }
        Err(e) => {
            // If the file already exists, that's fine
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                Ok(false)
            } else {
                Err(e)
            }
        }
    }
}
