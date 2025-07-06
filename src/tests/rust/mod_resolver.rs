use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use syn::{Attribute, Item, ItemMod, LitStr, Meta};

use crate::errors::FztError;

fn resolve_module_path(base_file: &PathBuf, path: &PathBuf, mod_name: &str) -> PathBuf {
    let candidate1 = path.join(format!("{}.rs", mod_name));
    let candidate2 = path.join(mod_name).join("mod.rs");
    if candidate1.exists() {
        candidate1
    } else if candidate2.exists() {
        candidate2
    } else {
        base_file.clone()
    }
}

fn resolve_module(
    base_file: &PathBuf,
    path: &PathBuf,
    module_item: &ItemMod,
    module_path: &[String],
    seen: &mut HashMap<Vec<String>, PathBuf>,
) -> Result<(), FztError> {
    let mod_name = module_item.ident.to_string();
    if module_item.attrs.iter().len() == 0 {
        let mut new_module_path = module_path.to_vec();
        new_module_path.push(mod_name.clone());
        seen.insert(
            new_module_path.clone(),
            resolve_module_path(base_file, path, &mod_name),
        );
        if let Some((_, nested_items)) = &module_item.content {
            for item in nested_items {
                if let Item::Mod(submod) = item {
                    resolve_module(base_file, &path, submod, new_module_path.as_slice(), seen)?;
                }
            }
        }
        return Ok(());
    }
    for attr in module_item.attrs.iter() {
        if attr.path().is_ident("path") {
            let mut new_module_path = module_path.to_vec();
            let lit: LitStr = attr.parse_args()?;
            let lit_value = lit.value().clone();
            let relative_path = Path::new(&lit_value);
            if !lit_value.ends_with(".rs") {
                new_module_path.push(mod_name.clone());
                seen.insert(new_module_path.clone(), base_file.clone());
                if let Some((_, nested_items)) = &module_item.content {
                    for item in nested_items {
                        if let Item::Mod(submod) = item {
                            resolve_module(
                                base_file,
                                &path.join(relative_path),
                                submod,
                                new_module_path.as_slice(),
                                seen,
                            )?;
                        }
                    }
                }
            } else {
                let mod_name = module_item.ident.to_string();
                new_module_path.push(mod_name.clone());
                seen.insert(new_module_path, path.join(relative_path));
            }
        } else {
            let mod_name = module_item.ident.to_string();
            if let Some((_, sub_items)) = &module_item.content {
                if sub_items.is_empty() {
                    let mut new_module_path = module_path.to_vec();
                    new_module_path.push(mod_name.clone());
                    seen.insert(
                        new_module_path.clone(),
                        resolve_module_path(base_file, path, &mod_name),
                    );
                } else {
                    let mut new_module_path = module_path.to_vec();
                    new_module_path.push(mod_name.clone());
                    seen.insert(new_module_path.clone(), base_file.clone());
                    for item in sub_items {
                        if let Item::Mod(submod) = item {
                            resolve_module(
                                base_file,
                                &path,
                                submod,
                                new_module_path.as_slice(),
                                seen,
                            )?;
                        }
                    }
                }
            } else {
                let mut new_module_path = module_path.to_vec();
                new_module_path.push(mod_name.clone());
                seen.insert(
                    new_module_path.clone(),
                    resolve_module_path(base_file, path, &mod_name),
                );
            }
        }
    }

    Ok(())
}

fn path_visit(
    path: &PathBuf,
    module_paths: &mut HashMap<Vec<String>, PathBuf>,
    module_path: &[String],
) -> Result<(), FztError> {
    let content = fs::read_to_string(path)?;
    let file = syn::parse_file(&content)?;

    for item in file.items {
        if let Item::Mod(submod) = item {
            let mut local_seen = HashMap::new();
            resolve_module(
                path,
                &path.parent().unwrap().to_path_buf(),
                &submod,
                &module_path,
                &mut local_seen,
            )?;
            for (module_path, module_file_path) in local_seen.into_iter() {
                module_paths.insert(module_path.clone(), module_file_path.clone());
                if path != &module_file_path {
                    path_visit(&module_file_path, module_paths, &module_path)?;
                }
            }
        }
    }
    Ok(())
}

pub fn get_module_paths(root_path: &PathBuf) -> Result<HashMap<Vec<String>, PathBuf>, FztError> {
    let mut module_paths = HashMap::new();
    path_visit(root_path, &mut module_paths, &[])?;
    Ok(module_paths)
}

#[cfg(test)]
mod tests {
    use syn::Item;

    use crate::tests::rust::mod_resolver::{get_module_paths, resolve_module};
    use std::{
        collections::HashMap,
        path::{Path, PathBuf},
    };

    #[test]
    fn parse_file() {
        let path = Path::new("src/tests/rust/test_data/mods/nested_path_attr/src/lib.rs");
        let mut seen = HashMap::new();
        let item = &syn::parse_file(&std::fs::read_to_string(path).unwrap())
            .unwrap()
            .items[0];

        let thread_key = vec!["crate".to_string(), "thread".to_string()];
        let local_data_key = vec![
            "crate".to_string(),
            "thread".to_string(),
            "local_data".to_string(),
        ];
        let local_pf_key = vec!["crate".to_string(), "thread".to_string(), "pf".to_string()];
        let local_pf_local_data_key = vec![
            "crate".to_string(),
            "thread".to_string(),
            "pf".to_string(),
            "local_data".to_string(),
        ];
        let local_pf_hello_key = vec![
            "crate".to_string(),
            "thread".to_string(),
            "pf".to_string(),
            "hello".to_string(),
        ];
        let local_foo_key = vec!["crate".to_string(), "thread".to_string(), "foo".to_string()];

        if let Item::Mod(submod) = item {
            resolve_module(
                &path.to_path_buf(),
                &path.parent().unwrap().to_path_buf(),
                submod,
                &vec!["crate".to_string()],
                &mut seen,
            )
            .unwrap();

            println!("{:?}", seen);

            assert_eq!(
                seen.get(&thread_key).unwrap().to_str().unwrap(),
                "src/tests/rust/test_data/mods/nested_path_attr/src/lib.rs"
            );
            assert_eq!(
                seen.get(&local_data_key).unwrap().to_str().unwrap(),
                "src/tests/rust/test_data/mods/nested_path_attr/src/thread_files/tls.rs"
            );
            assert_eq!(
                seen.get(&local_pf_key).unwrap().to_str().unwrap(),
                "src/tests/rust/test_data/mods/nested_path_attr/src/lib.rs"
            );
            assert_eq!(
                seen.get(&local_pf_local_data_key)
                    .unwrap()
                    .to_str()
                    .unwrap(),
                "src/tests/rust/test_data/mods/nested_path_attr/src/thread_files/process_files/pid.rs"
            );
            assert_eq!(
                seen.get(&local_pf_hello_key).unwrap().to_str().unwrap(),
                "src/tests/rust/test_data/mods/nested_path_attr/src/thread_files/process_files/hello/mod.rs"
            );
            assert_eq!(
                seen.get(&local_foo_key).unwrap().to_str().unwrap(),
                "src/tests/rust/test_data/mods/nested_path_attr/src/lib.rs"
            );
        } else {
            panic!("now mod item exists");
        }
    }

    fn map_keys(map: &HashMap<Vec<String>, PathBuf>) -> Vec<String> {
        let mut keys: Vec<String> = map.keys().map(|k| k.join("::")).collect();
        keys.sort();
        keys
    }

    #[test]
    fn resolves_standard_mod_structure() {
        let path = Path::new("src/tests/rust/test_data/mods/standard/src/lib.rs");
        let map = get_module_paths(&path.to_path_buf()).unwrap();
        let keys = map_keys(&map);
        assert_eq!(keys, vec!["a", "a::b",]);
    }

    #[test]
    fn resolves_custom_path_attribute() {
        let path = Path::new("src/tests/rust/test_data/mods/custom_path/src/lib.rs");
        let map = get_module_paths(&path.to_path_buf()).unwrap();
        let keys = map_keys(&map);
        assert_eq!(keys, vec!["custom_mod",]);
    }

    #[test]
    fn handles_inline_mods() {
        let path = Path::new("src/tests/rust/test_data/mods/inline/src/lib.rs");
        let map = get_module_paths(&path.to_path_buf()).unwrap();
        let keys = map_keys(&map);
        assert_eq!(keys, vec!["inline", "inline::nested",]);
    }

    #[test]
    fn resolves_nested_custom_path() {
        let path = Path::new("src/tests/rust/test_data/mods/nested_custom/src/lib.rs");
        let map = get_module_paths(&path.to_path_buf()).unwrap();
        let keys = map_keys(&map);
        assert_eq!(keys, vec!["nested", "nested::deep_mod",]);
    }

    #[test]
    fn resolves_nested_path_attributes() {
        let path = Path::new("src/tests/rust/test_data/mods/nested_path_attr/src/lib.rs");
        let map = get_module_paths(&path.to_path_buf()).unwrap();
        let thread_key = vec!["thread".to_string()];
        let local_data_key = vec!["thread".to_string(), "local_data".to_string()];
        let local_pf_key = vec!["thread".to_string(), "pf".to_string()];
        let local_pf_local_data_key = vec![
            "thread".to_string(),
            "pf".to_string(),
            "local_data".to_string(),
        ];
        let local_pf_hello_key = vec!["thread".to_string(), "pf".to_string(), "hello".to_string()];
        let local_foo_key = vec!["thread".to_string(), "foo".to_string()];

        assert_eq!(
            map.get(&thread_key).unwrap().to_str().unwrap(),
            "src/tests/rust/test_data/mods/nested_path_attr/src/lib.rs"
        );
        assert_eq!(
            map.get(&local_data_key).unwrap().to_str().unwrap(),
            "src/tests/rust/test_data/mods/nested_path_attr/src/thread_files/tls.rs"
        );
        assert_eq!(
            map.get(&local_pf_key).unwrap().to_str().unwrap(),
            "src/tests/rust/test_data/mods/nested_path_attr/src/lib.rs"
        );
        assert_eq!(
            map.get(&local_pf_local_data_key).unwrap().to_str().unwrap(),
            "src/tests/rust/test_data/mods/nested_path_attr/src/thread_files/process_files/pid.rs"
        );
        assert_eq!(
            map.get(&local_pf_hello_key).unwrap().to_str().unwrap(),
            "src/tests/rust/test_data/mods/nested_path_attr/src/thread_files/process_files/hello/mod.rs"
        );
        assert_eq!(
            map.get(&local_foo_key).unwrap().to_str().unwrap(),
            "src/tests/rust/test_data/mods/nested_path_attr/src/lib.rs"
        );
    }
}
