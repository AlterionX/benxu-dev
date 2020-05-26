use std::fmt::Display;

use chrono::{DateTime, Utc};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use tap::*;

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
            PostRange::ByDate { .. } => None
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
            PostRange::ByDate { .. } => None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PostQuery {
    Structured {
        range: PostRange,
        sort: Option<PostSort>,
    },
}
impl Display for PostQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
        }
    }
}

impl std::convert::TryFrom<&seed::browser::url::UrlSearch> for PostQuery {
    type Error = String;
    fn try_from(search: &seed::browser::url::UrlSearch) -> Result<PostQuery, Self::Error> {
        log::debug!("Parsing url search params {:?}." , search);
        let mut lim = None;
        let mut offset = None;
        let mut ord = None;
        let mut ord_criteria = None;
        let mut stop_time = None;
        let mut start_time = None;
        for (k, vv) in search.iter() {
            let v = if let Some(v) = vv.get(0) { v.as_str() } else { continue; };
            let k = k.as_str();
            match k {
                "ord" => {
                    match v {
                        "asc" => ord.replace(SortOrdering::Descending),
                        "dsc" => ord.replace(SortOrdering::Ascending),
                        _ => return Err(format!("Unknown `ord` field {:?} in post query.", v)),
                    };
                },
                "ord_criteria" => { ord_criteria.replace(v); },
                "lim" => { lim.replace(v); },
                "offset" => { offset.replace(v); },
                "stop_time" => { stop_time.replace(v); },
                "start_time" => { start_time.replace(v); },
                _ => return Err(format!("Unknown search parameter {:?}.", k)),
            }
        }
        let opt_sort_ordering = if ord.is_some() || ord_criteria.is_some() {
            let ord_criteria = ord_criteria.unwrap_or("title");
            let ord = ord.map_or_else(|| match ord_criteria {
                "title" => Ok(SortOrdering::Ascending),
                "date" => Ok(SortOrdering::Descending),
                _ => return Err("Unknown `ord_criteria` field.")
                    .tap_err(|_| log::error!("Unknown `ord_criteria` field {:?} in post query", ord_criteria)),
            }, |v| Ok(v))?;
            Some(match ord_criteria {
                "title" => PostSort::AlphabeticalTitle(ord),
                "date" => PostSort::Date(ord),
                _ => return Err("Unknown `ord_criteria` field.".to_string())
                    .tap_err(|_| log::error!("Unknown `ord_criteria` field {:?} in post query", ord_criteria)),
            })
        }  else {
            None
        };

        let mut opt_search_params = None;
        if let (Some(lim), Some(offset)) = (lim, offset) {
            if opt_search_params.is_some() {
                return Err(format!("Unexpected multi search param sets."));
            }
            opt_search_params.replace(PostRange::LimAndOffset {
                lim: lim.parse()
                    .tap_err(|e| log::error!("Failed to parse `lim` {:?} from search params due to {:?}.", lim, e))
                    .map_err(|_| "Failed to parse `lim`.".to_string())?,
                offset: offset.parse()
                    .tap_err(|e| log::error!("Failed to parse `offset` {:?} from search params {:?}.", offset, e))
                    .map_err(|_| "Failed to parse `offset`.".to_string())?,
            });
        } else if lim.is_some() {
            return Err(format!("Unexpected missing `offset` in search param."));
        } else if offset.is_some() {
            return Err(format!("Unexpected missing `lim` in search param."));
        }
        
        if let (Some(start_time), Some(stop_time)) = (start_time, stop_time) {
            if opt_search_params.is_some() {
                return Err("Unexpected multi search param sets.".to_string());
            }
            opt_search_params.replace(PostRange::ByDate {
                begin: DateTime::parse_from_rfc3339(start_time)
                    .tap_err(|e| log::error!("Failed to parse `start_time` {:?} from search params due to {:?}.", start_time, e))
                    .map_err(|_| "Failed to parse `start_time`.".to_string())?
                    .with_timezone(&Utc),
                end: DateTime::parse_from_rfc3339(stop_time)
                    .tap_err(|e| log::error!("Failed to parse `stop_time` {:?} from search params {:?}.", stop_time, e))
                    .map_err(|_| "Failed to parse `stop_time`.".to_string())?
                    .with_timezone(&Utc),
            });
        } else if start_time.is_some() {
            return Err("Unexpected missing `stop_time` in search param.".to_string());
        } else if stop_time.is_some() {
            return Err("Unexpected missing `start_time` in search param.".to_string());
        }
        let post_range = opt_search_params.unwrap_or_else(Default::default);

        Ok(PostQuery::Structured {
            range: post_range,
            sort: opt_sort_ordering,
        })
    }
}

impl Into<seed::browser::url::UrlSearch> for &PostQuery {
    fn into(self) -> seed::browser::url::UrlSearch {
        let PostQuery::Structured {
            range,
            sort,
        } = self;
        let mut search = vec![];
        match range.clone().into_offset_and_lim() {
            Ok((offset, lim)) => {
                search.push(("offset".to_string(), vec![offset.to_string()]));
                search.push(("lim".to_string(), vec![lim.to_string()]));
            },
            Err((begin, end)) => {
                let begin = begin.to_rfc3339();
                let end = end.to_rfc3339();
                let begin = percent_encode(begin.as_bytes(), NON_ALPHANUMERIC);
                let end = percent_encode(end.as_bytes(), NON_ALPHANUMERIC);
                search.push(("start_time".to_string(), vec![begin.to_string()]));
                search.push(("stop_time".to_string(), vec![end.to_string()]));
            },
        }
        if let Some(sort) = sort {
            match sort {
                PostSort::Date(ord) => {
                    let ord = match ord {
                        SortOrdering::Ascending => "asc",
                        SortOrdering::Descending => "dsc",
                    };
                    search.push(("ord_criteria".to_string(), vec!["date".to_string()]));
                    search.push(("ord".to_string(), vec![ord.to_string()]));
                },
                PostSort::AlphabeticalTitle(ord) => {
                    let ord = match ord {
                        SortOrdering::Ascending => "asc",
                        SortOrdering::Descending => "dsc",
                    };
                    search.push(("ord_criteria".to_string(), vec!["title".to_string()]));
                    search.push(("ord".to_string(), vec![ord.to_string()]));
                },
            }
        }
        seed::browser::url::UrlSearch::new(search)
    }
}