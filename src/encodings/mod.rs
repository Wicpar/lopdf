pub mod cmap;
mod glyphnames;
mod mappings;

extern crate encoding;

use crate::Error;
use crate::Result;
use cmap::ToUnicodeCMap;
use encoding::EncoderTrap;
use encoding::{all::UTF_16BE, DecoderTrap, Encoding as _};

pub use self::mappings::*;

pub fn bytes_to_string(encoding: &ByteToGlyphMap, bytes: &[u8]) -> String {
    let code_points = bytes
        .iter()
        .filter_map(|&byte| encoding[byte as usize])
        .collect::<Vec<u16>>();
    String::from_utf16_lossy(&code_points)
}

pub fn string_to_bytes(encoding: &ByteToGlyphMap, text: &str) -> Vec<u8> {
    text.encode_utf16()
        .filter_map(|ch| encoding.iter().position(|&code| code == Some(ch)))
        .map(|byte| byte as u8)
        .collect()
}

pub enum Encoding<'a> {
    OneByteEncoding(&'a ByteToGlyphMap),
    SimpleEncoding(&'a str),
    UnicodeMapEncoding(ToUnicodeCMap),
}

impl<'a> Encoding<'a> {
    pub fn bytes_to_string(&self, bytes: &[u8]) -> Result<String> {
        match self {
            Self::OneByteEncoding(map) => Ok(bytes_to_string(map, bytes)),
            Self::SimpleEncoding(name) if ["UniGB-UCS2-H", "UniGB−UTF16−H"].contains(name) => UTF_16BE
                .decode(bytes, DecoderTrap::Ignore)
                .map_err(|_| Error::ContentDecode),
            Self::UnicodeMapEncoding(unicode_map) => {
                let glyphs = bytes
                    .chunks_exact(2)
                    .map(|chunk| chunk[0] as u16 * 256 + chunk[1] as u16)
                    .flat_map(|cp| {
						// println!("{cp:x}");
						let ret = unicode_map.get_or_replacement_char(cp);
						// println!("{}", String::from_utf16_lossy(&ret));
						ret
					}).collect::<Vec<_>>();
				Ok(String::from_utf16_lossy(&glyphs))
            }
            _ => Err(Error::ContentDecode),
        }
    }
    pub fn string_to_bytes(&self, text: &str) -> Vec<u8> {
        match self {
            Self::OneByteEncoding(map) => string_to_bytes(map, text),
            Self::SimpleEncoding(name) if ["UniGB-UCS2-H", "UniGB-UTF16-H"].contains(name) => {
                UTF_16BE.encode(text, EncoderTrap::Ignore).unwrap()
            }
            Self::UnicodeMapEncoding(unicode_map) => {
				let map = unicode_map.get_best_possible_reverse_map();
				text.encode_utf16().filter_map(|it|map.get(&it).copied()).flat_map(|it|it.to_be_bytes()).collect()
            }
            _ => string_to_bytes(&STANDARD_ENCODING, text),
        }
    }
}
