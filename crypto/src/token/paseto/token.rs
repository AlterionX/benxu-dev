use crate::{
    encoding::base64::{decode_no_padding as b64_decode, encode_no_padding as b64_encode},
    token::paseto::collapse_to_vec,
};
use std::{
    convert::TryFrom,
    ops::{Bound, Deref},
    str,
};

const MINIMUM_SECTION_COUNT: usize = 3;
const MAXIMUM_SECTION_COUNT: usize = 4;
enum ThreeOrFourPeriods {
    ThreePeriods([usize; MINIMUM_SECTION_COUNT]),
    FourPeriods([usize; MAXIMUM_SECTION_COUNT]),
}
impl ThreeOrFourPeriods {
    // TODO maybe in the future?
    // fn copy_to_array<T: Copy + Default, const N: usize>(slice: &[T]) -> [T; N] {
    //     let mut arr = [T::default(); N]; // TODO eliminate default value, use std::mem::uninitialized()
    //     arr.copy_from_slice(slice);
    //     arr
    // }
    // TODO remove when copy_to_array can work
    fn copy_to_3_array<T: Copy + Default>(slice: &[T]) -> [T; 3] {
        let mut arr = [T::default(); 3]; // TODO eliminate default value, use std::mem::uninitialized()
        arr.copy_from_slice(slice);
        arr
    }
    // TODO remove when copy_to_array can work
    fn copy_to_4_array<T: Copy + Default>(slice: &[T]) -> [T; 4] {
        let mut arr = [T::default(); 4]; // TODO eliminate default value, use std::mem::uninitialized()
        arr.copy_from_slice(slice);
        arr
    }
    fn from_slice(period_indices: &[usize]) -> Result<Self, UnexpectedNumberOfPeriods> {
        match period_indices.len() {
            MINIMUM_SECTION_COUNT => Ok(Self::ThreePeriods(Self::copy_to_3_array(period_indices))),
            MAXIMUM_SECTION_COUNT => Ok(Self::FourPeriods(Self::copy_to_4_array(period_indices))),
            _ => Err(UnexpectedNumberOfPeriods::new(period_indices.len())),
        }
    }
    fn period_cnt(&self) -> usize {
        // TODO generic size
        match self {
            Self::ThreePeriods(_) => 3,
            Self::FourPeriods(_) => 4,
        }
    }
    // TODO enforce const range on idx from 0..2
    // TODO replace with const generics when available
    fn opt_val_at(&self, i: usize) -> Option<usize> {
        if i < self.period_cnt() {
            let a = match self {
                Self::ThreePeriods(a) => &a[..],
                Self::FourPeriods(a) => &a[..],
            };
            Some(a[i])
        } else {
            None
        }
    }
    // TODO enforce const range on idx from 0..1
    // TODO replace with const generics when available
    fn val_at(&self, i: usize) -> usize {
        match self {
            Self::ThreePeriods(a) => a[i],
            Self::FourPeriods(a) => a[i],
        }
    }
    fn version_range(&self) -> (Bound<usize>, Bound<usize>) {
        let start = 0;
        let end = self.val_at(0);
        (Bound::Included(start), Bound::Excluded(end))
    }
    fn purpose_range(&self) -> (Bound<usize>, Bound<usize>) {
        let start = self.val_at(0);
        let end = self.val_at(1);
        (Bound::Excluded(start), Bound::Excluded(end))
    }
    fn body_range(&self) -> (Bound<usize>, Bound<usize>) {
        let start = self.val_at(1);
        let end = self.opt_val_at(2);
        (
            Bound::Excluded(start),
            end.map_or(Bound::Unbounded, |e| Bound::Excluded(e)),
        )
    }
    fn footer_range(&self) -> Option<(Bound<usize>, Bound<usize>)> {
        let start = self.opt_val_at(2)?;
        Some((Bound::Excluded(start), Bound::Unbounded))
    }
}
struct UnexpectedNumberOfPeriods(usize);
impl UnexpectedNumberOfPeriods {
    fn new(periods_cnt: usize) -> Self {
        if periods_cnt < MINIMUM_SECTION_COUNT || periods_cnt > MAXIMUM_SECTION_COUNT {
            panic!(
                "Expected illegal number of periods, instead was provided {}.",
                periods_cnt
            );
        }
        Self(periods_cnt)
    }
}

#[derive(Debug)]
pub enum UnpackingError {
    IncorrectNumberOfSections,
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

#[derive(Debug)]
pub enum DeserializeError {
    Json,
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

#[derive(Debug, PartialEq, Eq)]
pub struct Header<'a> {
    version: &'a [u8],
    purpose: &'a [u8],
}
impl<'a> Header<'a> {
    pub const fn new(version: &'a [u8], purpose: &'a [u8]) -> Self {
        Header {
            version: version,
            purpose: purpose,
        }
    }
    pub fn to_combined(&self) -> Vec<u8> {
        // TODO cache + custom PartialEq if needed
        collapse_to_vec(&[self.version, b".", self.purpose, b"."])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data<M, F> {
    pub msg: M,
    pub footer: Option<F>,
}
impl<M: serde::Serialize, F: serde::Serialize> Data<M, F> {
    pub fn serialize(self) -> Result<SerializedData, serde_json::Error> {
        SerializedData::try_from(self)
    }
}
impl<M: serde::de::DeserializeOwned, F: serde::de::DeserializeOwned> Data<M, F> {
    fn deserialize_component<T: serde::de::DeserializeOwned>(
        target: &Vec<u8>,
    ) -> Result<T, DeserializeError> {
        Ok(serde_json::from_str(str::from_utf8(target.as_slice())?)?)
    }
    fn opt_deserialize_component<T: serde::de::DeserializeOwned>(
        target: &Option<Vec<u8>>,
    ) -> Option<Result<T, DeserializeError>> {
        let target = target.as_ref()?;
        Some(Self::deserialize_component(target))
    }
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

pub struct SerializedData {
    pub msg: Vec<u8>,
    pub footer: Option<Vec<u8>>,
}
impl SerializedData {
    fn serialize_component<T: serde::Serialize>(target: &T) -> Result<Vec<u8>, serde_json::Error> {
        Ok(serde_json::to_string(target)?.as_bytes().to_vec())
    }
    fn opt_serialize_component<T: serde::Serialize>(
        target: &Option<T>,
    ) -> Option<Result<Vec<u8>, serde_json::Error>> {
        let target = target.as_ref()?;
        Some(Self::serialize_component(target))
    }
    fn serialize<M: serde::Serialize, F: serde::Serialize>(
        tok: Data<M, F>,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            msg: Self::serialize_component(&tok.msg)?,
            footer: Self::opt_serialize_component(&tok.footer).transpose()?,
        })
    }
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

pub struct Packed(Vec<u8>);
impl Packed {
    // called from elsewhere
    pub fn new(buffer: Vec<u8>) -> Self {
        Self(buffer)
    }
    pub fn unpack(self) -> Result<Unpacked, UnpackingError> {
        Unpacked::unpack(self)
    }
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

pub struct Unpacked {
    pub version: Vec<u8>,
    pub purpose: Vec<u8>,
    pub body: Vec<u8>,
    pub footer: Option<Vec<u8>>,
}
impl Unpacked {
    // creation
    pub(super) fn new(header: Header, body: Vec<u8>, footer: Option<Vec<u8>>) -> Self {
        Self {
            version: header.version.to_vec(),
            purpose: header.purpose.to_vec(),
            body: body,
            footer: footer,
        }
    }
    // marshalling from Packed
    fn find_first_four_periods(
        buf: &[u8],
    ) -> Result<ThreeOrFourPeriods, UnexpectedNumberOfPeriods> {
        let mut indices = Vec::with_capacity(5);
        for (idx, c) in buf.iter().enumerate() {
            if *c == b'.' {
                indices.push(idx);
            }
        }
        Ok(ThreeOrFourPeriods::from_slice(indices.as_slice())?)
    }
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
    fn unpack(packed: Packed) -> Result<Self, UnpackingError> {
        let packed = &*packed;
        let period_indices = Self::find_first_four_periods(packed)?;
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
    // marshalling to Packed
    pub fn pack(self) -> Packed {
        Packed::pack(self)
    }
    // verification
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
