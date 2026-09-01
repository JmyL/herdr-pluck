use crate::model::{RenderLine, RenderStyle};
use crate::theme::PickerPalette;
use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Attribute, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::Write;

/// Emits abstract picker render lines to a terminal writer using the current Herdr theme.
pub fn emit_render_lines(writer: &mut impl Write, lines: &[RenderLine]) -> Result<()> {
    emit_render_lines_with_palette(writer, lines, &PickerPalette::load())
}

/// Emits picker lines with an explicit palette so tests can pin contrast colors.
pub fn emit_render_lines_with_palette(
    writer: &mut impl Write,
    lines: &[RenderLine],
    palette: &PickerPalette,
) -> Result<()> {
    queue!(writer, Clear(ClearType::All), MoveTo(0, 0))?;

    for (line_index, line) in lines.iter().enumerate() {
        for span in &line.spans {
            queue_style(writer, span.style, palette)?;
            queue!(writer, Print(&span.text))?;
        }
        if line_index + 1 < lines.len() {
            queue!(writer, Print("\r\n"))?;
        }
    }

    queue!(writer, ResetColor, SetAttribute(Attribute::Reset))?;
    Ok(())
}

fn queue_style(writer: &mut impl Write, style: RenderStyle, palette: &PickerPalette) -> Result<()> {
    match style {
        RenderStyle::Unmatched => {
            queue!(
                writer,
                SetAttribute(Attribute::Reset),
                SetForegroundColor(palette.unmatched_fg)
            )?;
            if palette.dim_unmatched {
                queue!(writer, SetAttribute(Attribute::Dim))?;
            }
        }
        RenderStyle::Match => queue!(
            writer,
            SetAttribute(Attribute::Reset),
            SetForegroundColor(palette.match_fg)
        )?,
        RenderStyle::Hint => queue!(
            writer,
            SetAttribute(Attribute::Reset),
            SetForegroundColor(palette.hint_fg),
            SetBackgroundColor(palette.hint_bg),
            SetAttribute(Attribute::Bold)
        )?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{RenderSpan, RenderStyle};
    use crate::theme::Appearance;

    fn sample_line() -> Vec<RenderLine> {
        vec![RenderLine {
            spans: vec![
                RenderSpan {
                    text: "open ".to_string(),
                    style: RenderStyle::Unmatched,
                },
                RenderSpan {
                    text: "a".to_string(),
                    style: RenderStyle::Hint,
                },
                RenderSpan {
                    text: "ttps://example.com".to_string(),
                    style: RenderStyle::Match,
                },
            ],
        }]
    }

    #[test]
    fn terminal_emission_clears_screen_and_writes_all_spans() {
        let mut output = Vec::new();
        emit_render_lines_with_palette(
            &mut output,
            &sample_line(),
            &PickerPalette::for_appearance(Appearance::Dark),
        )
        .unwrap();
        let output = String::from_utf8(output).unwrap();

        assert!(output.starts_with("\u{1b}[2J\u{1b}[1;1H"));
        assert!(output.contains("open "));
        assert!(output.contains("a"));
        assert!(output.contains("ttps://example.com"));
        assert!(output.contains("\u{1b}[38;5;0m"));
        assert!(output.contains("\u{1b}[48;5;14m"));
        assert!(output.contains("\u{1b}[38;5;11m"));
    }

    #[test]
    fn light_theme_hint_badge_uses_256color_white_on_blue() {
        let mut output = Vec::new();
        emit_render_lines_with_palette(
            &mut output,
            &sample_line(),
            &PickerPalette::for_appearance(Appearance::Light),
        )
        .unwrap();
        let output = String::from_utf8(output).unwrap();

        assert!(output.contains("\u{1b}[38;5;231m"));
        assert!(output.contains("\u{1b}[48;5;27m"));
        assert!(output.contains("\u{1b}[38;5;27m"));
        assert!(!output.contains("\u{1b}[38;2;"));
        assert!(!output.contains("\u{1b}[48;2;"));
    }

    #[test]
    fn terminal_emission_separates_lines_with_crlf() {
        let lines = vec![
            RenderLine {
                spans: vec![RenderSpan {
                    text: "one".to_string(),
                    style: RenderStyle::Unmatched,
                }],
            },
            RenderLine {
                spans: vec![RenderSpan {
                    text: "two".to_string(),
                    style: RenderStyle::Match,
                }],
            },
        ];
        let mut output = Vec::new();

        emit_render_lines_with_palette(
            &mut output,
            &lines,
            &PickerPalette::for_appearance(Appearance::Dark),
        )
        .unwrap();
        let output = String::from_utf8(output).unwrap();

        assert!(strip_ansi(&output).contains("one\r\ntwo"));
    }

    fn strip_ansi(text: &str) -> String {
        let mut output = String::new();
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\u{1b}' && chars.peek() == Some(&'[') {
                chars.next();
                for code_ch in chars.by_ref() {
                    if code_ch.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else {
                output.push(ch);
            }
        }
        output
    }
}
