#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SseEvent {
    Data(String),
    Done,
}

#[derive(Debug, Default)]
pub struct SseParser {
    buffer: String,
}

impl SseParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) -> Vec<SseEvent> {
        self.buffer.push_str(&String::from_utf8_lossy(bytes));
        self.drain_events(false)
    }

    pub fn finish(&mut self) -> Vec<SseEvent> {
        self.drain_events(true)
    }

    fn drain_events(&mut self, flush_partial: bool) -> Vec<SseEvent> {
        let mut events = Vec::new();

        while let Some(newline_idx) = self.buffer.find('\n') {
            let mut line = self.buffer[..newline_idx].to_string();
            if line.ends_with('\r') {
                line.pop();
            }

            self.buffer.drain(..(newline_idx + 1));
            if let Some(event) = parse_data_line(&line) {
                events.push(event);
            }
        }

        if flush_partial && !self.buffer.is_empty() {
            let line = std::mem::take(&mut self.buffer);
            if let Some(event) = parse_data_line(line.trim_end_matches('\r')) {
                events.push(event);
            }
        }

        events
    }
}

fn parse_data_line(line: &str) -> Option<SseEvent> {
    let data = line.strip_prefix("data:")?.trim_start();

    if data == "[DONE]" {
        return Some(SseEvent::Done);
    }

    Some(SseEvent::Data(data.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{SseEvent, SseParser};

    #[test]
    fn emits_data_for_fragmented_line() {
        let mut parser = SseParser::new();
        let part1 = br#"data: {"choices":[{"delta":{"content":"hel"}}]"#;
        let part2 = br#"}
"#;

        assert!(parser.push_bytes(part1).is_empty());

        let events = parser.push_bytes(part2);
        assert_eq!(
            events,
            vec![SseEvent::Data(
                r#"{"choices":[{"delta":{"content":"hel"}}]}"#.to_string()
            )]
        );
    }

    #[test]
    fn emits_multiple_data_lines_in_order() {
        let mut parser = SseParser::new();
        let payload = b"data: one\ndata: two\n";

        let events = parser.push_bytes(payload);
        assert_eq!(
            events,
            vec![
                SseEvent::Data("one".to_string()),
                SseEvent::Data("two".to_string())
            ]
        );
    }

    #[test]
    fn emits_done_without_corrupting_buffer() {
        let mut parser = SseParser::new();
        let part1 = b"data: [DONE]\n";
        let part2 = b"data: ok\n";

        let first = parser.push_bytes(part1);
        let second = parser.push_bytes(part2);

        assert_eq!(first, vec![SseEvent::Done]);
        assert_eq!(second, vec![SseEvent::Data("ok".to_string())]);
    }

    #[test]
    fn handles_crlf_and_lf() {
        let mut parser = SseParser::new();
        let payload = b"data: a\r\ndata: b\n";

        let events = parser.push_bytes(payload);
        assert_eq!(
            events,
            vec![
                SseEvent::Data("a".to_string()),
                SseEvent::Data("b".to_string())
            ]
        );
    }

    #[test]
    fn flushes_partial_line_on_finish() {
        let mut parser = SseParser::new();
        let payload = b"data: tail-without-newline";

        assert!(parser.push_bytes(payload).is_empty());
        assert_eq!(
            parser.finish(),
            vec![SseEvent::Data("tail-without-newline".to_string())]
        );
    }
}
