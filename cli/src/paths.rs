use std::path::PathBuf;

/// Returns the default `filename` file located at the user's home directory
/// If the file does not exist then it returns `None`
pub fn get_default_path(filename: PathBuf) -> Option<PathBuf> {
    home::home_dir()
        .map(|mut pathbuf| {
            pathbuf.push(filename);
            pathbuf
        })
        .filter(|pb| pb.exists() && pb.is_file())
}
