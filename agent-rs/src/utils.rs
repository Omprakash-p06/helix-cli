#[derive(Debug, Clone)]
struct ProtectedBlock {
    placeholder: String,
    content: String,
}

fn protect_fenced_code(input: &str, blocks: &mut Vec<ProtectedBlock>, idx: &mut usize) -> String {
    let mut out = String::new();
    let mut remaining = input;

    while let Some(start) = remaining.find("```") {
        out.push_str(&remaining[..start]);
        let after = &remaining[start + 3..];
        if let Some(end_rel) = after.find("```") {
            let end = start + 3 + end_rel + 3;
            let block = &remaining[start..end];
            let placeholder = format!("__PROTECTED_BLOCK_{}__", *idx);
            *idx += 1;
            blocks.push(ProtectedBlock {
                placeholder: placeholder.clone(),
                content: block.to_string(),
            });
            out.push_str(&placeholder);
            remaining = &remaining[end..];
        } else {
            out.push_str(&remaining[start..]);
            return out;
        }
    }

    out.push_str(remaining);
    out
}

fn protect_inline_code(input: &str, blocks: &mut Vec<ProtectedBlock>, idx: &mut usize) -> String {
    let mut out = String::new();
    let mut remaining = input;

    while let Some(start) = remaining.find('`') {
        out.push_str(&remaining[..start]);
        let after = &remaining[start + 1..];
        if let Some(end_rel) = after.find('`') {
            let end = start + 1 + end_rel + 1;
            let block = &remaining[start..end];
            let placeholder = format!("__PROTECTED_BLOCK_{}__", *idx);
            *idx += 1;
            blocks.push(ProtectedBlock {
                placeholder: placeholder.clone(),
                content: block.to_string(),
            });
            out.push_str(&placeholder);
            remaining = &remaining[end..];
        } else {
            out.push_str(&remaining[start..]);
            return out;
        }
    }

    out.push_str(remaining);
    out
}

fn protect_tool_json(input: &str, blocks: &mut Vec<ProtectedBlock>, idx: &mut usize) -> String {
    let bytes = input.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    let mut last = 0;

    while i < bytes.len() {
        if bytes[i] == b'{' {
            let start = i;
            let mut depth = 1;
            i += 1;
            while i < bytes.len() && depth > 0 {
                match bytes[i] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }
            if depth == 0 {
                let candidate = &input[start..i];
                let is_tool_json = candidate.contains("\"tool_calls\"")
                    || (candidate.contains("\"name\"") && candidate.contains("\"arguments\""));
                if is_tool_json {
                    out.push_str(&input[last..start]);
                    let placeholder = format!("__PROTECTED_BLOCK_{}__", *idx);
                    *idx += 1;
                    blocks.push(ProtectedBlock {
                        placeholder: placeholder.clone(),
                        content: candidate.to_string(),
                    });
                    out.push_str(&placeholder);
                    last = i;
                }
                continue;
            }
        }
        i += 1;
    }

    out.push_str(&input[last..]);
    out
}

fn restore_blocks(input: &str, blocks: &[ProtectedBlock]) -> String {
    let mut restored = input.to_string();
    for block in blocks {
        restored = restored.replace(&block.placeholder, &block.content);
    }
    restored
}

fn strip_block_family(input: &str, open: &str, close: &str) -> String {
    let mut out = String::new();
    let mut cursor = 0;

    while let Some(start_rel) = input[cursor..].find(open) {
        let start = cursor + start_rel;
        out.push_str(&input[cursor..start]);
        let after_open = start + open.len();
        if let Some(end_rel) = input[after_open..].find(close) {
            cursor = after_open + end_rel + close.len();
        } else {
            out.push_str(&input[start..]);
            cursor = input.len();
        }
    }

    out.push_str(&input[cursor..]);
    out
}

fn strip_malformed_openers(input: &str, marker: &str) -> String {
    let mut out = String::new();
    let mut i = 0;
    let bytes = input.as_bytes();

    while i < bytes.len() {
        if input[i..].starts_with(marker) {
            let end_tag = input[i..].find('>').map(|p| i + p + 1);
            if let Some(end) = end_tag {
                i = end;
            } else {
                let end_line = input[i..].find('\n').map(|p| i + p).unwrap_or(bytes.len());
                i = end_line;
            }
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }

    out
}

pub fn strip_reasoning_blocks(text: &str) -> String {
    let mut out = text.to_string();

    out = strip_block_family(&out, "<think", "</think>");
    out = strip_block_family(&out, "<thinking", "</thinking>");
    out = strip_block_family(&out, "<analysis", "</analysis>");

    out = strip_malformed_openers(&out, "<think");
    out = strip_malformed_openers(&out, "<thinking");
    out = strip_malformed_openers(&out, "<analysis");

    out = out.replace("</think>", "");
    out = out.replace("</thinking>", "");
    out = out.replace("</analysis>", "");

    out
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '.' | '!' | '?') {
            sentences.push(current.clone());
            current.clear();
        }
    }

    if !current.is_empty() {
        sentences.push(current);
    }

    sentences
}

pub fn deduplicate_consecutive_sentences(text: &str) -> String {
    let mut out = String::new();
    let mut prev: Option<String> = None;

    for sentence in split_sentences(text) {
        let normalized = sentence.trim().to_string();
        if prev.as_ref().map(|p| p == &normalized).unwrap_or(false) {
            continue;
        }
        out.push_str(&sentence);
        prev = Some(normalized);
    }

    out
}

pub fn normalize_quotes(text: &str) -> String {
    let mut out = text
        .replace('“', "\"")
        .replace('”', "\"")
        .replace('‘', "'")
        .replace('’', "'");

    let double_quotes = out.chars().filter(|c| *c == '"').count();
    if double_quotes % 2 == 1 {
        out.push('"');
    }

    out
}

pub fn clean_chat_output(text: &str) -> String {
    let mut blocks = Vec::new();
    let mut idx = 0;

    let protected_fenced = protect_fenced_code(text, &mut blocks, &mut idx);
    let protected_inline = protect_inline_code(&protected_fenced, &mut blocks, &mut idx);
    let protected = protect_tool_json(&protected_inline, &mut blocks, &mut idx);

    let stripped = strip_reasoning_blocks(&protected);
    let deduped = deduplicate_consecutive_sentences(&stripped);
    let normalized = normalize_quotes(&deduped);

    restore_blocks(&normalized, &blocks).trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::{clean_chat_output, deduplicate_consecutive_sentences, normalize_quotes, strip_reasoning_blocks};

    #[test]
    fn strips_think_family_blocks() {
        let input = "hello <think>internal</think> world <analysis>meta</analysis> done";
        assert_eq!(strip_reasoning_blocks(input), "hello  world  done");
    }

    #[test]
    fn strips_malformed_reasoning_markers() {
        let input = "ok <think this broke\nstill visible";
        assert_eq!(strip_reasoning_blocks(input), "ok \nstill visible");
    }

    #[test]
    fn deduplicates_exact_consecutive_sentences_only() {
        let input = "Hello. Hello. hello. Hello.";
        assert_eq!(deduplicate_consecutive_sentences(input), "Hello. hello. Hello.");
    }

    #[test]
    fn normalizes_curly_quotes() {
        let input = "“Hey” and ‘hi’";
        assert_eq!(normalize_quotes(input), "\"Hey\" and 'hi'");
    }

    #[test]
    fn preserves_code_and_tool_json_while_cleaning() {
        let input = "<think>secret</think>```\nconst x = \"hello\";\n``` and `code` {\"name\":\"read_file\",\"arguments\":{\"path\":\"a\"}}";
        let output = clean_chat_output(input);
        assert!(output.contains("```\nconst x = \"hello\";\n```"));
        assert!(output.contains("`code`"));
        assert!(output.contains("{\"name\":\"read_file\",\"arguments\":{\"path\":\"a\"}}"));
        assert!(!output.contains("secret"));
    }
}
