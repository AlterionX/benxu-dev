use seed::browser::fetch::FetchError;

use super::AllowRetry;

const TIME_OUT_CODE: u16 = 408;
const RESOURCE_CONFLICT_CODE: u16 = 409;
const TEAPOT_CODE: u16 = 418;
const TOO_EARLY_CODE: u16 = 425;
const TOO_MANY_CODE: u16 = 429;
const BAD_GATEWAY_CODE: u16 = 502;
const SERVICE_UNAVAILABLE_CODE: u16 = 504;

struct ErrorReportInfo<T> {
    serde: T,
    dom: T,
    promise: T,
    network: T,
    request_construction: T,
    status_code: T,
}

impl<T> ErrorReportInfo<T> {
    fn resolve(&self, e: &FetchError) -> &T {
        match &e {
            FetchError::SerdeError(_) => &self.serde,
            FetchError::DomException(_) => &self.dom,
            FetchError::PromiseError(_) => &self.promise,
            FetchError::NetworkError(_) => &self.network,
            FetchError::RequestError(_) => &self.request_construction,
            FetchError::StatusError(_) => &self.status_code,
        }
    }
}

pub(super) enum FailSource {
    Initial,
    Confirm,
    Parsing,
}

struct FailSourceVar<T> {
    initial: T,
    confirm: T,
    parsing: T,
}

impl<T> FailSourceVar<T> {
    fn new(initial: T, confirm: T, parsing: T) -> Self {
        Self {
            initial,
            confirm,
            parsing,
        }
    }

    fn resolve(&self, src: &FailSource) -> &T {
        match src {
            FailSource::Initial => &self.initial,
            FailSource::Confirm => &self.confirm,
            FailSource::Parsing => &self.parsing,
        }
    }
}

fn log_err<Err: std::fmt::Debug>(e: Err, name: &str, msg: &str, is_possible: bool, can_retry: bool) {
    let continuation_status_msg = if is_possible {
        if can_retry {
            "Retrying."
        } else {
            "Aborting."
        }
    } else {
        "Should be impossible. Aborting."
    };
    log::error!("Encountered {} {:?} while {}. {}", name, e, msg, continuation_status_msg);
}

pub(super) fn process_fetch_err(e: FetchError, msg: &str, source: FailSource) -> AllowRetry {
    const ERR_POSSIBILITY: ErrorReportInfo<FailSourceVar<bool>> = ErrorReportInfo {
        serde: FailSourceVar::new(false, false, true),
        dom: FailSourceVar::new(true, false, false),
        promise: FailSourceVar::new(true, false, true),
        network: FailSourceVar::new(true, false, false),
        request_construction: FailSourceVar::new(false, false, false),
        status_code: FailSourceVar::new(false, true, false),
    };

    const RETRY_ALLOWANCE: ErrorReportInfo<FailSourceVar<bool>> = ErrorReportInfo {
        serde: FailSourceVar::new(false, false, false),
        dom: FailSourceVar::new(true, false, false),
        promise: FailSourceVar::new(true, false, true),
        network: FailSourceVar::new(true, false, false),
        request_construction: FailSourceVar::new(false, false, false),
        status_code: FailSourceVar::new(false, true, true),
    };

    const ERROR_NAME: ErrorReportInfo<&str> = ErrorReportInfo {
        serde: "serialization error",
        dom: "DomException",
        promise: "PromiseError",
        network: "network error",
        request_construction: "request construction",
        status_code: "status code error",
    };

    let is_possible = *ERR_POSSIBILITY.resolve(&e).resolve(source);
    let can_retry = *RETRY_ALLOWANCE.resolve(&e).resolve(source);
    let name = *ERROR_NAME.resolve(&e);

    // Need another match and repetitive calls due to `e` having different types.
    let passes_secondary_check = match e {
        FetchError::SerdeError(e) => {
            log_err(e, name, msg, is_possible, can_retry);
            true
        },
        FetchError::DomException(e) => {
            log_err(e, name, msg, is_possible, can_retry);
            true
        },
        FetchError::PromiseError(e) => {
            log_err(e, name, msg, is_possible, can_retry);
            true
        },
        FetchError::NetworkError(e) => {
            log_err(e, name, msg, is_possible, can_retry);
            true
        },
        FetchError::RequestError(e) => {
            log_err(e, name, msg, is_possible, can_retry);
            true
        },
        FetchError::StatusError(e) => match e.code {
            TIME_OUT_CODE
            | RESOURCE_CONFLICT_CODE
            | TEAPOT_CODE
            | TOO_EARLY_CODE
            | TOO_MANY_CODE
            | BAD_GATEWAY_CODE
            | SERVICE_UNAVAILABLE_CODE => {
                log_err(e, name, msg, is_possible, can_retry);
                true
            },
            _ => {
                log_err(e, name, msg, is_possible, can_retry);
                false
            }
        },
    };

    if is_possible && can_retry && passes_secondary_check {
        AllowRetry::Allow
    } else {
        AllowRetry::Disallow
    }
}