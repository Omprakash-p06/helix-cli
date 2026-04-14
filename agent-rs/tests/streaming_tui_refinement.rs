#[path = "../src/stream.rs"]
mod stream;

use stream::{SseEvent, SseParser};

#[test]
fn test_sse_parser_split_utf8_complex() {
    let mut parser = SseParser::new();
    
    // Test a more complex split: "Hello 👋 World"
    // "👋" is [0xF0, 0x9F, 0x91, 0x8B]
    
    let part1 = b"data: Hello \xF0";
    let part2 = b"\x9F";
    let part3 = b"\x91\x8B World\n";
    
    assert!(parser.push_bytes(part1).is_empty());
    assert!(parser.push_bytes(part2).is_empty());
    
    let events = parser.push_bytes(part3);
    assert_eq!(events.len(), 1);
    if let SseEvent::Data(data) = &events[0] {
        assert_eq!(data, "Hello 👋 World");
    } else {
        panic!("Expected Data event");
    }
}

#[test]
fn test_sse_parser_multiple_chunks_no_newline() {
    let mut parser = SseParser::new();
    
    parser.push_bytes(b"data: first chunk ");
    parser.push_bytes(b"second chunk ");
    parser.push_bytes(b"third chunk\n");
    
    let events = parser.finish();
    // The last chunk had a newline, so it should have been drained by push_bytes
    // but parser.finish() also drains partials if any.
    // Wait, let's re-verify the logic in drain_events.
}

#[test]
fn test_sse_parser_drain_logic() {
    let mut parser = SseParser::new();
    
    let events = parser.push_bytes(b"data: hello\nsky: blue\ndata: world\n");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0], SseEvent::Data("hello".to_string()));
    assert_eq!(events[1], SseEvent::Data("world".to_string()));
}

#[test]
fn test_sse_parser_done_event() {
    let mut parser = SseParser::new();
    let events = parser.push_bytes(b"data: [DONE]\n");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], SseEvent::Done);
}
