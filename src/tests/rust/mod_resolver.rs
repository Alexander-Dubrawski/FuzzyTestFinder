use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use syn::{Attribute, Item, ItemMod, LitStr, Meta};

use crate::errors::FztError;

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
        let candidate1 = path.join(format!("{}.rs", mod_name));
        let candidate2 = path.join(mod_name).join("mod.rs");
        if candidate1.exists() {
            seen.insert(new_module_path.clone(), candidate1);
        } else if candidate2.exists() {
            seen.insert(new_module_path.clone(), candidate2);
        } else {
            seen.insert(new_module_path.clone(), base_file.clone());
        }
        if let Some((_, nested_items)) = &module_item.content {
            for item in nested_items {
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
                    let candidate1 = path.join(format!("{}.rs", mod_name));
                    let candidate2 = path.join(mod_name).join("mod.rs");
                    if candidate1.exists() {
                        seen.insert(new_module_path, candidate1);
                    } else if candidate2.exists() {
                        seen.insert(new_module_path, candidate2);
                    } else {
                        seen.insert(new_module_path, base_file.clone());
                    }
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
                let candidate1 = path.join(format!("{}.rs", mod_name));
                let candidate2 = path.join(mod_name).join("mod.rs");
                if candidate1.exists() {
                    seen.insert(new_module_path, candidate1);
                } else if candidate2.exists() {
                    seen.insert(new_module_path, candidate2);
                } else {
                    seen.insert(new_module_path, base_file.clone());
                }
            }
        }
    }

    Ok(())
}

pub fn path_visit(path: &PathBuf, module_paths: &mut HashMap<Vec<String>, PathBuf>, module_path: &[String],) -> Result<(), FztError> {
    let content = fs::read_to_string(path)?;
    let file = syn::parse_file(&content)?;

    for item in file.items {
        if let Item::Mod(submod) = item {
            let mut local_seen = HashMap::new();
            resolve_module(path, &path.parent().unwrap().to_path_buf(), &submod, &module_path, &mut local_seen)?;
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
    use std::{collections::HashMap, path::{Path, PathBuf}};

    #[test]
    fn foo() {
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
        let mut keys: Vec<String> = map.keys()
            .map(|k| k.join("::"))
            .collect();
        keys.sort();
        keys
    }

    #[test]
    fn resolves_standard_mod_structure() {
        let path = Path::new("src/tests/rust/test_data/mods/standard/src/lib.rs");
        let map = get_module_paths(&path.to_path_buf()).unwrap();
        let keys = map_keys(&map);
        assert_eq!(
            keys,
            vec![
                "a",
                "a::b",
            ]
        );
    }

    #[test]
    fn resolves_custom_path_attribute() {
        let path = Path::new("src/tests/rust/test_data/mods/custom_path/src/lib.rs");
        let map = get_module_paths(&path.to_path_buf()).unwrap();
        let keys = map_keys(&map);
        assert_eq!(
            keys,
            vec![
                "custom_mod",
            ]
        );
    }

    #[test]
    fn handles_inline_mods() {
        let path = Path::new("src/tests/rust/test_data/mods/inline/src/lib.rs");
        let map = get_module_paths(&path.to_path_buf()).unwrap();
        let keys = map_keys(&map);
        assert_eq!(
            keys,
            vec![
                "inline",
                "inline::nested",
            ]
        );
    }

    #[test]
    fn resolves_nested_custom_path() {
        let path = Path::new("src/tests/rust/test_data/mods/nested_custom/src/lib.rs");
        let map = get_module_paths(&path.to_path_buf()).unwrap();
        let keys = map_keys(&map);
        assert_eq!(
            keys,
            vec![
                "nested",
                "nested::deep_mod",
            ]
        );
    }

}

// fn extract_modules_from_item(
//     item: &Item,
//     path: &PathBuf,
//     module_path: &[String]
// ) -> Result<Vec<ModuleType>, FztError> {
//     if let Item::Mod(submod) = item {
//         if let Some((_, sub_items)) = &submod.content {
//             sub_items.iter().map(|module_item| {
//                 resolve_module(path, submod, module_path)
//             }).collect()
//         }
//     }
//     Ok(vec![])

// }

// fn extract_modules_from_file(
//     path: &Path,
// ) ->  Result<HashMap<Vec<String>, PathBuf>, FztError> {
//     let content = fs::read_to_string(path)?;
//     let file = syn::parse_file(&content)?;
//     for item in file.items {

//     }

// }

// pub fn build_module_map(root: &Path) -> Result<HashMap<Vec<String>, PathBuf>, FztError> {
//     let mut seen = HashMap::new();
//     visit_module_file(root, vec!["crate".into()], root.parent().unwrap_or(Path::new("")), &mut seen)?;
//     Ok(seen)
// }

// fn visit_inline_mod(
//     path: &Path,
//     mod_stack: &[String],
//     parent_dir: &Path,
//     seen: &mut HashMap<Vec<String>, PathBuf>,
//     item: &Item,
// ) -> Result<(), FztError> {

//     if let Item::Mod(submod) = item {

//         // Inline: don't use parent dir, path is the parent one in this case
//         let resolved_mod_path = resolve_mod_file(&submod, path.parent().unwrap())?;

//         let mut next_stack = mod_stack.to_vec().clone();
//         next_stack.push(submod.ident.to_string());
//         seen.insert(next_stack.clone(), resolved_mod_path.to_path_buf().clone());
//         if let Some((_, sub_items)) = &submod.content {
//              for sub_item in sub_items {
//                 if let Some(mod_path) = resolved_mod_path.clone() {
//                     visit_inline_mod(mod_path.as_path(), &next_stack, parent_dir, seen, &sub_item)?;
//                 }  else {
//                     visit_inline_mod(path, &next_stack, parent_dir, seen, &sub_item)?;
//                 }
//              }
//         } else {
//             // External mod
//             if let Some(mod_path) = resolved_mod_path {
//                 visit_module_file(&mod_path, next_stack, mod_path.parent().unwrap_or(parent_dir), seen)?;
//             }
//         }
//     }
//     Ok(())
// }

// fn visit_module_file(
//     path: &Path,
//     mod_stack: Vec<String>,
//     parent_dir: &Path,
//     seen: &mut HashMap<Vec<String>, PathBuf>,
// ) -> Result< (), FztError> {
//     let content = fs::read_to_string(path)?;
//     let file = syn::parse_file(&content)?;

//     for item in file.items {
//         if let Item::Mod(m) = item {
//             let mut next_stack = mod_stack.clone();

//             next_stack.push(m.ident.to_string());

//             let resolved_mod_path = resolve_mod_file(&m, path.parent().unwrap())?;
//             seen.insert(next_stack.clone(), path.to_path_buf().clone());

//             if let Some((_, items)) = m.content {
//                 for sub_item in items {
//                     visit_inline_mod(resolved_mod_path.as_path(), &next_stack, parent_dir, seen, &sub_item)?;
//                 }
//             } else {
//                 visit_module_file(&resolved_mod_path, next_stack, resolved_mod_path.parent().unwrap_or(parent_dir), seen)?;
//             }
//         }
//     }
//     Ok(())
// }

// fn resolve_mod_file(m: &ItemMod, parent_dir: &Path) -> Result<PathBuf, FztError>  {
//     let mod_name = m.ident.to_string();
//     // #[path = "..."]
//     for attr in m.attrs.iter() {
//         let custom_path = get_path_attr(attr, parent_dir)?;
//         if let Some(path_str) = custom_path {
//             return Ok(path_str)
//         }
//     }
//     let candidate1 = parent_dir.join(format!("{}.rs", mod_name));
//     let candidate2 = parent_dir.join(mod_name).join("mod.rs");
//     if candidate1.exists() {
//         Ok(candidate1)
//     } else if candidate2.exists() {
//         Ok(candidate2)
//     } else {
//         // TODO: return parent file path
//         Ok(parent_dir.to_path_buf())
//     }
// }

// fn get_path_attr(attr: &Attribute, parent_dir: &Path) -> Result<Option<PathBuf>, FztError> {
//     if attr.path().is_ident("path") {
//         let lit: LitStr = attr.parse_args()?;
//         let lit_value = lit.value();
//         let relative_path = Path::new(&lit_value);
//         // Just join and normalize (but don't canonicalize to avoid fs dependency)
//         let resolved = parent_dir.join(relative_path).components().collect::<PathBuf>();

//         return Ok(Some(resolved));
//     }
//     Ok(None)
// }

// #[cfg(test)]
// mod tests {
//     use std::collections::HashMap;
//     use std::path::{Path, PathBuf};

//     use crate::tests::rust::mod_resolver::build_module_map;

//     fn map_keys(map: &HashMap<Vec<String>, PathBuf>) -> Vec<String> {
//         let mut keys: Vec<String> = map.keys()
//             .map(|k| k.join("::"))
//             .collect();
//         keys.sort();
//         keys
//     }

//     #[test]
//     fn resolves_standard_mod_structure() {
//         let path = Path::new("src/tests/rust/test_data/mods/standard/src/lib.rs");
//         let map = build_module_map(path);
//         let keys = map_keys(&map.unwrap());
//         assert_eq!(
//             keys,
//             vec![
//                 "crate::a",
//                 "crate::a::b",
//             ]
//         );
//     }

//     #[test]
//     fn resolves_custom_path_attribute() {
//         let path = Path::new("src/tests/rust/test_data/mods/custom_path/src/lib.rs");
//         let map = build_module_map(path).unwrap();
//         let keys = map_keys(&map);
//         assert_eq!(
//             keys,
//             vec![
//                 "crate::custom_mod",
//             ]
//         );
//     }

//     #[test]
//     fn handles_inline_mods() {
//         let path = Path::new("src/tests/rust/test_data/mods/inline/src/lib.rs");
//         let map = build_module_map(path).unwrap();
//         let keys = map_keys(&map);
//         assert_eq!(
//             keys,
//             vec![
//                 "crate::inline",
//                 "crate::inline::nested",
//             ]
//         );
//     }

//     #[test]
//     fn resolves_nested_custom_path() {
//         let path = Path::new("src/tests/rust/test_data/mods/nested_custom/src/lib.rs");
//         let map = build_module_map(path).unwrap();
//         let keys = map_keys(&map);
//         assert_eq!(
//             keys,
//             vec![
//                 "crate::nested",
//                 "crate::nested::deep_mod",
//             ]
//         );
//     }

//     #[test]
//     fn resolves_nested_path_attributes() {
//         let path = Path::new("src/tests/rust/test_data/mods/nested_path_attr/src/lib.rs");
//         let map = build_module_map(path).unwrap();
//         let thread_key = vec!["crate".to_string(), "thread".to_string()];
//         let local_data_key = vec!["crate".to_string(), "thread".to_string(), "local_data".to_string()];
//         assert_eq!(map.get(&thread_key).unwrap().to_str().unwrap(), "src/tests/rust/test_data/mods/nested_path_attr/src/lib.rs");
//         assert_eq!(map.get(&local_data_key).unwrap().to_str().unwrap(), "src/tests/rust/test_data/mods/nested_path_attr/src/thread_files/tls.rs");

//     }
// }
