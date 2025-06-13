/*
MIT License

Copyright (c) 2018 Philipp Keller

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 */

use std::fmt;
use regex::Regex;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ReadUntil {
    String(String),
    Regex(Regex),
    EOF,
    NBytes(usize),
    Any(Vec<ReadUntil>),
}

impl fmt::Display for ReadUntil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let printable = match self {
            ReadUntil::String(s) if s == "\n" => "\\n (newline)".to_owned(),
            ReadUntil::String(s) if s == "\r" => "\\r (carriage return)".to_owned(),
            ReadUntil::String(s) => format!("\"{s}\""),
            ReadUntil::Regex(r) => format!("Regex: \"{r}\""),
            ReadUntil::EOF => "EOF (End of File)".to_owned(),
            ReadUntil::NBytes(n) => format!("reading {n} bytes"),
            ReadUntil::Any(v) => {
                let mut res = Vec::new();
                for r in v {
                    res.push(r.to_string());
                }
                res.join(", ")
            }
        };
        write!(f, "{printable}")
    }
}

/// find first occurrence of needle within buffer
///
/// # Arguments:
///
/// - buffer: the currently read buffer from a process which will still grow in the future
/// - eof: if the process already sent an EOF or a HUP
///
/// # Return
///
/// Tuple with match positions:
/// 1. position before match (0 in case of EOF and Nbytes)
/// 2. position after match
pub(crate) fn find(needle: &ReadUntil, buffer: &str, eof: bool) -> Option<(usize, usize)> {
    match needle {
        ReadUntil::String(s) => buffer.find(s).map(|pos| (pos, pos + s.len())),
        ReadUntil::Regex(pattern) => pattern.find(buffer).map(|mat| (mat.start(), mat.end())),
        ReadUntil::EOF => {
            if eof {
                Some((0, buffer.len()))
            } else {
                None
            }
        }
        ReadUntil::NBytes(n) => {
            if *n <= buffer.len() {
                Some((0, *n))
            } else if eof && !buffer.is_empty() {
                // reached almost end of buffer, return string, even though it will be
                // smaller than the wished n bytes
                Some((0, buffer.len()))
            } else {
                None
            }
        }
        ReadUntil::Any(anys) => anys
            .iter()
            // Filter matching needles
            .filter_map(|any| find(any, buffer, eof))
            // Return the left-most match
            .min_by(|(start1, end1), (start2, end2)| {
                if start1 == start2 {
                    end1.cmp(end2)
                } else {
                    start1.cmp(start2)
                }
            }),
    }
}
