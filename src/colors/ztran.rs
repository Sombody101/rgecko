use crate::colors::transform::{MachineState, MarkupOptions};
use crate::v_log;

pub fn z_markup_text(text: &str, options: MarkupOptions) {
    let logger = options.logger;

    let mut output = String::with_capacity(text.len());
    let mut state = MachineState::Normal;

    let mut start = 0;

    let mut bytes = text.as_bytes();
    let len = bytes.len();

    let peek_equals = |index: usize, b: u8| -> bool { index + 1 < len && bytes[index + 1] == b };

    let lookbehind_equals = |index: usize, b: u8| -> bool { index > 0 && bytes[index - 1] == b };

    for idx in 0..len {
        let byte = bytes[idx];

        match state {
            MachineState::Normal => {
                if byte == b'[' {
                    if byte != b'[' {
                        state = MachineState::ReadingColor;
                        continue;
                    }

                    v_log!(logger, "Escaping tag at index {idx}");
                    
                }
            }

            _ => {}
        }
    }
}
