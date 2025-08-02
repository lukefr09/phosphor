use phosphor_common::traits::{ControlEvent, ParsedEvent, TerminalParser};
use tracing::trace;

/// Basic parser for Phase 1 - handles plain text and basic control characters
pub struct BasicParser;

impl BasicParser {
    pub fn new() -> Self {
        Self
    }
}

impl TerminalParser for BasicParser {
    fn parse(&mut self, data: &[u8]) -> Vec<ParsedEvent> {
        let mut events = Vec::new();
        let text = String::from_utf8_lossy(data);
        
        for ch in text.chars() {
            trace!("Parsing character: {:?}", ch);
            match ch {
                '\n' => events.push(ParsedEvent::Control(ControlEvent::NewLine)),
                '\r' => events.push(ParsedEvent::Control(ControlEvent::CarriageReturn)),
                '\t' => events.push(ParsedEvent::Control(ControlEvent::Tab)),
                '\x08' => events.push(ParsedEvent::Control(ControlEvent::Backspace)),
                '\x0C' => events.push(ParsedEvent::Control(ControlEvent::Clear)),
                _ => {
                    // Accumulate printable characters
                    if let Some(ParsedEvent::Text(ref mut s)) = events.last_mut() {
                        s.push(ch);
                    } else {
                        events.push(ParsedEvent::Text(ch.to_string()));
                    }
                }
            }
        }
        
        events
    }
}

impl Default for BasicParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plain_text() {
        let mut parser = BasicParser::new();
        let events = parser.parse(b"Hello, World!");
        
        assert_eq!(events.len(), 1);
        match &events[0] {
            ParsedEvent::Text(s) => assert_eq!(s, "Hello, World!"),
            _ => panic!("Expected text event"),
        }
    }
    
    #[test]
    fn test_control_characters() {
        let mut parser = BasicParser::new();
        let events = parser.parse(b"Hello\nWorld\r\n");
        
        assert_eq!(events.len(), 4);
        assert!(matches!(events[0], ParsedEvent::Text(_)));
        assert!(matches!(events[1], ParsedEvent::Control(ControlEvent::NewLine)));
        assert!(matches!(events[2], ParsedEvent::Text(_)));
        assert!(matches!(events[3], ParsedEvent::Control(ControlEvent::CarriageReturn)));
    }
}