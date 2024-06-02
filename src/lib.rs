mod options;

use std::borrow::Borrow;
use std::io::Read;
use std::io::Write;

pub use options::NumberingMode;
pub use options::Options;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CatError {
    #[error("io error")]
    Io(#[from] std::io::Error),
}

pub type CatResult<T> = Result<T, CatError>;

struct State {
    /// The current line number
    line_number: usize,

    /// Whether the output cursor is at the beginning of a new line
    at_line_start: bool,

    /// Whether we skipped a \r, which still needs to be printed
    skipped_carriage_return: bool,

    /// Whether we have already printed a blank line    
    one_blank_kept: bool,
}

fn cat_fast<R: Read, W: Write>(input: &mut R, output: &mut W, _options: &Options) -> CatResult<()> {
    let mut buf = [0; 1024 * 64];
    while let Ok(n) = input.read(&mut buf) {
        if n == 0 {
            break;
        }
        output.write(&buf[..n])?;
    }

    Ok(())
}

fn cat_lines<R: Read, W: Write>(
    input: &mut R,
    output: &mut W,
    options: &Options,
    mut state: State,
) -> CatResult<()> {
    let mut inbuf = [0; 1024 * 31];
    while let Ok(n) = input.read(&mut inbuf) {
        if n == 0 {
            break;
        }

        let inbuf = &inbuf[..n];
        let mut pos = 0;
        while pos < n {
            // skip empty line_number, enumerating them if needed
            if inbuf[pos] == b'\n' {
                write_new_line(output, options, &mut state)?;
                state.at_line_start = true;
                pos += 1;
                continue;
            }
            if state.skipped_carriage_return {
                output.write_all(b"\r")?;
                state.skipped_carriage_return = false;
                state.at_line_start = false;
            }
            state.one_blank_kept = false;
            if state.at_line_start && options.number != NumberingMode::None {
                write!(output, "{0:6}\t", state.line_number)?;
                state.line_number += 1;
            }

            // print to end of line or end of buffer
            let offset = write_end(output, &inbuf[pos..], options);

            // end of buffer?
            if offset + pos == inbuf.len() {
                state.at_line_start = false;
                break;
            }

            if inbuf[pos + offset] == b'\r' {
                state.skipped_carriage_return = true;
            } else {
                debug_assert_eq!(inbuf[pos + offset], b'\n');
                // print suitable end of line
                write_end_of_line(output, options.end_of_line().as_bytes())?;
                state.at_line_start = true;
            }
            pos += offset + 1;
        }
    }

    Ok(())
}

fn write_new_line<W: Write>(output: &mut W, options: &Options, state: &mut State) -> CatResult<()> {
    if state.skipped_carriage_return && options.show_ends {
        output.write_all(b"^M")?;
        state.skipped_carriage_return = false;
    }

    if !state.at_line_start || !options.squeeze_blank || !state.one_blank_kept {
        state.one_blank_kept = true;
        if state.at_line_start && options.number == NumberingMode::All {
            write!(output, "{0:6}\t", state.line_number)?;
            state.line_number += 1;
        }
        output.write_all(options.end_of_line().as_bytes())?;
        output.flush()?;
    }

    Ok(())
}

fn write_end<W: Write>(output: &mut W, inbuf: &[u8], options: &Options) -> usize {
    if options.show_nonprinting {
        write_nonprint_to_end(inbuf, output, options.tab().as_bytes())
    } else if options.show_tabs {
        write_tab_to_end(inbuf, output)
    } else {
        write_to_end(inbuf, output)
    }
}

// write***_to_end methods
// Write all symbols till \n or \r or end of buffer is reached
// We need to stop at \r because it may be written as ^M depending on the byte after and settings;
// however, write_nonprint_to_end doesn't need to stop at \r because it will always write \r as ^M.
// Return the number of written symbols
fn write_to_end<W: Write>(inbuf: &[u8], output: &mut W) -> usize {
    match inbuf.iter().position(|c| *c == b'\n' || *c == b'\r') {
        Some(p) => {
            output.write_all(&inbuf[..p]).unwrap();
            p
        }
        None => {
            output.write_all(inbuf).unwrap();
            inbuf.len()
        }
    }
}

fn write_tab_to_end<W: Write>(mut inbuf: &[u8], output: &mut W) -> usize {
    let mut count = 0;
    loop {
        match inbuf
            .iter()
            .position(|c| *c == b'\n' || *c == b'\t' || *c == b'\r')
        {
            Some(p) => {
                output.write_all(&inbuf[..p]).unwrap();
                if inbuf[p] == b'\t' {
                    output.write_all(b"^I").unwrap();
                    inbuf = &inbuf[p + 1..];
                    count += p + 1;
                } else {
                    // b'\n' or b'\r'
                    return count + p;
                }
            }
            None => {
                output.write_all(inbuf).unwrap();
                return inbuf.len();
            }
        };
    }
}

fn write_nonprint_to_end<W: Write>(inbuf: &[u8], output: &mut W, tab: &[u8]) -> usize {
    let mut count = 0;

    for byte in inbuf.iter().copied() {
        if byte == b'\n' {
            break;
        }
        match byte {
            9 => output.write_all(tab),
            0..=8 | 10..=31 => output.write_all(&[b'^', byte + 64]),
            32..=126 => output.write_all(&[byte]),
            127 => output.write_all(&[b'^', b'?']),
            128..=159 => output.write_all(&[b'M', b'-', b'^', byte - 64]),
            160..=254 => output.write_all(&[b'M', b'-', byte - 128]),
            _ => output.write_all(&[b'M', b'-', b'^', b'?']),
        }
        .unwrap();
        count += 1;
    }
    count
}

fn write_end_of_line<W: Write>(writer: &mut W, end_of_line: &[u8]) -> CatResult<()> {
    writer.write_all(end_of_line)?;
    writer.flush()?;
    Ok(())
}

pub fn cat<R: Read, W: Write>(input: &mut R, output: &mut W, options: &Options) -> CatResult<()> {
    if options.can_write_fast() {
        cat_fast(input, output, options)
    } else {
        cat_lines(
            input,
            output,
            options,
            State {
                line_number: 0,
                at_line_start: true,
                skipped_carriage_return: false,
                one_blank_kept: false,
            },
        )
    }
}

#[derive(Error, Debug)]
pub enum CatFilesError {
    #[error("file not found")]
    NotFound(String),
    #[error("io error")]
    Io(#[from] std::io::Error),
}

pub fn cat_files<T: Borrow<String>>(files: &[T], options: &Options) -> Result<(), CatFilesError> {
    let mut stdout = std::io::stdout();
    for file in files {
        let mut file = std::fs::File::open(file.borrow()).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => CatFilesError::NotFound(file.borrow().to_string()),
            _ => CatFilesError::Io(e),
        })?;
        cat(&mut file, &mut stdout, options).map_err(|e| match e {
            CatError::Io(e) => CatFilesError::Io(e),
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // write all the tests
    use super::*;

    #[test]
    fn test_write_to_end() {
        let mut output = Vec::new();
        let input = b"Hello, world!";
        let n = write_to_end(input, &mut output);
        assert_eq!(n, input.len());
        assert_eq!(output, input);
    }

    #[test]
    fn test_write_tab_to_end() {
        let mut output = Vec::new();
        let input = b"Hello, world!";
        let n = write_tab_to_end(input, &mut output);
        assert_eq!(n, input.len());
        assert_eq!(output, input);
    }

    #[test]
    fn test_write_nonprint_to_end() {
        let mut output = Vec::new();
        let input = b"Hello, world!";
        let tab = b"    ";
        let n = write_nonprint_to_end(input, &mut output, tab);
        assert_eq!(n, input.len());
        assert_eq!(output, input);
    }

    #[test]
    fn test_write_end_of_line() {
        let mut output = Vec::new();
        let input = b"\n";
        write_end_of_line(&mut output, input).unwrap();
        assert_eq!(output, input);
    }

    // Copilot: test cat stuff with unicode, nonprinting, an assorted set of options

    #[test]
    fn test_cat_files_not_found() {
        let options = Options::new();
        let files = vec!["nonexistent_file".to_string()];
        let result = cat_files(&files, &options);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CatFilesError::NotFound(_)));
    }

    #[test]
    fn test_cat_fast() {
        let options = Options::new();
        let mut input = std::io::Cursor::new(b"Hello, world!");
        let mut output = Vec::new();
        let result = cat_fast(&mut input, &mut output, &options);
        assert!(result.is_ok());
        assert_eq!(output, b"Hello, world!");
    }

    #[test]
    fn test_cat_lines() {
        let options = Options::new();
        let mut input = std::io::Cursor::new(b"Hello, world!");
        let mut output = Vec::new();
        let result = cat_lines(
            &mut input,
            &mut output,
            &options,
            State {
                line_number: 0,
                at_line_start: true,
                skipped_carriage_return: false,
                one_blank_kept: false,
            },
        );
        assert!(result.is_ok());
        assert_eq!(output, b"Hello, world!");
    }

    #[test]
    fn test_cat() {
        let options = Options::new();
        let mut input = std::io::Cursor::new(b"Hello, world!");
        let mut output = Vec::new();
        let result = cat(&mut input, &mut output, &options);
        assert!(result.is_ok());
        assert_eq!(output, b"Hello, world!");
    }

    #[test]
    fn test_cat_nonprinting() {
        let options = Options::new().show_nonprinting(true);
        let mut input = std::io::Cursor::new(b"Hello, world!\x08");
        let mut output = Vec::new();
        let result = cat(&mut input, &mut output, &options);
        assert!(result.is_ok());
        assert_eq!(output, b"Hello, world!^H");
    }
}
