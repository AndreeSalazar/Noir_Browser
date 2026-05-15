#[derive(Debug, Default, Clone)]
pub struct RuntimeDom {
    pub title: Option<String>,
    pub appended_text: Vec<String>,
}

impl RuntimeDom {
    pub fn set_title(&mut self, value: String) {
        self.title = Some(value);
    }

    pub fn set_body_text(&mut self, value: String) {
        self.appended_text.clear();
        self.appended_text.push(value);
    }

    pub fn append_text(&mut self, value: String) {
        self.appended_text.push(value);
    }
}

#[derive(Debug, Default, Clone)]
pub struct RuntimeReport {
    pub dom: RuntimeDom,
    pub console_messages: Vec<String>,
    pub inline_scripts_executed: usize,
    pub external_scripts_seen: Vec<String>,
    pub unsupported_statements: Vec<String>,
}
