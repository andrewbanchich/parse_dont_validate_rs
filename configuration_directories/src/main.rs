use std::{
    env,
    path::{Path, PathBuf},
};

mod non_empty;

fn main() {
    let config_dirs = get_configuration_directories();

    match head(&config_dirs) {
        Some(cache_dir) => initialize_cache(cache_dir),
        None => panic!("should never happen; already checked configDirs is non-empty"),
    }
}

fn get_configuration_directories() -> Vec<PathBuf> {
    let config_dirs_string = env::var("CONFIG_DIRS").unwrap_or_default();
    let config_dirs_list: Vec<_> = config_dirs_string.split(',').map(|s| s.into()).collect();

    if config_dirs_list.is_empty() {
        panic!("CONFIG_DIRS cannot be empty")
    }

    config_dirs_list
}

fn head<T>(slice: &[T]) -> Option<&T> {
    slice.get(0)
}

fn initialize_cache(cache_dir: &Path) {
    todo!("just imagine this does something")
}
