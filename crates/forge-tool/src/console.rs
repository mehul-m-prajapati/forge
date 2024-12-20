use forge_tool_macros::Description;
use ignore::WalkBuilder;
use inquire::autocompletion::Replacement;
use inquire::Autocomplete;
use serde::Serialize;

use crate::model::Prompt;
use crate::prompt_parser::{PromptParser, Token};
use crate::{Description, ToolTrait};

/// Read a line from the console
#[derive(Serialize, Description)]
pub(crate) struct ReadLine;

/// Write a line to the console
#[derive(Serialize)]
pub(crate) struct WriteLine;

#[derive(Clone)]
struct Completion {
    suggestions: Vec<String>,
}

impl Completion {
    pub fn new(suggestions: Vec<String>) -> Self {
        Self { suggestions }
    }
}

impl Autocomplete for Completion {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        // Performs a case-insensitive substring search on the suggestions.
        let input = input.trim().to_lowercase();
        let suggestions = if input.is_empty() {
            Vec::new()
        } else {
            self.suggestions
                .iter()
                .filter(|c| match &input {
                    s if s.starts_with("@") => input
                        .split("@")
                        .last()
                        .filter(|file| !file.contains("@") && !file.is_empty())
                        .is_some_and(|file| c.to_lowercase().contains(file)),
                    _ => false,
                })
                .cloned()
                .collect()
        };

        Ok(suggestions)
    }

    fn get_completion(
        &mut self,
        _: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        Ok(Replacement::from(highlighted_suggestion))
    }
}

#[async_trait::async_trait]
impl ToolTrait for ReadLine {
    type Input = Option<String>;
    type Output = Prompt;

    fn description(&self) -> String {
        "Read a line from the console".to_string()
    }

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        // TODO: improve the file listing logic not to execute on each call.
        let suggestions = ls_files(std::path::Path::new("."))
            .map(|v| v.into_iter().map(|a| format!("@{}", a)).collect::<Vec<_>>())
            .unwrap_or_default();

        let input = input.unwrap_or_default();
        let prompt = inquire::Text::new(&input)
            .with_autocomplete(Completion::new(suggestions))
            .prompt()
            .map_err(|e| e.to_string())?;

        Prompt::parse(prompt).await
    }
}

fn ls_files(path: &std::path::Path) -> std::io::Result<Vec<String>> {
    let mut paths = Vec::new();
    let walker = WalkBuilder::new(path)
        .hidden(true) // Skip hidden files
        .git_global(true) // Use global gitignore
        .git_ignore(true) // Use local .gitignore
        .ignore(true) // Use .ignore files
        .build();

    for result in walker {
        if let Ok(entry) = result {
            if entry.file_type().is_some_and(|ft| ft.is_file()) {
                paths.push(entry.path().display().to_string());
            }
        }
    }

    Ok(paths)
}

#[async_trait::async_trait]
impl ToolTrait for WriteLine {
    type Input = String;
    type Output = ();

    fn description(&self) -> String {
        "Write a line to the console".to_string()
    }

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        println!("{}", input);
        Ok(())
    }
}
