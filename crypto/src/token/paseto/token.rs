//! A collection of types used throughout all four standard PASETO protocols.

use std::{
    convert::TryFrom,
    ops::{Bound, Deref},
    str,
};

use crate::{
    encoding::base64::{decode_no_padding as b64_decode, encode_no_padding as b64_encode},
    token::paseto::util::collapse_to_vec,
};

/// Maximum number of sections.
const MINIMUM_SECTION_COUNT: usize = 3;
/// Minimum number of sections.
const MAXIMUM_SECTION_COUNT: usize = 4;
/// Maximum number of sections.
const MINIMUM_PERIOD_COUNT: usize = MINIMUM_SECTION_COUNT - 1;
/// Minimum number of sections.
const MAXIMUM_PERIOD_COUNT: usize = MAXIMUM_SECTION_COUNT - 1;
/// Stores the location of either three or four periods.
enum MinToMaxPeriods {
    MinPeriods([usize; MINIMUM_PERIOD_COUNT]),
    MaxPeriods([usize; MAXIMUM_PERIOD_COUNT]),
}
impl MinToMaxPeriods {
    // TODO maybe in the future?
    // fn copy_to_array<T: Copy + Default, const N: usize>(slice: &[T]) -> [T; N] {
    //     let mut arr = [T::default(); N]; // TODO eliminate default value, use std::mem::uninitialized()
    //     arr.copy_from_slice(slice);
    //     arr
    // }
    /// Copies the first two elements of the slice into an array.
    // TODO remove when copy_to_array can work
    fn copy_to_min_array<T: Copy + Default>(slice: &[T]) -> [T; MINIMUM_PERIOD_COUNT] {
        let mut arr = [T::default(); MINIMUM_PERIOD_COUNT]; // TODO eliminate default value, use std::mem::uninitialized()
        arr.copy_from_slice(slice);
        arr
    }
    // TODO remove when copy_to_array can work
    /// Copies the first three elements of the slice into an array.
    fn copy_to_max_array<T: Copy + Default>(slice: &[T]) -> [T; MAXIMUM_PERIOD_COUNT] {
        let mut arr = [T::default(); MAXIMUM_PERIOD_COUNT]; // TODO eliminate default value, use std::mem::uninitialized()
        arr.copy_from_slice(slice);
        arr
    }
    /// Converts a slice of indices into the enum.
    fn from_slice(period_indices: &[usize]) -> Result<Self, UnexpectedNumberOfPeriods> {
        match period_indices.len() {
            MINIMUM_PERIOD_COUNT => Ok(Self::MinPeriods(Self::copy_to_min_array(period_indices))),
            MAXIMUM_PERIOD_COUNT => Ok(Self::MaxPeriods(Self::copy_to_max_array(period_indices))),
            _ => Err(UnexpectedNumberOfPeriods::new(period_indices.len())),
        }
    }
    /// Returns the number of periods represented.
    fn period_cnt(&self) -> usize {
        // TODO generic size
        match self {
            Self::MinPeriods(_) => MINIMUM_PERIOD_COUNT,
            Self::MaxPeriods(_) => MAXIMUM_PERIOD_COUNT,
        }
    }
    /// Get, optionally, the index of the nth period where n is less than 4.
    // TODO enforce const range on idx from 0..2
    // TODO replace with const generics when available
    fn opt_val_at(&self, i: usize) -> Option<usize> {
        if i < self.period_cnt() {
            let a = match self {
                Self::MinPeriods(a) => &a[..],
                Self::MaxPeriods(a) => &a[..],
            };
            Some(a[i])
        } else {
            None
        }
    }
    /// Get the index of the nth period where n is less than 2.
    // TODO enforce const range on idx from 0..1
    // TODO replace with const generics when available
    fn val_at(&self, i: usize) -> usize {
        match self {
            Self::MinPeriods(a) => a[i],
            Self::MaxPeriods(a) => a[i],
        }
    }
    /// The range of the protocol version header.
    fn version_range(&self) -> (Bound<usize>, Bound<usize>) {
        let start = 0;
        let end = self.val_at(0);
        (Bound::Included(start), Bound::Excluded(end))
    }
    /// The range of the protocol version header.
    fn purpose_range(&self) -> (Bound<usize>, Bound<usize>) {
        let start = self.val_at(0);
        let end = self.val_at(1);
        (Bound::Excluded(start), Bound::Excluded(end))
    }
    /// The range of the body, should it exist.
    fn body_range(&self) -> (Bound<usize>, Bound<usize>) {
        let start = self.val_at(1);
        let end = self.opt_val_at(2);
        (
            Bound::Excluded(start),
            end.map_or(Bound::Unbounded, |e| Bound::Excluded(e)),
        )
    }
    /// The range of the footer, should it exist.
    fn footer_range(&self) -> Option<(Bound<usize>, Bound<usize>)> {
        let start = self.opt_val_at(2)?;
        Some((Bound::Excluded(start), Bound::Unbounded))
    }
}
/// Stores the illegal quantity of periods found within the payload.
struct UnexpectedNumberOfPeriods(
    /// The number of periods.
    usize,
);
impl UnexpectedNumberOfPeriods {
    fn new(periods_cnt: usize) -> Self {
        if periods_cnt < MINIMUM_PERIOD_COUNT || periods_cnt > MAXIMUM_PERIOD_COUNT {
            panic!(
                "Expected illegal number of periods, instead was provided {}.",
                periods_cnt
            );
        }
        Self(periods_cnt)
    }
}

/// Errors that can occur during unpacking.
#[derive(Debug)]
pub enum UnpackingError {
    /// Incorrect number of sections.
    IncorrectNumberOfSections,
    /// Incorrect encoding.
    MalformedEncoding,
}
impl From<UnexpectedNumberOfPeriods> for UnpackingError {
    fn from(_: UnexpectedNumberOfPeriods) -> Self {
        UnpackingError::IncorrectNumberOfSections
    }
}
impl From<base64::DecodeError> for UnpackingError {
    fn from(_: base64::DecodeError) -> Self {
        UnpackingError::MalformedEncoding
    }
}

/// Errors that can occur during deserialization.
#[derive(Debug)]
pub enum DeserializeError {
    /// Payload was not JSON encoded.
    Json,
    /// Payload was not UTF8.
    Utf8,
}
impl From<serde_json::Error> for DeserializeError {
    fn from(_: serde_json::Error) -> Self {
        DeserializeError::Json
    }
}
impl From<str::Utf8Error> for DeserializeError {
    fn from(_: str::Utf8Error) -> Self {
        DeserializeError::Utf8
    }
}

/// Header structure.
#[derive(Debug, PartialEq, Eq)]
pub struct Header<'a> {
    /// Version of the protocol used.
    version: &'a [u8],
    /// Purpose of the protocol used.
    purpose: &'a [u8],
}
impl<'a> Header<'a> {
    /// Constructs a [`Header`] with listed `version` and `purpose`.
    pub const fn new(version: &'a [u8], purpose: &'a [u8]) -> Self {
        Self {
            version: version,
            purpose: purpose,
        }
    }
    /// Created the [`Header`] with periods inserted as such: "`version`.`purpose`."
    pub fn to_combined(&self) -> Vec<u8> {
        // TODO cache + custom PartialEq if needed
        collapse_to_vec(&[self.version, b".", self.purpose, b"."])
    }
}

/// Hold the pre-protocol data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data<M, F> {
    /// The message to be serialized and sent.
    pub msg: M,
    /// The footer to be serialized and sent.
    pub footer: Option<F>,
}
impl<M: serde::Serialize, F: serde::Serialize> Data<M, F> {
    /// Serialize [`Data`] into the [`SerializedData`] form.
    pub fn serialize(self) -> Result<SerializedData, serde_json::Error> {
        SerializedData::try_from(self)
    }
}
impl<M: serde::de::DeserializeOwned, F: serde::de::DeserializeOwned> Data<M, F> {
    /// Deserialize a [`Vec`] of bytes into the a struct.
    fn deserialize_component<T: serde::de::DeserializeOwned>(
        target: &Vec<u8>,
    ) -> Result<T, DeserializeError> {
        Ok(serde_json::from_str(str::from_utf8(target.as_slice())?)?)
    }
    /// Deserialize an optional [`Vec`] of bytes into the a struct.
    fn opt_deserialize_component<T: serde::de::DeserializeOwned>(
        target: &Option<Vec<u8>>,
    ) -> Option<Result<T, DeserializeError>> {
        let target = target.as_ref()?;
        Some(Self::deserialize_component(target))
    }
    /// Deserialize a [`Vec`] of bytes into the a [`Data`] struct.
    fn deserialize(tok: SerializedData) -> Result<Self, DeserializeError> {
        Ok(Self {
            msg: Self::deserialize_component(&tok.msg)?,
            footer: Self::opt_deserialize_component(&tok.footer).transpose()?,
        })
    }
}
impl<M: serde::de::DeserializeOwned, F: serde::de::DeserializeOwned> TryFrom<SerializedData>
    for Data<M, F>
{
    type Error = DeserializeError;
    fn try_from(tok: SerializedData) -> Result<Self, Self::Error> {
        Self::deserialize(tok)
    }
}

/// Represents the [`Data`] struct, but serialized.
pub struct SerializedData {
    /// The message to be sent.
    pub msg: Vec<u8>,
    /// The footer to be sent.
    pub footer: Option<Vec<u8>>,
}
impl SerializedData {
    /// Serialize part of the struct.
    fn serialize_component<T: serde::Serialize>(target: &T) -> Result<Vec<u8>, serde_json::Error> {
        Ok(serde_json::to_string(target)?.as_bytes().to_vec())
    }
    /// Serialize an optional part of the struct.
    fn opt_serialize_component<T: serde::Serialize>(
        target: &Option<T>,
    ) -> Option<Result<Vec<u8>, serde_json::Error>> {
        let target = target.as_ref()?;
        Some(Self::serialize_component(target))
    }
    /// Serialize the [`Data`] struct into the [`SerializedData`] struct.
    fn serialize<M: serde::Serialize, F: serde::Serialize>(
        tok: Data<M, F>,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            msg: Self::serialize_component(&tok.msg)?,
            footer: Self::opt_serialize_component(&tok.footer).transpose()?,
        })
    }
    /// Deserialize the [`SerializedData`] into the [`Data`] struct.
    pub fn deserialize<M: serde::de::DeserializeOwned, F: serde::de::DeserializeOwned>(
        self,
    ) -> Result<Data<M, F>, DeserializeError> {
        Data::try_from(self)
    }
}
impl<M: serde::Serialize, F: serde::Serialize> TryFrom<Data<M, F>> for SerializedData {
    type Error = serde_json::Error;
    fn try_from(tok: Data<M, F>) -> Result<Self, Self::Error> {
        Self::serialize(tok)
    }
}

/// The packed PASETO token.
pub struct Packed(Vec<u8>);
impl Packed {
    /// Creates a new token from a payload.
    pub fn new(buffer: Vec<u8>) -> Self {
        Self(buffer)
    }
    /// Attempts to unpack a [`Packed`] token into an [`Unpacked`] one. Errors if the payload is
    /// malformed.
    pub fn unpack(self) -> Result<Unpacked, UnpackingError> {
        Unpacked::unpack(self)
    }
    /// Packs an [`Unpacked`] token into a [`Packed`] one. This follows the
    /// "`version`.`protocol`.`body`(.`footer`)" structure.
    fn pack(tok: Unpacked) -> Packed {
        let possible_footer = tok
            .footer
            .as_ref()
            .map_or(b"".to_vec(), |f| b64_encode(f.as_slice()));
        Packed(collapse_to_vec(&[
            tok.version.as_slice(),
            b".",
            tok.purpose.as_slice(),
            b".",
            b64_encode(tok.body.as_slice()).as_slice(),
            tok.footer.as_ref().map_or(b"", |_| b"."),
            possible_footer.as_slice(),
        ]))
    }
}
impl Deref for Packed {
    type Target = [u8];
    fn deref<'a>(&'a self) -> &'a Self::Target {
        &self.0
    }
}
impl From<Unpacked> for Packed {
    fn from(token: Unpacked) -> Self {
        Self::pack(token)
    }
}

/// The unpacked, but encrypted/signed PASETO token.
pub struct Unpacked {
    /// The binary version of the version.
    pub version: Vec<u8>,
    /// The binary version of the purpose.
    pub purpose: Vec<u8>,
    /// The binary version of the body of the message.
    pub body: Vec<u8>,
    /// The binary version of the optional footer.
    pub footer: Option<Vec<u8>>,
}
impl Unpacked {
    /// Creates a new token from the [`Header`], body, and optional footer. The [`Header`] is
    /// effectively cloned.
    pub(super) fn new(header: Header, body: Vec<u8>, footer: Option<Vec<u8>>) -> Self {
        Self {
            version: header.version.to_vec(),
            purpose: header.purpose.to_vec(),
            body: body,
            footer: footer,
        }
    }

    /// Locate and return the positions of the periods in the payload.
    fn find_section_dividers(buf: &[u8]) -> Result<MinToMaxPeriods, UnexpectedNumberOfPeriods> {
        let mut indices = Vec::with_capacity(5);
        for (idx, c) in buf.iter().enumerate() {
            if *c == b'.' {
                indices.push(idx);
            }
        }
        Ok(MinToMaxPeriods::from_slice(indices.as_slice())?)
    }
    /// Slice a slice with a bound two-tuple.
    fn extract_bounds<'a>(slice: &'a [u8], (start, end): (Bound<usize>, Bound<usize>)) -> &'a [u8] {
        let start = match start {
            Bound::Included(i) => i,
            Bound::Excluded(i) => i + 1,
            Bound::Unbounded => 0,
        };
        let end = match end {
            Bound::Included(i) => i + 1,
            Bound::Excluded(i) => i,
            Bound::Unbounded => slice.len(),
        };
        &slice[start..end]
    }
    /// Attempt to unpack a [`Packed`] token.
    fn unpack(packed: Packed) -> Result<Self, UnpackingError> {
        let packed = &*packed;
        let period_indices = Self::find_section_dividers(packed)?;
        Ok(Self {
            version: Self::extract_bounds(packed, period_indices.version_range()).to_vec(),
            purpose: Self::extract_bounds(packed, period_indices.purpose_range()).to_vec(),
            body: b64_decode(Self::extract_bounds(packed, period_indices.body_range()))?,
            footer: period_indices
                .footer_range()
                .map(|r| b64_decode(Self::extract_bounds(packed, r)))
                .transpose()?,
        })
    }
    /// Pack token.
    pub fn pack(self) -> Packed {
        Packed::pack(self)
    }
    /// Verify header of the token.
    pub fn verify_header(&self, header: Header<'_>) -> bool {
        Header {
            version: self.version.as_slice(),
            purpose: self.purpose.as_slice(),
        } == header
    }
}
impl TryFrom<Packed> for Unpacked {
    type Error = UnpackingError;
    fn try_from(token: Packed) -> Result<Self, UnpackingError> {
        Self::unpack(token)
    }
}
