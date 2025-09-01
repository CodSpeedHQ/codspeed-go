use std::path::{Path, PathBuf};
use std::{fs, io};

pub fn copy_dir_recursively(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            if entry.file_name() == ".git" {
                continue;
            }

            copy_dir_recursively(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn get_parent_git_repo_path(abs_path: &Path) -> io::Result<PathBuf> {
    if abs_path.join(".git").exists() {
        Ok(abs_path.to_path_buf())
    } else {
        get_parent_git_repo_path(
            abs_path
                .parent()
                .ok_or(io::Error::from(io::ErrorKind::NotFound))?,
        )
    }
}

pub fn get_git_relative_path<P>(abs_path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    if let Ok(canonicalized_abs_path) = abs_path.as_ref().canonicalize() {
        // `repo_path` is still canonicalized as it is a subpath of `canonicalized_abs_path`
        if let Ok(repo_path) = get_parent_git_repo_path(&canonicalized_abs_path) {
            canonicalized_abs_path
                .strip_prefix(repo_path)
                .expect("Repository path is malformed.")
                .to_path_buf()
        } else {
            canonicalized_abs_path
        }
    } else {
        abs_path.as_ref().to_path_buf()
    }
}
