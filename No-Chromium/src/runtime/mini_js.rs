use crate::runtime::dom_runtime::{RuntimeDom, RuntimeReport};
use crate::runtime::script_collector::ScriptSource;

#[derive(Debug, Default)]
pub struct BrowserRuntime {
    dom: RuntimeDom,
    console_messages: Vec<String>,
    unsupported_statements: Vec<String>,
    inline_scripts_executed: usize,
    external_scripts_seen: Vec<String>,
}

impl BrowserRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute_scripts(mut self, scripts: &[ScriptSource]) -> RuntimeReport {
        for script in scripts {
            match script {
                ScriptSource::Inline(code) => {
                    self.execute_inline(code);
                    self.inline_scripts_executed += 1;
                }
                ScriptSource::External(url) => {
                    self.external_scripts_seen.push(url.clone());
                }
            }
        }

        RuntimeReport {
            dom: self.dom,
            console_messages: self.console_messages,
            inline_scripts_executed: self.inline_scripts_executed,
            external_scripts_seen: self.external_scripts_seen,
            unsupported_statements: self.unsupported_statements,
        }
    }

    fn execute_inline(&mut self, code: &str) {
        for statement in split_statements(code) {
            let statement = statement.trim();
            if statement.is_empty() {
                continue;
            }

            if self.try_console_log(statement)
                || self.try_document_assignment(statement)
                || self.try_body_append(statement)
            {
                continue;
            }

            self.unsupported_statements.push(statement.chars().take(160).collect());
        }
    }

    fn try_console_log(&mut self, statement: &str) -> bool {
        let Some(argument) = call_argument(statement, "console.log") else {
            return false;
        };

        self.console_messages.push(argument);
        true
    }

    fn try_document_assignment(&mut self, statement: &str) -> bool {
        if let Some(value) = assignment_value(statement, "document.title") {
            self.dom.set_title(value);
            return true;
        }

        if let Some(value) = assignment_value(statement, "document.body.textContent")
            .or_else(|| assignment_value(statement, "document.body.innerText"))
            .or_else(|| assignment_value(statement, "document.body.innerHTML"))
        {
            self.dom.set_body_text(strip_htmlish(&value));
            return true;
        }

        if let Some(value) = assignment_value(statement, "document.querySelector(\"title\").textContent")
            .or_else(|| assignment_value(statement, "document.querySelector('title').textContent"))
        {
            self.dom.set_title(value);
            return true;
        }

        false
    }

    fn try_body_append(&mut self, statement: &str) -> bool {
        if let Some(value) = nested_text_node_argument(statement) {
            self.dom.append_text(value);
            return true;
        }

        if let Some(value) = call_argument(statement, "document.body.append")
            .or_else(|| call_argument(statement, "document.body.appendChild"))
        {
            self.dom.append_text(strip_htmlish(&value));
            return true;
        }

        false
    }
}

fn split_statements(code: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for ch in code.chars() {
        current.push(ch);

        if escaped {
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        match quote {
            Some(q) if ch == q => quote = None,
            Some(_) => {}
            None if ch == '"' || ch == '\'' || ch == '`' => quote = Some(ch),
            None if ch == ';' || ch == '\n' => {
                statements.push(current.trim_end_matches([';', '\n']).trim().to_string());
                current.clear();
            }
            None => {}
        }
    }

    if !current.trim().is_empty() {
        statements.push(current.trim().to_string());
    }

    statements
}

fn call_argument(statement: &str, callee: &str) -> Option<String> {
    let rest = statement.trim().strip_prefix(callee)?.trim_start();
    let inner = rest.strip_prefix('(')?.trim_end().strip_suffix(')')?.trim();
    parse_js_string(inner)
}

fn assignment_value(statement: &str, target: &str) -> Option<String> {
    let rest = statement.trim().strip_prefix(target)?.trim_start();
    let value = rest.strip_prefix('=')?.trim();
    parse_js_string(value)
}

fn nested_text_node_argument(statement: &str) -> Option<String> {
    let inner = statement
        .trim()
        .strip_prefix("document.body.appendChild")?
        .trim_start()
        .strip_prefix('(')?
        .trim_end()
        .strip_suffix(')')?
        .trim();

    call_argument(inner, "document.createTextNode")
}

fn parse_js_string(value: &str) -> Option<String> {
    let value = value.trim().trim_end_matches(';').trim();
    let quote = value.chars().next()?;
    if quote != '"' && quote != '\'' && quote != '`' {
        return Some(value.to_string());
    }

    let mut out = String::new();
    let mut chars = value[quote.len_utf8()..].chars();
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if escaped {
            out.push(match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '\\' => '\\',
                '"' => '"',
                '\'' => '\'',
                '`' => '`',
                other => other,
            });
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == quote {
            return Some(out);
        }

        out.push(ch);
    }

    None
}

fn strip_htmlish(value: &str) -> String {
    let mut out = String::new();
    let mut inside_tag = false;

    for ch in value.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => out.push(ch),
            _ => {}
        }
    }

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
