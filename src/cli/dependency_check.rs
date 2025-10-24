use colored::Colorize;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use tabled::{Table, Tabled, settings::Style};

#[derive(Debug, Clone, Copy, PartialEq)]
enum DependencyStatus {
    Installed,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DependencyType {
    Required,
    #[allow(dead_code)]
    Optional,
}

#[derive(Tabled)]
struct DependencyRow {
    #[tabled(rename = "Dependency")]
    name: String,
    #[tabled(rename = "Status")]
    status: String, // Plain text, no ANSI codes
    #[tabled(rename = "Description")]
    description: String,
}

struct Dependency {
    name: &'static str,
    check_command: &'static str,
    check_args: &'static [&'static str],
    dep_type: DependencyType,
    description: &'static str,
}

impl Dependency {
    fn check_status(&self) -> DependencyStatus {
        Command::new(self.check_command)
            .args(self.check_args)
            .output()
            .map(|output| {
                if output.status.success() {
                    DependencyStatus::Installed
                } else {
                    DependencyStatus::Missing
                }
            })
            .unwrap_or(DependencyStatus::Missing)
    }

    fn to_row(&self, status: DependencyStatus) -> DependencyRow {
        // Store plain text without ANSI codes
        let status_text = match (status, self.dep_type) {
            (DependencyStatus::Installed, _) => "✓ OK",
            (DependencyStatus::Missing, DependencyType::Required) => "✗ MISSING",
            (DependencyStatus::Missing, DependencyType::Optional) => "- MISSING",
        };

        DependencyRow {
            name: self.name.to_string(),
            status: status_text.to_string(),
            description: self.description.to_string(),
        }
    }
}

fn check_file_exists(path: &str) -> DependencyStatus {
    let expanded_path = if path.starts_with("~/") {
        if let Some(home) = env::var("HOME").ok() {
            PathBuf::from(home).join(&path[2..])
        } else {
            PathBuf::from(path)
        }
    } else {
        PathBuf::from(path)
    };

    if expanded_path.exists() {
        DependencyStatus::Installed
    } else {
        DependencyStatus::Missing
    }
}

fn check_command(cmd: &str, args: &[&str]) -> DependencyStatus {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|output| {
            if output.status.success() {
                DependencyStatus::Installed
            } else {
                DependencyStatus::Missing
            }
        })
        .unwrap_or(DependencyStatus::Missing)
}

fn format_optional_status(status: DependencyStatus) -> String {
    match status {
        DependencyStatus::Installed => "✓ OK".to_string(),
        DependencyStatus::Missing => "- MISSING".to_string(),
    }
}

// Helper function to colorize table output
fn colorize_table_output(table_str: &str) -> String {
    table_str
        .lines()
        .map(|line| {
            if line.contains("✓ OK") {
                line.replace("✓ OK", &"✓ OK".green().to_string())
            } else if line.contains("✗ MISSING") {
                line.replace("✗ MISSING", &"✗ MISSING".red().to_string())
            } else if line.contains("- MISSING") {
                line.replace("- MISSING", &"- MISSING".yellow().to_string())
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn check_dependencies() {
    let dependencies = vec![
        Dependency {
            name: "fzf",
            check_command: "which",
            check_args: &["fzf"],
            dep_type: DependencyType::Required,
            description: "Fuzzy finder",
        },
        Dependency {
            name: "expect",
            check_command: "which",
            check_args: &["expect"],
            dep_type: DependencyType::Required,
            description: "Automation tool",
        },
        Dependency {
            name: "bat",
            check_command: "which",
            check_args: &["bat"],
            dep_type: DependencyType::Required,
            description: "File preview",
        },
        Dependency {
            name: "ripgrep",
            check_command: "which",
            check_args: &["rg"],
            dep_type: DependencyType::Required,
            description: "Search tool",
        },
    ];

    println!("\n{}\n", "FuzzyTestFinder Dependency Check".bold());

    let mut all_required_installed = true;
    let mut rows = Vec::new();

    // Check core dependencies
    for dep in &dependencies {
        let status = dep.check_status();
        if status == DependencyStatus::Missing && dep.dep_type == DependencyType::Required {
            all_required_installed = false;
        }
        rows.push(dep.to_row(status));
    }

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{}", colorize_table_output(&table.to_string()));

    // Python dependencies
    println!("\n{}", "Optional: Python Support".bold());
    let python_deps = vec![
        (
            "pytest",
            "pytest",
            &["--version"] as &[&str],
            "Test framework [Needed for python]",
        ),
        (
            "pytest-json-report",
            "pip3",
            &["show", "pytest-json-report"],
            "JSON reporting",
        ),
        (
            "pytest-cov",
            "pip3",
            &["show", "pytest-cov"],
            "Coverage support",
        ),
    ];

    let mut python_rows = Vec::new();
    for (name, cmd, args, desc) in python_deps {
        let status = check_command(cmd, args);
        python_rows.push(DependencyRow {
            name: name.to_string(),
            status: format_optional_status(status),
            description: desc.to_string(),
        });
    }

    let mut table = Table::new(python_rows);
    table.with(Style::rounded());
    println!("{}", colorize_table_output(&table.to_string()));

    // Rust coverage and runtime
    println!("\n{}", "Optional: Rust Coverage".bold());
    let cargo_status = check_command("cargo", &["--version"]);
    let tarpaulin_status = check_command("cargo", &["tarpaulin", "--version"]);
    let next_test_status = check_command("cargo", &["nextest", "--version"]);

    let rust_rows = vec![
        DependencyRow {
            name: "cargo".to_string(),
            status: format_optional_status(cargo_status),
            description: "Cargo build tool".to_string(),
        },
        DependencyRow {
            name: "cargo-tarpaulin".to_string(),
            status: format_optional_status(tarpaulin_status),
            description: "Rust coverage".to_string(),
        },
        DependencyRow {
            name: "cargo-nextest".to_string(),
            status: format_optional_status(next_test_status),
            description: "Test runner".to_string(),
        },
    ];

    let mut table = Table::new(rust_rows);
    table.with(Style::rounded());
    println!("{}", colorize_table_output(&table.to_string()));

    // Java support
    println!("\n{}", "Optional: Java Support".bold());
    let java_gradle_status = check_command("./gradlew", &["version"]);
    let java_parser_status = check_file_exists("~/.fzt/fzt-java-parser.jar");

    let java_rows = vec![
        DependencyRow {
            name: "Gradle".to_string(),
            status: format_optional_status(java_gradle_status),
            description: "Gradle Build Tool [Needed for Java]".to_string(),
        },
        DependencyRow {
            name: "fzt-java-parser.jar".to_string(),
            status: format_optional_status(java_parser_status),
            description: "Java test parser".to_string(),
        },
    ];

    let mut table = Table::new(java_rows);
    table.with(Style::rounded());
    println!("{}", colorize_table_output(&table.to_string()));

    // Configuration
    println!("\n{}", "Configuration".bold());
    let fzt_folder_status = check_file_exists("~/.fzt");
    let status_text = match fzt_folder_status {
        DependencyStatus::Installed => "✓ OK",
        DependencyStatus::Missing => {
            all_required_installed = false;
            "✗ MISSING"
        }
    };

    let config_rows = vec![DependencyRow {
        name: "~/.fzt/".to_string(),
        status: status_text.to_string(),
        description: "Config directory".to_string(),
    }];

    let mut table = Table::new(config_rows);
    table.with(Style::rounded());
    println!("{}", colorize_table_output(&table.to_string()));

    // Summary
    println!();
    if all_required_installed {
        println!("{}", "✓ All required dependencies are installed!".green());
    } else {
        println!(
            "{}",
            "✗ Some required dependencies are missing. Please install them.".red()
        );
    }

    println!("\n{}", "Legend:".bold());
    println!("  {} - Installed", "✓ OK".green());
    println!("  {} - Optional, not installed", "- MISSING".yellow());
    println!("  {} - Required, not installed\n", "✗ MISSING".red());
}
