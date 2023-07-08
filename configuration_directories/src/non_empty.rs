use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    let config_dirs = get_configuration_directories();
    initialize_cache(&head(config_dirs));
}

fn get_configuration_directories() -> NonEmptyVec<PathBuf> {
    let config_dirs_string = env::var("CONFIG_DIRS").unwrap_or_default();
    let mut config_dirs_list: Vec<_> = config_dirs_string.split(',').map(|s| s.into()).collect();

    match config_dirs_list.pop() {
        Some(head) => NonEmptyVec(head, config_dirs_list),
        None => panic!("CONFIG_DIRS cannot be empty"),
    }
}

struct NonEmptyVec<T>(T, Vec<T>);

fn head<T>(vec: NonEmptyVec<T>) -> T {
    vec.0
}

fn initialize_cache(cache_dir: &Path) {
    todo!("just imagine this does something")
}

fn validate_non_empty<T>(vec: Vec<T>) -> Result<(), String> {
    if vec.is_empty() {
        Err("Slice was empty".to_string())
    } else {
        Ok(())
    }
}

fn parse_non_empty<T>(mut vec: Vec<T>) -> Result<NonEmptyVec<T>, String> {
    match vec.pop() {
        None => Err("Vec was empty".to_string()),
        Some(head) => Ok(NonEmptyVec(head, vec)),
    }
}

// parseNonEmpty :: [a] -> IO (NonEmpty a)
// parseNonEmpty (x:xs) = pure (x:|xs)
// parseNonEmpty [] = throwIO $ userError "list cannot be empty"
