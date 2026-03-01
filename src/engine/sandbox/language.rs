use std::path::{Path, PathBuf};

use crate::engine::models::Language;

#[derive(Debug, Clone)]
pub struct LanguageSpec {
    pub source_name: &'static str,
    pub docker_image: &'static str,
    pub docker_script: &'static str,
    pub process_interpreted_cmd: Option<&'static str>,
    pub process_compile_cmd: Option<&'static str>,
}

impl LanguageSpec {
    pub fn for_language(language: &Language) -> Self {
        match language {
            Language::Python => Self {
                source_name: "main.py",
                docker_image: "python:3.12-alpine",
                docker_script: "python3 -I /workspace/main.py \"$@\"",
                process_interpreted_cmd: Some("python"),
                process_compile_cmd: None,
            },
            Language::JavaScript => Self {
                source_name: "main.js",
                docker_image: "node:22-alpine",
                docker_script: "node /workspace/main.js \"$@\"",
                process_interpreted_cmd: Some("node"),
                process_compile_cmd: None,
            },
            Language::Rust => Self {
                source_name: "main.rs",
                docker_image: "rust:1.76-alpine",
                docker_script: "rustc /workspace/main.rs -O -o /tmp/app && /tmp/app \"$@\"",
                process_interpreted_cmd: None,
                process_compile_cmd: Some("rustc"),
            },
            Language::C => Self {
                source_name: "main.c",
                docker_image: "gcc:14",
                docker_script: "gcc /workspace/main.c -O2 -o /tmp/app && /tmp/app \"$@\"",
                process_interpreted_cmd: None,
                process_compile_cmd: Some("gcc"),
            },
        }
    }

    pub fn source_path(&self, work_dir: &Path) -> PathBuf {
        work_dir.join(self.source_name)
    }
}
