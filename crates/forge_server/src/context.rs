use std::io::Cursor;

use forge_provider::Request;
use pulldown_cmark::{html, Options, Parser};

pub struct ContextEngine {
    context: Request,
}

impl ContextEngine {
    pub fn new(context: Request) -> Self {
        Self { context }
    }

    // TODO: It should render HTML based on OpenRouter context
    pub fn render_html(&self) -> String {
        // Convert context to markdown format
        let mut markdown = String::new();
        for msg in &self.context.messages {
            let role = match msg {
                forge_provider::CompletionMessage::ContentMessage(msg) => msg.role.to_string(),
                forge_provider::CompletionMessage::ToolMessage(_) => "tool".to_string(),
            };

            markdown.push_str(&format!("# [:{}]\n{}\n", role, msg.content()));
        }

        // Convert markdown to HTML with basic styling
        let parser = Parser::new_ext(&markdown, Options::all());
        let mut bytes = Vec::new();

        html::write_html(Cursor::new(&mut bytes), parser).unwrap();
        let html_output = String::from_utf8(bytes).unwrap();
        include_str!("./context.html").replace("<!-- CURRENT_CONTEXT_INFORMATION -->", &html_output)
    }
}
