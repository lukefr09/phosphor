use phosphor_common::traits::{
    ControlEvent, ParsedEvent, TerminalParser, CsiSequence, OscSequence, EscSequence,
    EraseMode, SgrParameter
};
use phosphor_common::types::Color;
use tracing::{trace, debug};
use vte::{Parser, Perform, Params};

/// VTE-based ANSI/VT parser for terminal escape sequences
pub struct VteParser {
    parser: Parser,
    performer: TerminalPerformer,
}

impl VteParser {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            performer: TerminalPerformer::new(),
        }
    }
    
    /// Get events that have been accumulated and clear the buffer
    pub fn take_events(&mut self) -> Vec<ParsedEvent> {
        std::mem::take(&mut self.performer.events)
    }
}

impl TerminalParser for VteParser {
    fn parse(&mut self, data: &[u8]) -> Vec<ParsedEvent> {
        // Clear previous events
        self.performer.events.clear();
        
        // Process each byte through VTE
        for &byte in data {
            self.parser.advance(&mut self.performer, byte);
        }
        
        // Flush any pending text
        self.performer.flush_text();
        
        // Take accumulated events
        self.take_events()
    }
}

impl Default for VteParser {
    fn default() -> Self {
        Self::new()
    }
}

/// VTE performer that translates VTE callbacks into ParsedEvents
struct TerminalPerformer {
    events: Vec<ParsedEvent>,
    current_text: String,
}

impl TerminalPerformer {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            current_text: String::new(),
        }
    }
    
    /// Flush any accumulated text as a Text event
    fn flush_text(&mut self) {
        if !self.current_text.is_empty() {
            let text = std::mem::take(&mut self.current_text);
            self.events.push(ParsedEvent::Text(text));
        }
    }
    
    /// Parse SGR (Select Graphic Rendition) parameters
    fn parse_sgr_params(&self, params: &Params) -> Vec<SgrParameter> {
        let mut sgr_params = Vec::new();
        let mut i = 0;
        let params_vec: Vec<i64> = params.iter().map(|p| p[0] as i64).collect();
        
        while i < params_vec.len() {
            let param = params_vec[i] as u32;
            match param {
                0 => sgr_params.push(SgrParameter::Reset),
                1 => sgr_params.push(SgrParameter::Bold),
                2 => sgr_params.push(SgrParameter::Dim),
                3 => sgr_params.push(SgrParameter::Italic),
                4 => sgr_params.push(SgrParameter::Underline),
                5 => sgr_params.push(SgrParameter::Blink),
                7 => sgr_params.push(SgrParameter::Reverse),
                8 => sgr_params.push(SgrParameter::Hidden),
                9 => sgr_params.push(SgrParameter::Strikethrough),
                
                21 => sgr_params.push(SgrParameter::NoBold),
                22 => sgr_params.push(SgrParameter::NoDim),
                23 => sgr_params.push(SgrParameter::NoItalic),
                24 => sgr_params.push(SgrParameter::NoUnderline),
                25 => sgr_params.push(SgrParameter::NoBlink),
                27 => sgr_params.push(SgrParameter::NoReverse),
                28 => sgr_params.push(SgrParameter::NoHidden),
                29 => sgr_params.push(SgrParameter::NoStrikethrough),
                
                // Foreground colors
                30..=37 => sgr_params.push(SgrParameter::Foreground(Color::from_ansi((param - 30) as u8))),
                38 => {
                    // Extended color
                    if i + 1 < params_vec.len() {
                        match params_vec[i + 1] {
                            5 if i + 2 < params_vec.len() => {
                                // 256 color
                                let color = Color::Indexed(params_vec[i + 2] as u8);
                                sgr_params.push(SgrParameter::Foreground(color));
                                i += 2;
                            }
                            2 if i + 4 < params_vec.len() => {
                                // RGB color
                                let r = params_vec[i + 2].clamp(0, 255) as u8;
                                let g = params_vec[i + 3].clamp(0, 255) as u8;
                                let b = params_vec[i + 4].clamp(0, 255) as u8;
                                sgr_params.push(SgrParameter::Foreground(Color::Rgb(r, g, b)));
                                i += 4;
                            }
                            _ => {}
                        }
                    }
                }
                39 => sgr_params.push(SgrParameter::DefaultForeground),
                
                // Background colors
                40..=47 => sgr_params.push(SgrParameter::Background(Color::from_ansi((param - 40) as u8))),
                48 => {
                    // Extended background color
                    if i + 1 < params_vec.len() {
                        match params_vec[i + 1] {
                            5 if i + 2 < params_vec.len() => {
                                // 256 color
                                let color = Color::Indexed(params_vec[i + 2] as u8);
                                sgr_params.push(SgrParameter::Background(color));
                                i += 2;
                            }
                            2 if i + 4 < params_vec.len() => {
                                // RGB color
                                let r = params_vec[i + 2].clamp(0, 255) as u8;
                                let g = params_vec[i + 3].clamp(0, 255) as u8;
                                let b = params_vec[i + 4].clamp(0, 255) as u8;
                                sgr_params.push(SgrParameter::Background(Color::Rgb(r, g, b)));
                                i += 4;
                            }
                            _ => {}
                        }
                    }
                }
                49 => sgr_params.push(SgrParameter::DefaultBackground),
                
                // Bright foreground colors
                90..=97 => sgr_params.push(SgrParameter::Foreground(Color::from_ansi((param - 90 + 8) as u8))),
                
                // Bright background colors
                100..=107 => sgr_params.push(SgrParameter::Background(Color::from_ansi((param - 100 + 8) as u8))),
                
                _ => debug!("Unhandled SGR parameter: {}", param),
            }
            i += 1;
        }
        
        sgr_params
    }
    
    /// Get a single numeric parameter with default value
    fn get_param(&self, params: &Params, index: usize, default: u16) -> u16 {
        params.iter()
            .nth(index)
            .map(|p| p[0] as u16)
            .filter(|&v| v > 0)
            .unwrap_or(default)
    }
}

impl Perform for TerminalPerformer {
    fn print(&mut self, c: char) {
        trace!("VTE print: {:?}", c);
        self.current_text.push(c);
    }
    
    fn execute(&mut self, byte: u8) {
        trace!("VTE execute: 0x{:02x}", byte);
        self.flush_text();
        
        match byte {
            0x07 => self.events.push(ParsedEvent::Control(ControlEvent::Bell)),
            0x08 => self.events.push(ParsedEvent::Control(ControlEvent::Backspace)),
            0x09 => self.events.push(ParsedEvent::Control(ControlEvent::Tab)),
            0x0A => self.events.push(ParsedEvent::Control(ControlEvent::NewLine)),
            0x0B => self.events.push(ParsedEvent::Control(ControlEvent::VerticalTab)),
            0x0C => self.events.push(ParsedEvent::Control(ControlEvent::FormFeed)),
            0x0D => self.events.push(ParsedEvent::Control(ControlEvent::CarriageReturn)),
            _ => debug!("Unhandled execute byte: 0x{:02x}", byte),
        }
    }
    
    fn hook(&mut self, params: &Params, intermediates: &[u8], ignore: bool, action: char) {
        trace!("VTE hook: params={:?}, intermediates={:?}, ignore={}, action={}", 
               params.iter().collect::<Vec<_>>(), intermediates, ignore, action);
    }
    
    fn put(&mut self, byte: u8) {
        trace!("VTE put: 0x{:02x}", byte);
    }
    
    fn unhook(&mut self) {
        trace!("VTE unhook");
    }
    
    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        trace!("VTE OSC: params={:?}, bell_terminated={}", params.len(), bell_terminated);
        self.flush_text();
        
        if params.is_empty() {
            return;
        }
        
        // Parse the OSC number
        let osc_num = std::str::from_utf8(params[0])
            .ok()
            .and_then(|s| s.parse::<u32>().ok());
            
        match osc_num {
            Some(0) | Some(2) => {
                // Set window title
                if params.len() > 1 {
                    if let Ok(title) = std::str::from_utf8(params[1]) {
                        self.events.push(ParsedEvent::Osc(OscSequence::SetTitle(title.to_string())));
                    }
                }
            }
            Some(8) => {
                // Hyperlink
                if params.len() > 2 {
                    if let Ok(uri) = std::str::from_utf8(params[2]) {
                        let id = if params[1].is_empty() {
                            None
                        } else {
                            // Parse params field for id=value
                            std::str::from_utf8(params[1])
                                .ok()
                                .and_then(|param_str| {
                                    param_str.split(';')
                                        .find(|p| p.starts_with("id="))
                                        .map(|p| p.strip_prefix("id=").unwrap_or(p).to_string())
                                })
                        };
                        
                        if uri.is_empty() {
                            self.events.push(ParsedEvent::Osc(OscSequence::ResetHyperlink));
                        } else {
                            self.events.push(ParsedEvent::Osc(OscSequence::SetHyperlink { 
                                id, 
                                uri: uri.to_string() 
                            }));
                        }
                    }
                }
            }
            _ => debug!("Unhandled OSC sequence: {:?}", osc_num),
        }
    }
    
    fn csi_dispatch(
        &mut self,
        params: &Params,
        intermediates: &[u8],
        ignore: bool,
        action: char,
    ) {
        trace!("VTE CSI: params={:?}, intermediates={:?}, ignore={}, action={}", 
               params.iter().collect::<Vec<_>>(), intermediates, ignore, action);
        self.flush_text();
        
        if ignore {
            return;
        }
        
        match action {
            // Cursor movement
            'A' => {
                let n = self.get_param(params, 0, 1);
                self.events.push(ParsedEvent::Csi(CsiSequence::CursorUp(n)));
            }
            'B' => {
                let n = self.get_param(params, 0, 1);
                self.events.push(ParsedEvent::Csi(CsiSequence::CursorDown(n)));
            }
            'C' => {
                let n = self.get_param(params, 0, 1);
                self.events.push(ParsedEvent::Csi(CsiSequence::CursorForward(n)));
            }
            'D' => {
                let n = self.get_param(params, 0, 1);
                self.events.push(ParsedEvent::Csi(CsiSequence::CursorBack(n)));
            }
            'E' => {
                let n = self.get_param(params, 0, 1);
                self.events.push(ParsedEvent::Csi(CsiSequence::CursorNextLine(n)));
            }
            'F' => {
                let n = self.get_param(params, 0, 1);
                self.events.push(ParsedEvent::Csi(CsiSequence::CursorPreviousLine(n)));
            }
            'G' => {
                let col = self.get_param(params, 0, 1);
                self.events.push(ParsedEvent::Csi(CsiSequence::CursorColumn(col)));
            }
            'H' | 'f' => {
                let row = self.get_param(params, 0, 1);
                let col = self.get_param(params, 1, 1);
                self.events.push(ParsedEvent::Csi(CsiSequence::CursorPosition { row, col }));
            }
            
            // Erase
            'J' => {
                let mode = match params.iter().next().map(|p| p[0]).unwrap_or(0) {
                    0 => EraseMode::Below,
                    1 => EraseMode::Above,
                    2 => EraseMode::All,
                    3 => EraseMode::Saved,
                    _ => EraseMode::Below,
                };
                self.events.push(ParsedEvent::Csi(CsiSequence::EraseDisplay(mode)));
            }
            'K' => {
                let mode = match params.iter().next().map(|p| p[0]).unwrap_or(0) {
                    0 => EraseMode::Below,
                    1 => EraseMode::Above,
                    2 => EraseMode::All,
                    _ => EraseMode::Below,
                };
                self.events.push(ParsedEvent::Csi(CsiSequence::EraseLine(mode)));
            }
            
            // Scrolling
            'S' => {
                let n = self.get_param(params, 0, 1);
                self.events.push(ParsedEvent::Csi(CsiSequence::ScrollUp(n)));
            }
            'T' => {
                let n = self.get_param(params, 0, 1);
                self.events.push(ParsedEvent::Csi(CsiSequence::ScrollDown(n)));
            }
            
            // SGR - Select Graphic Rendition
            'm' => {
                let sgr_params = self.parse_sgr_params(params);
                self.events.push(ParsedEvent::Csi(CsiSequence::SetGraphicsRendition(sgr_params)));
            }
            
            // Cursor visibility
            'h' if intermediates == b"?" => {
                for param in params.iter() {
                    match param[0] {
                        25 => self.events.push(ParsedEvent::Csi(CsiSequence::ShowCursor)),
                        _ => debug!("Unhandled DECSET mode: {}", param[0]),
                    }
                }
            }
            'l' if intermediates == b"?" => {
                for param in params.iter() {
                    match param[0] {
                        25 => self.events.push(ParsedEvent::Csi(CsiSequence::HideCursor)),
                        _ => debug!("Unhandled DECRST mode: {}", param[0]),
                    }
                }
            }
            
            // Save/Restore cursor
            's' => self.events.push(ParsedEvent::Csi(CsiSequence::SaveCursor)),
            'u' => self.events.push(ParsedEvent::Csi(CsiSequence::RestoreCursor)),
            
            _ => debug!("Unhandled CSI sequence: {}", action),
        }
    }
    
    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        trace!("VTE ESC: intermediates={:?}, ignore={}, byte=0x{:02x}", 
               intermediates, ignore, byte);
        self.flush_text();
        
        if ignore {
            return;
        }
        
        match byte {
            b'D' => self.events.push(ParsedEvent::Esc(EscSequence::Index)),
            b'E' => self.events.push(ParsedEvent::Esc(EscSequence::NextLine)),
            b'H' => self.events.push(ParsedEvent::Esc(EscSequence::TabSet)),
            b'M' => self.events.push(ParsedEvent::Esc(EscSequence::ReverseIndex)),
            b'c' => self.events.push(ParsedEvent::Esc(EscSequence::Reset)),
            b'7' => self.events.push(ParsedEvent::Esc(EscSequence::SaveCursor)),
            b'8' => self.events.push(ParsedEvent::Esc(EscSequence::RestoreCursor)),
            b'=' => self.events.push(ParsedEvent::Esc(EscSequence::KeypadApplicationMode)),
            b'>' => self.events.push(ParsedEvent::Esc(EscSequence::KeypadNumericMode)),
            _ => debug!("Unhandled ESC sequence: 0x{:02x}", byte),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plain_text() {
        let mut parser = VteParser::new();
        let events = parser.parse(b"Hello, World!");
        
        assert_eq!(events.len(), 1);
        match &events[0] {
            ParsedEvent::Text(s) => assert_eq!(s, "Hello, World!"),
            _ => panic!("Expected text event"),
        }
    }
    
    #[test]
    fn test_control_characters() {
        let mut parser = VteParser::new();
        let events = parser.parse(b"Hello\nWorld\r\n");
        
        assert_eq!(events.len(), 5);
        assert!(matches!(events[0], ParsedEvent::Text(_)));
        assert!(matches!(events[1], ParsedEvent::Control(ControlEvent::NewLine)));
        assert!(matches!(events[2], ParsedEvent::Text(_)));
        assert!(matches!(events[3], ParsedEvent::Control(ControlEvent::CarriageReturn)));
        assert!(matches!(events[4], ParsedEvent::Control(ControlEvent::NewLine)));
    }
    
    #[test]
    fn test_cursor_movement() {
        let mut parser = VteParser::new();
        
        // Cursor up
        let events = parser.parse(b"\x1b[5A");
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], ParsedEvent::Csi(CsiSequence::CursorUp(5))));
        
        // Cursor position
        let events = parser.parse(b"\x1b[10;20H");
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], ParsedEvent::Csi(CsiSequence::CursorPosition { row: 10, col: 20 })));
    }
    
    #[test]
    fn test_sgr_colors() {
        let mut parser = VteParser::new();
        
        // Basic colors
        let events = parser.parse(b"\x1b[31;42m");
        assert_eq!(events.len(), 1);
        match &events[0] {
            ParsedEvent::Csi(CsiSequence::SetGraphicsRendition(params)) => {
                assert_eq!(params.len(), 2);
                assert!(matches!(params[0], SgrParameter::Foreground(Color::Red)));
                assert!(matches!(params[1], SgrParameter::Background(Color::Green)));
            }
            _ => panic!("Expected SGR event"),
        }
        
        // 256 color
        let events = parser.parse(b"\x1b[38;5;123m");
        assert_eq!(events.len(), 1);
        match &events[0] {
            ParsedEvent::Csi(CsiSequence::SetGraphicsRendition(params)) => {
                assert_eq!(params.len(), 1);
                assert!(matches!(params[0], SgrParameter::Foreground(Color::Indexed(123))));
            }
            _ => panic!("Expected SGR event"),
        }
        
        // RGB color
        let events = parser.parse(b"\x1b[48;2;255;128;0m");
        assert_eq!(events.len(), 1);
        match &events[0] {
            ParsedEvent::Csi(CsiSequence::SetGraphicsRendition(params)) => {
                assert_eq!(params.len(), 1);
                assert!(matches!(params[0], SgrParameter::Background(Color::Rgb(255, 128, 0))));
            }
            _ => panic!("Expected SGR event"),
        }
    }
    
    #[test]
    fn test_osc_sequences() {
        let mut parser = VteParser::new();
        
        // Set title
        let events = parser.parse(b"\x1b]0;My Terminal\x07");
        assert_eq!(events.len(), 1);
        match &events[0] {
            ParsedEvent::Osc(OscSequence::SetTitle(title)) => {
                assert_eq!(title, "My Terminal");
            }
            _ => panic!("Expected OSC SetTitle event"),
        }
        
        // Hyperlink
        let events = parser.parse(b"\x1b]8;id=test;https://example.com\x07");
        assert_eq!(events.len(), 1);
        match &events[0] {
            ParsedEvent::Osc(OscSequence::SetHyperlink { id, uri }) => {
                assert_eq!(id.as_deref(), Some("test"));
                assert_eq!(uri, "https://example.com");
            }
            _ => panic!("Expected OSC SetHyperlink event"),
        }
    }
}