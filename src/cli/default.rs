use crate::{
    cache::manager::LocalCacheManager,
    errors::FztError,
    runner::{MetaData, RunnerName, config::Language},
};

pub fn get_default(project_id: &str) -> Result<Language, FztError> {
    let reader = LocalCacheManager::get_meta(project_id)?;
    let meta_data: MetaData = match reader {
        Some(reader) => serde_json::from_reader(reader)?,
        None => {
            return Err(FztError::GeneralParsingError(
                "Metadata not found. Did you initialize the project `fzt -d <LANGUAGE>` ?"
                    .to_string(),
            ));
        }
    };

    Ok(match meta_data.runner_name {
        RunnerName::RustPythonRunner => Language::Python {
            parser: "rustpython".to_string(),
            runtime: meta_data.runtime,
        },
        RunnerName::PytestRunner => Language::Python {
            parser: "pytest".to_string(),
            runtime: meta_data.runtime,
        },
        RunnerName::JavaJunit5Runner => Language::Java {
            test_framework: "junit5".to_string(),
            runtime: meta_data.runtime,
        },
        RunnerName::RustCargoRunner => Language::Rust {
            runtime: meta_data.runtime,
        },
        RunnerName::RustNextestRunner => Language::Rust {
            runtime: meta_data.runtime,
        },
    })
}
