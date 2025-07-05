use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use syn::{Attribute, Item, ItemMod, LitStr, Meta};

use crate::errors::FztError;

pub fn build_module_map(root: &Path) -> Result<HashMap<Vec<String>, PathBuf>, FztError> {
    let mut seen = HashMap::new();
    visit_module_file(root, vec!["crate".into()], root.parent().unwrap_or(Path::new("")), &mut seen)?;
    Ok(seen)
}


fn visit_inline_mod(
    path: &Path,
    mod_stack: &[String],
    parent_dir: &Path,
    seen: &mut HashMap<Vec<String>, PathBuf>,
    item: &Item,
) -> Result<(), FztError> {
    let mut next_stack = mod_stack.to_vec().clone();
    seen.insert(next_stack.clone(), path.to_path_buf().clone());
    
    if let Item::Mod(submod) = item {
        next_stack.push(submod.ident.to_string());
        if let Some((_, sub_items)) = &submod.content {
             for sub_item in sub_items {
                visit_inline_mod(path, &next_stack, parent_dir, seen, sub_item)?;
             }
             seen.insert(next_stack.clone(), path.to_path_buf().clone());
        } else {
            // External mod
            if let Some(mod_path) = resolve_mod_file(&submod, parent_dir)? {
                visit_module_file(&mod_path, next_stack, mod_path.parent().unwrap_or(parent_dir), seen)?;
            } else {
                seen.insert(next_stack.clone(), path.to_path_buf().clone());
            }
        }
    }
    Ok(())
}

fn visit_module_file(
    path: &Path,
    mod_stack: Vec<String>,
    parent_dir: &Path,
    seen: &mut HashMap<Vec<String>, PathBuf>,
) -> Result< (), FztError> {
    let content = fs::read_to_string(path)?;
    let file = syn::parse_file(&content)?;

    //let abs_path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    seen.insert(mod_stack.clone(), path.to_path_buf());

    for item in file.items {
        if let Item::Mod(m) = item {
            let mut next_stack = mod_stack.clone();
            next_stack.push(m.ident.to_string());

            if let Some((_, items)) = m.content {
                for sub_item in items {
                    visit_inline_mod(path, &next_stack, parent_dir, seen, &sub_item)?;
                }
            } else {
                // External mod
                if let Some(mod_path) = resolve_mod_file(&m, parent_dir)? {
                    visit_module_file(&mod_path, next_stack, mod_path.parent().unwrap_or(parent_dir), seen)?;
                }
            }
        }
    }
    Ok(())
}

fn resolve_mod_file(m: &ItemMod, parent_dir: &Path) -> Result<Option<PathBuf>, FztError>  {
    let mod_name = m.ident.to_string();
    // #[path = "..."]
    for attr in m.attrs.iter() {
        let custom_path = get_path_attr(attr, parent_dir)?;
        if let Some(path_str) = custom_path {
            return Ok(Some(path_str))
        }
    }
    let candidate1 = parent_dir.join(format!("{}.rs", mod_name));
    let candidate2 = parent_dir.join(mod_name).join("mod.rs");
    if candidate1.exists() {
        Ok(Some(candidate1))
    } else if candidate2.exists() {
        Ok(Some(candidate2))
    } else {
        Ok(None)
    }
}

fn get_path_attr(attr: &Attribute, parent_dir: &Path) -> Result<Option<PathBuf>, FztError> {
    if attr.path().is_ident("path") {
        let lit: LitStr = attr.parse_args()?;
        let lit_value = lit.value();
        let relative_path = Path::new(&lit_value);
        // Just join and normalize (but don't canonicalize to avoid fs dependency)
        let resolved = parent_dir.join(relative_path).components().collect::<PathBuf>();

        return Ok(Some(resolved));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    use crate::tests::rust::mod_resolver::build_module_map;

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
        let map = build_module_map(path);
        let keys = map_keys(&map.unwrap());
        assert_eq!(
            keys,
            vec![
                "crate",
                "crate::a",
                "crate::a::b",
            ]
        );
    }

    #[test]
    fn resolves_custom_path_attribute() {
        let path = Path::new("src/tests/rust/test_data/mods/custom_path/src/lib.rs");
        let map = build_module_map(path).unwrap();
        let keys = map_keys(&map);
        assert_eq!(
            keys,
            vec![
                "crate",
                "crate::custom_mod",
            ]
        );
    }

    #[test]
    fn handles_inline_mods() {
        let path = Path::new("src/tests/rust/test_data/mods/inline/src/lib.rs");
        let map = build_module_map(path).unwrap();
        let keys = map_keys(&map);
        assert_eq!(
            keys,
            vec![
                "crate",
                "crate::inline",
                "crate::inline::nested",
            ]
        );
    }

    #[test]
    fn resolves_nested_custom_path() {
        let path = Path::new("src/tests/rust/test_data/mods/nested_custom/src/lib.rs");
        let map = build_module_map(path).unwrap();
        let keys = map_keys(&map);
        assert_eq!(
            keys,
            vec![
                "crate",
                "crate::nested",
                "crate::nested::deep_mod",
            ]
        );
    }

    #[test]
    fn gracefully_handles_missing_file() {
        let path = Path::new("src/tests/rust/test_data/mods/missing/src/lib.rs");
        let map = build_module_map(path).unwrap();
        let keys = map_keys(&map);
        assert_eq!(keys, vec!["crate"]);
    }
}
