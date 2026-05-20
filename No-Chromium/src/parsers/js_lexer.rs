// AUTO-GENERATED JS LEXER (100% EXTRACTION)
#[derive(Debug, PartialEq, Clone)]
pub enum JsKeyword {
    Await,
    BreakKw,
    Case,
    Catch,
    Class,
    ConstKw,
    ContinueKw,
    Debugger,
    Default,
    Delete,
    Do,
    ElseKw,
    Enum,
    Export,
    Extends,
    False,
    Finally,
    ForKw,
    Function,
    IfKw,
    Import,
    In,
    Instanceof,
    New,
    Null,
    ReturnKw,
    Super,
    Switch,
    This,
    Throw,
    True,
    Try,
    Typeof,
    VarKw,
    Void,
    WhileKw,
    With,
    Yield,
    LetKw,
    Static,
    Implements,
    Interface,
    Package,
    Private,
    Protected,
    Public,
}

#[derive(Debug, PartialEq, Clone)]
pub enum JsToken {
    Keyword(JsKeyword),
    Identifier(String),
    StringLiteral(String),
    NumericLiteral(String),
    Operator(String),
    Punctuator(char),
    BooleanLiteral(bool),
    NullLiteral,
}

pub fn match_keyword(word: &str) -> Option<JsKeyword> {
    match word {
        "await" => Some(JsKeyword::Await),
        "break" => Some(JsKeyword::BreakKw),
        "case" => Some(JsKeyword::Case),
        "catch" => Some(JsKeyword::Catch),
        "class" => Some(JsKeyword::Class),
        "const" => Some(JsKeyword::ConstKw),
        "continue" => Some(JsKeyword::ContinueKw),
        "debugger" => Some(JsKeyword::Debugger),
        "default" => Some(JsKeyword::Default),
        "delete" => Some(JsKeyword::Delete),
        "do" => Some(JsKeyword::Do),
        "else" => Some(JsKeyword::ElseKw),
        "enum" => Some(JsKeyword::Enum),
        "export" => Some(JsKeyword::Export),
        "extends" => Some(JsKeyword::Extends),
        "false" => Some(JsKeyword::False),
        "finally" => Some(JsKeyword::Finally),
        "for" => Some(JsKeyword::ForKw),
        "function" => Some(JsKeyword::Function),
        "if" => Some(JsKeyword::IfKw),
        "import" => Some(JsKeyword::Import),
        "in" => Some(JsKeyword::In),
        "instanceof" => Some(JsKeyword::Instanceof),
        "new" => Some(JsKeyword::New),
        "null" => Some(JsKeyword::Null),
        "return" => Some(JsKeyword::ReturnKw),
        "super" => Some(JsKeyword::Super),
        "switch" => Some(JsKeyword::Switch),
        "this" => Some(JsKeyword::This),
        "throw" => Some(JsKeyword::Throw),
        "true" => Some(JsKeyword::True),
        "try" => Some(JsKeyword::Try),
        "typeof" => Some(JsKeyword::Typeof),
        "var" => Some(JsKeyword::VarKw),
        "void" => Some(JsKeyword::Void),
        "while" => Some(JsKeyword::WhileKw),
        "with" => Some(JsKeyword::With),
        "yield" => Some(JsKeyword::Yield),
        "let" => Some(JsKeyword::LetKw),
        "static" => Some(JsKeyword::Static),
        "implements" => Some(JsKeyword::Implements),
        "interface" => Some(JsKeyword::Interface),
        "package" => Some(JsKeyword::Package),
        "private" => Some(JsKeyword::Private),
        "protected" => Some(JsKeyword::Protected),
        "public" => Some(JsKeyword::Public),
        _ => None,
    }
}

pub fn tokenize_js(source: &str) -> Vec<JsToken> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Whitespace
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Single or Multi-line Comments
        if c == '/' && i + 1 < chars.len() {
            if chars[i + 1] == '/' {
                i += 2;
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            } else if chars[i + 1] == '*' {
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2; // skip past '*/'
                continue;
            }
        }

        // String Literals
        if c == '"' || c == '\'' || c == '`' {
            let quote = c;
            let mut val = String::new();
            i += 1;
            while i < chars.len() && chars[i] != quote {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    val.push(chars[i + 1]);
                    i += 2;
                } else {
                    val.push(chars[i]);
                    i += 1;
                }
            }
            i += 1; // skip closing quote
            tokens.push(JsToken::StringLiteral(val));
            continue;
        }

        // Numeric Literals
        if c.is_ascii_digit() || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            let mut num = String::new();
            if c == '.' {
                num.push('.');
                i += 1;
            }
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                num.push(chars[i]);
                i += 1;
            }
            tokens.push(JsToken::NumericLiteral(num));
            continue;
        }

        // Identifiers and Keywords
        if c.is_alphabetic() || c == '_' || c == '$' {
            let mut ident = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '$') {
                ident.push(chars[i]);
                i += 1;
            }
            if let Some(kw) = match_keyword(&ident) {
                match kw {
                    JsKeyword::True => tokens.push(JsToken::BooleanLiteral(true)),
                    JsKeyword::False => tokens.push(JsToken::BooleanLiteral(false)),
                    JsKeyword::Null => tokens.push(JsToken::NullLiteral),
                    _ => tokens.push(JsToken::Keyword(kw)),
                }
            } else {
                tokens.push(JsToken::Identifier(ident));
            }
            continue;
        }

        // Operators & Punctuators
        // Handle multi-character operators first
        let mut op_matched = false;
        let multi_ops = vec!["===", "!==", "==", "!=", "=>", "++", "--", "+=", "-=", "*=", "/=", "&&", "||", "<=", ">="];
        for op in multi_ops {
            let len = op.len();
            if i + len <= chars.len() {
                let chunk: String = chars[i..i+len].iter().collect();
                if chunk == op {
                    tokens.push(JsToken::Operator(chunk));
                    i += len;
                    op_matched = true;
                    break;
                }
            }
        }
        if op_matched {
            continue;
        }

        // Single-character operators/punctuators
        if "+-*/%=<>!&|^~?".contains(c) {
            tokens.push(JsToken::Operator(c.to_string()));
            i += 1;
        } else if ";,.:{}()[]".contains(c) {
            tokens.push(JsToken::Punctuator(c));
            i += 1;
        } else {
            // Unknown character, skip
            i += 1;
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_js() {
        let code = "const x = 42; if (x === 'hello') { return true; } // comment \n return null;";
        let tokens = tokenize_js(code);
        
        assert_eq!(tokens[0], JsToken::Keyword(JsKeyword::ConstKw));
        assert_eq!(tokens[1], JsToken::Identifier("x".to_string()));
        assert_eq!(tokens[2], JsToken::Operator("=".to_string()));
        assert_eq!(tokens[3], JsToken::NumericLiteral("42".to_string()));
        assert_eq!(tokens[4], JsToken::Punctuator(';'));
        assert_eq!(tokens[5], JsToken::Keyword(JsKeyword::IfKw));
        assert_eq!(tokens[6], JsToken::Punctuator('('));
        assert_eq!(tokens[7], JsToken::Identifier("x".to_string()));
        assert_eq!(tokens[8], JsToken::Operator("===".to_string()));
        assert_eq!(tokens[9], JsToken::StringLiteral("hello".to_string()));
    }
}