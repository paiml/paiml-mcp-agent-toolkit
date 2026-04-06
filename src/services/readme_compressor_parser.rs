// Markdown parsing methods for ReadmeCompressor
// Included by readme_compressor.rs — no `use` imports or `#!` attributes allowed

impl ReadmeCompressor {
    fn handle_heading(
        &self,
        level: u8,
        current_section: &mut Option<Section>,
        sections: &mut Vec<Section>,
        text_buffer: &mut String,
    ) {
        // Save previous section if exists
        if let Some(mut section) = current_section.take() {
            if !text_buffer.is_empty() {
                section.paragraphs.push(text_buffer.clone());
                text_buffer.clear();
            }
            sections.push(section);
        }
        *current_section = Some(Section {
            title: String::new(),
            level,
            paragraphs: Vec::new(),
            lists: Vec::new(),
            code_snippets: Vec::new(),
        });
    }

    fn handle_text(
        &self,
        text: &str,
        current_section: &mut Option<Section>,
        in_list: bool,
        list_items: &mut Vec<String>,
        in_code_block: bool,
        text_buffer: &mut String,
    ) {
        if let Some(ref mut section) = current_section {
            if section.title.is_empty() {
                section.title = text.to_string();
            } else if in_list {
                list_items.push(text.to_string());
            } else if !in_code_block {
                text_buffer.push_str(text);
            }
        }
    }

    fn handle_list_end(&self, current_section: &mut Option<Section>, list_items: &mut Vec<String>) {
        if let Some(ref mut section) = current_section {
            if !list_items.is_empty() {
                section.lists.push(List {
                    items: list_items.clone(),
                });
                list_items.clear();
            }
        }
    }

    fn handle_paragraph_end(
        &self,
        current_section: &mut Option<Section>,
        text_buffer: &mut String,
    ) {
        if let Some(ref mut section) = current_section {
            if !text_buffer.is_empty() {
                section.paragraphs.push(text_buffer.clone());
                text_buffer.clear();
            }
        }
    }

    fn parse_markdown_sections(&self, content: &str) -> Vec<Section> {
        let parser = Parser::new(content);
        let mut sections = Vec::new();
        let mut current_section: Option<Section> = None;
        let mut in_list = false;
        let mut list_items = Vec::new();
        let mut in_code_block = false;
        let mut text_buffer = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    self.handle_heading(
                        level as u8,
                        &mut current_section,
                        &mut sections,
                        &mut text_buffer,
                    );
                }
                Event::Text(text) => {
                    self.handle_text(
                        &text,
                        &mut current_section,
                        in_list,
                        &mut list_items,
                        in_code_block,
                        &mut text_buffer,
                    );
                }
                Event::Start(Tag::List(_)) => {
                    in_list = true;
                    list_items.clear();
                }
                Event::End(TagEnd::List(_)) => {
                    in_list = false;
                    self.handle_list_end(&mut current_section, &mut list_items);
                }
                Event::Start(Tag::CodeBlock(_)) => {
                    in_code_block = true;
                }
                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;
                }
                Event::SoftBreak | Event::HardBreak => {
                    text_buffer.push(' ');
                }
                Event::End(TagEnd::Paragraph) => {
                    self.handle_paragraph_end(&mut current_section, &mut text_buffer);
                }
                _ => {}
            }
        }

        // Save last section
        if let Some(mut section) = current_section {
            if !text_buffer.is_empty() {
                section.paragraphs.push(text_buffer);
            }
            sections.push(section);
        }

        sections
    }
}
