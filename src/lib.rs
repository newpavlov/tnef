//! A basic [TNEF] parser written in pure Rust.
//!
//! TNEF file contains a stream of records called "attributes". Using
//! `TnefReader` you can read attributes stored in the provided TNEF buffer.
//! At the moment we do not handle parsing of attribute data outside of
//! attachement attributes.
//!
//! If you just want to unpack attachments stored in TNEF, you can use a
//! convenience function `read_attachements`.
//!
//! # Usage example
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let tnef_data = b"\
//! #   \x78\x9f\x3e\x22\x23\x28\x01\x06\x90\x08\x00\x04\x00\x00\x00\x00\
//! #   \x00\x01\x00\x01\x00\x01\x07\x90\x06\x00\x08\x00\x00\x00\xe3\x04\
//! #   \x00\x00\x00\x00\x00\x00\xe7\x00\
//! # ";
//! for attribute in tnef::TnefReader::new(tnef_data)? {
//!     let (id, data) = attribute?;
//!     println!("{:?} {}", id, data.len());
//! }
//! # Ok(()) }
//! ```
//! [TNEF]: https://en.wikipedia.org/wiki/Transport_Neutral_Encapsulation_Format
use byteorder::{LE, ByteOrder};
use chrono::naive::NaiveDateTime;

mod error;
mod attr_ids;

use error::Error;
pub use attr_ids::*;

/// TNEF reader.
///
/// Core functionality is accessible via `Iterator` trait.
pub struct TnefReader<'a> {
    src: &'a [u8],
    code_page: u32,
    msg_section: bool,
    done: bool,
}

impl<'a> TnefReader<'a> {
    /// Create a new reader from the provided buffer.
    pub fn new(src: &'a [u8]) -> Result<Self, Error> {
        let mut ret = Self {
            src, code_page: 0, msg_section: true, done: false,
        };
        ret.read_header()?;
        ret.read_version()?;
        ret.code_page = ret.read_oem_code_page()?;
        Ok(ret)
    }

    /// Get OEM code page.
    pub fn get_code_page(&self) -> u32 {
        self.code_page
    }

    fn read_attribute(&mut self)
        -> Result<Option<(AttributeId, &'a [u8])>, Error>
    {
        if self.src.len() == 0 { return Ok(None); }
        let level = self.split_at(1)?[0];
        match (level, self.msg_section) {
            (0x01, true) | (0x02, false) => (),
            (0x01, false) => Err(Error::UnexpectedMessageAttribute)?,
            (0x02, true) => self.msg_section = false,
            _ => Err(Error::InvalidAttributeLevel(level))?,
        }
        let raw_id = self.read_u32()?;
        let id = AttributeId::from_u32(self.msg_section, raw_id)?;
        let len = self.read_u32()? as usize;
        let msg = self.split_at(len)?;
        self.verify_checksum(msg)?;
        Ok(Some((id, msg)))
    }

    fn verify_checksum(&mut self, msg: &[u8]) -> Result<(), Error> {
        let val = self.read_u16()?;
        let sum: u16 = msg.iter()
            .map(|b| *b as u16)
            .fold(0, |sum, i| sum.wrapping_add(i));
        if sum != val {
            Err(Error::ChecksumMismatch)
        } else {
            Ok(())
        }
    }

    fn read_u16(&mut self) -> Result<u16, Error> {
        Ok(LE::read_u16(self.split_at(2)?))
    }

    fn read_u32(&mut self) -> Result<u32, Error> {
        Ok(LE::read_u32(self.split_at(4)?))
    }

    fn split_at(&mut self, len: usize) -> Result<&'a [u8], Error> {
        if self.src.len() < len { return Err(Error::UnexpectedEof); }
        let (l, r) = {self.src}.split_at(len);
        self.src = r;
        Ok(l)
    }

    fn read_header(&mut self) -> Result<(), Error> {
        let h = self.read_u32()?;
        if h != 0x223e_9f78 {
            return Err(Error::InvalidHeader)?;
        }
        // ignore LegacyKey
        let _ = self.read_u16()?;
        Ok(())
    }

    fn read_version(&mut self) -> Result<(), Error> {
        let level = self.split_at(1)?[0];
        let id = self.read_u32()?;
        let len = self.read_u32()?;
        if level != 0x01 || id != 0x0008_9006 || len != 4 {
            return Err(Error::InvalidVersion);
        }
        let msg = self.split_at(4)?;
        if msg != b"\x00\x00\x01\x00" { return Err(Error::InvalidVersion); }
        self.verify_checksum(msg)?;
        Ok(())
    }

    fn read_oem_code_page(&mut self) -> Result<u32, Error> {
        let level = self.split_at(1)?[0];
        let id = self.read_u32()?;
        let len = self.read_u32()?;
        if level != 0x01 || id != 0x0006_9007 || len != 8 {
            return Err(Error::InvalidVersion)?;
        }
        let msg = self.split_at(8)?;
        self.verify_checksum(msg)?;
        let code_page = LE::read_u32(&msg[..4]);
        let sec_code_page = LE::read_u32(&msg[4..]);
        if sec_code_page != 0 {
            return Err(Error::InvalidOemCodePage)?;
        }

        codepage::to_encoding(code_page as u16)
            .ok_or(Error::InvalidOemCodePage)?;

        Ok(code_page)
    }
}

impl<'a> std::iter::Iterator for TnefReader<'a> {
    type Item = Result<(AttributeId, &'a [u8]), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done { return None; }
        match self.read_attribute() {
            Ok(Some(val)) => Some(Ok(val)),
            Ok(None) => {
                self.done = true;
                None
            }
            Err(err) => {
                self.done = true;
                Some(Err(err))
            }
        }
    }
}

impl<'a> std::iter::FusedIterator for TnefReader<'a> {}

fn parse_datetime(data: &[u8]) -> Result<NaiveDateTime, Error> {
    use chrono::naive::{NaiveDate, NaiveTime};
    if data.len() != 14 { return Err(Error::InvlidDateTime); }
    let year = LE::read_u16(&data[0..2]);
    let month = LE::read_u16(&data[2..4]);
    let day = LE::read_u16(&data[4..6]);
    let h = LE::read_u16(&data[6..8]);
    let m = LE::read_u16(&data[8..10]);
    let s = LE::read_u16(&data[10..12]);
    let _day_of_week = LE::read_u16(&data[12..14]);

    let d = NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
        .ok_or(Error::InvlidDateTime)?;
    let t = NaiveTime::from_hms_opt(h as u32, m as u32, s as u32)
        .ok_or(Error::InvlidDateTime)?;
    Ok(NaiveDateTime::new(d, t))
}

/// Attachment type.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum AttachType {
    File,
    Ole,
}

/// Attachment data flags
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum AttachDataFlags {
    FileDataDefault,
    FileDataMacBinary,
}

/// Attachment rendering data.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct RendData {
    pub attach_type: AttachType,
    pub attach_position: u32,
    pub render_width: u16,
    pub render_height: u16,
    pub flags: AttachDataFlags,
}

impl RendData {
    fn parse(data: &[u8]) -> Result<Self, Error> {
        if data.len() != 14 { return Err(Error::InvalidRendData); }
        let attach_type = match LE::read_u16(&data[0..2]) {
            0x0001 => AttachType::File,
            0x0002 => AttachType::Ole,
            _ => Err(Error::InvalidRendData)?,
        };
        let attach_position = LE::read_u32(&data[2..6]);
        let render_width = LE::read_u16(&data[6..8]);
        let render_height = LE::read_u16(&data[8..10]);
        let flags = match LE::read_u32(&data[10..14]) {
            0x0000_0000 => AttachDataFlags::FileDataDefault,
            0x0000_0001 => AttachDataFlags::FileDataMacBinary,
            _ => Err(Error::InvalidRendData)?,
        };
        Ok(Self {
            attach_type, attach_position, render_width, render_height, flags,
        })
    }
}

fn parse_string(data: &[u8], code_page: u32) -> Result<String, Error> {
    let n = data.len();
    if data.len() == 0 || data[n-1] != 0x00 {
        return Err(Error::InvalidString);
    }
    let (s, malformed) = codepage::to_encoding(code_page as u16)
        .ok_or(Error::InvalidString)?
        .decode_with_bom_removal(&data[..n-1]);
    if malformed { return Err(Error::InvalidString); }
    Ok(s.to_string())
}

/// TNEF attachment.
#[derive(Default)]
pub struct RawAttachment<'a> {
    pub data: Option<&'a [u8]>,
    pub title: Option<String>,
    pub meta: Option<&'a [u8]>,
    pub create_date: Option<NaiveDateTime>,
    pub modify_date: Option<NaiveDateTime>,
    pub transport_filename: Option<String>,
    pub rend_data: Option<RendData>,
    pub props: Option<&'a [u8]>,
}

impl<'a> RawAttachment<'a> {
    fn is_default(&self) -> bool {
        match self {
            RawAttachment {
                data: None, title: None, meta: None,
                create_date: None, modify_date: None,
                transport_filename: None, rend_data: None,
                props: None,
            } => true,
            _ => false,
        }
    }
}

/// TNEF attachment.
#[derive(Debug, Clone)]
pub struct Attachment<'a> {
    pub title: String,
    pub data: &'a [u8],
    pub create_date: NaiveDateTime,
    pub modify_date: NaiveDateTime,
    pub rend_data: RendData,
    pub props: &'a [u8],
    pub meta: Option<&'a [u8]>,
    pub transport_filename: Option<String>,
}

impl<'a> Attachment<'a> {
    fn from_raw(raw: RawAttachment<'a>) -> Option<Self> {
        match raw {
            RawAttachment {
                title: Some(title),
                data: Some(data),
                create_date: Some(create_date),
                modify_date: Some(modify_date),
                rend_data: Some(rend_data),
                props: Some(props),
                meta,
                transport_filename,
            } => Some(Attachment {
                title, data, create_date, modify_date, rend_data, props,
                meta, transport_filename,
            }),
            _ => None,
        }
    }
}

/// Convenience function for extracting attachments from TNEF data.
///
/// It assumes that attachements always contains the following fields:
/// `title`, `data`, `create_date`, `modify_date`, `rend_data` and `props`.
/// If one of those fields is missing the attachement will be ignored.
pub fn read_attachements(buf: &[u8]) -> Result<Vec<Attachment>, Error> {
    let r = TnefReader::new(&buf)?;
    let code_page = r.get_code_page();
    let mut buf = vec![];
    let mut t = RawAttachment::default();
    for attr in r {
        let (id, data) = attr?;
        let id = match id {
            AttributeId::Message(_) => continue,
            AttributeId::Attachment(v) => v,
        };
        // first attachment attribute must be AttachRendData
        if t.is_default() && id != AttachAttrId::AttachRendData {
            return Err(Error::AttachementParsingFailure);
        }
        match id {
            AttachAttrId::AttachRendData => {
                t.rend_data = Some(RendData::parse(data)?);
            },
            AttachAttrId::Data => {
                if t.data.is_none() {
                    t.data = Some(data);
                } else {
                    return Err(Error::AttachementParsingFailure);
                }
            }
            AttachAttrId::Title => {
                if t.title.is_none() {
                    t.title = Some(parse_string(data, code_page)?);
                } else {
                    return Err(Error::AttachementParsingFailure);
                }
            }
            AttachAttrId::MetaFile => {
                if t.meta.is_none() {
                    t.meta = Some(data);
                } else {
                    return Err(Error::AttachementParsingFailure);
                }
            }
            AttachAttrId::CreateDate => {
                if t.create_date.is_none() {
                    t.create_date = Some(parse_datetime(data)?);
                } else {
                    return Err(Error::AttachementParsingFailure);
                }
            }
            AttachAttrId::ModifyDate => {
                if t.modify_date.is_none() {
                    t.modify_date = Some(parse_datetime(data)?);
                } else {
                    return Err(Error::AttachementParsingFailure);
                }
            }
            AttachAttrId::TransportFilename => {
                if t.transport_filename.is_none() {
                    t.transport_filename = Some(parse_string(data, code_page)?);
                } else {
                    return Err(Error::AttachementParsingFailure);
                }
            }
            // last attachment attribute must be Attachment
            AttachAttrId::Attachment => {
                if t.props.is_none() {
                    t.props = Some(data);
                } else {
                    return Err(Error::AttachementParsingFailure);
                }
                if let Some(att) = Attachment::from_raw(t) {
                    buf.push(att);
                }
                t = RawAttachment::default();
            }
        }
    }
    Ok(buf)
}
