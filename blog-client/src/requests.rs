use std::fmt::Display;

use chrono::{DateTime, Utc};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SortOrdering {
    Ascending,
    Descending,
}
impl Display for SortOrdering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ascending => write!(f, "ord=asc"),
            Self::Descending => write!(f, "ord=dsc"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PostSort {
    Date(SortOrdering),
    AlphabeticalTitle(SortOrdering),
}
impl Display for PostSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Date(ord) => write!(f, "ord_criteria=date&{}", ord),
            Self::AlphabeticalTitle(ord) => write!(f, "ord_criteria=title&{}", ord),
        }
    }
}
impl Default for PostSort {
    fn default() -> Self {
        Self::Date(SortOrdering::Descending)
    }
}
impl PostSort {
    fn get_ordering(&self) -> &SortOrdering {
        match self {
            Self::AlphabeticalTitle(o) => o,
            Self::Date(o) => o,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PostPagination {
    Ten,
    Twenty,
    Fifty,
}
impl PostPagination {
    fn to_usize(&self) -> usize {
        match self {
            Self::Ten => 10,
            Self::Twenty => 20,
            Self::Fifty => 50,
        }
    }
}
impl Default for PostPagination {
    fn default() -> Self {
        Self::Twenty
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PostRange {
    ByDate {
        begin: DateTime<Utc>,
        end: DateTime<Utc>,
    },
    ByPage {
        page_size: PostPagination,
        page_num: usize,
    },
    LimAndOffset {
        offset: usize,
        lim: usize,
    },
}
impl PostRange {
    fn into_offset_and_lim(self) -> Result<(usize, usize), (DateTime<Utc>, DateTime<Utc>)> {
        match self {
            Self::ByPage {
                page_size,
                page_num,
            } => Ok({
                let page_size = page_size.to_usize();
                (page_size * page_num, page_size)
            }),
            Self::LimAndOffset { offset, lim } => Ok((offset, lim)),
            Self::ByDate { begin, end } => Err((begin, end)),
        }
    }
}
impl Default for PostRange {
    fn default() -> Self {
        Self::ByPage {
            page_num: 0,
            page_size: PostPagination::Twenty,
        }
    }
}
impl Display for PostRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.clone().into_offset_and_lim() {
            Ok((offset, lim)) => write!(f, "lim={}&offset={}", lim, offset),
            Err((begin, end)) => write!(
                f,
                "start_time={}&stop_time={}",
                percent_encode(begin.to_rfc3339().as_bytes(), NON_ALPHANUMERIC),
                percent_encode(end.to_rfc3339().as_bytes(), NON_ALPHANUMERIC),
            ),
        }
    }
}
impl PostRange {
    fn generate_next(&self) -> Option<PostRange> {
        match self {
            PostRange::ByPage {
                page_size,
                page_num,
            } => Some(PostRange::ByPage {
                page_size: page_size.clone(),
                page_num: page_num + 1,
            }),
            PostRange::LimAndOffset {
                lim,
                offset,
            } => Some(PostRange::LimAndOffset {
                lim: lim.clone(),
                offset: offset + lim,
            }),
            PostRange::ByDate {
                begin,
                end,
            } => None
        }
    }
    fn generate_prev(&self) -> Option<PostRange> {
        match self {
            PostRange::ByPage {
                page_size,
                page_num,
            } => if *page_num == 0 {
                None
            } else {
                Some(PostRange::ByPage {
                    page_size: page_size.clone(),
                    page_num: page_num - 1,
                })
            },
            PostRange::LimAndOffset {
                lim,
                offset,
            } => if *offset == 0 {
                None
            } else {
                Some(PostRange::LimAndOffset {
                    lim: lim.clone(),
                    offset: if *offset < *lim {
                        0
                    } else {
                        offset - lim
                    },
                })
            },
            PostRange::ByDate {
                begin,
                end,
            } => None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PostQuery {
    Structured {
        range: PostRange,
        sort: Option<PostSort>,
    },
    Raw(String),
}
impl Display for PostQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Raw(s) => write!(f, "{}", s),
            Self::Structured {
                range,
                sort: Some(sort),
            } => write!(f, "{}&{}", range, sort),
            Self::Structured { range, sort: None } => write!(f, "{}", range),
        }
    }
}
impl Default for PostQuery {
    fn default() -> Self {
        Self::Structured {
            range: PostRange::default(),
            sort: None,
        }
    }
}

impl PostQuery {
    pub fn generate_next(&self) -> Option<PostQuery> {
        // TODO be smarter about how many posts are actually available.
        match self {
            Self::Structured {
                range,
                sort,
            } => Some(Self::Structured {
                range: range.generate_next()?,
                sort: sort.clone(),
            }),
            Self::Raw(_) => None
        }
    }
    pub fn generate_prev(&self) -> Option<PostQuery> {
        match self {
            Self::Structured {
                range,
                sort,
            } => Some(Self::Structured {
                range: range.generate_prev()?,
                sort: sort.clone(),
            }),
            Self::Raw(_) => None
        }
    }
}