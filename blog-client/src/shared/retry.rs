use seed::browser::fetch::{fetch, Response, Request, Result as FetchResult};

mod error;

const RETRY_LIM: usize = 10;

pub struct LogPair<'a> {
    pub pre_completion: &'a str,
    pub post_completion: &'a str,
}

pub enum AllowRetry {
    Allow,
    Disallow,
}

async fn fetch_conditional<'a>(req: Request<'a>, logging_msg: &LogPair<'a>) -> Result<Response, AllowRetry> {
    fetch(req).await
        .map_err(|e| error::process_fetch_err(
            e,
            logging_msg.pre_completion,
            error::FailSource::Initial,
        ))
}

pub struct RetryResult {
    pub retries: usize,
    pub response: Response,
}

pub async fn fetch_with_retry<'a>(req: Request<'a>, logging_msg: &LogPair<'a>, retry_lim: Option<usize>) -> Result<RetryResult, ()> {
    // TODO Figure out a good default retry limit.
    let retry_lim = retry_lim.unwrap_or(RETRY_LIM);
    for retry_cnt in 0..retry_lim {
        if retry_cnt != 0 {
            let next_retry = ordinal::Ordinal(retry_cnt + 1);
            log::debug!("Performing {} retry of {}.", next_retry, logging_msg.pre_completion);
        }

        let fetch_attempt = fetch_conditional(req.clone(), logging_msg).await;
        let res = match fetch_attempt {
            Ok(res) => res,
            Err(AllowRetry::Allow) =>  {
                continue;
            },
            Err(AllowRetry::Disallow) =>  {
                break;
            },
        };

        let status_check = res.check_status()
            .map_err(|e| error::process_fetch_err(e, logging_msg.pre_completion, error::FailSource::Confirm));
        let res = match status_check {
            Ok(res) => res,
            Err(AllowRetry::Allow) =>  {
                continue;
            },
            Err(AllowRetry::Disallow) =>  {
                break;
            },
        };

        return Ok(RetryResult {
            retries: retry_cnt,
            response: res,
        });
    }
    log::error!("Hit retry limit or abort while {}, force aborting.", logging_msg.pre_completion);
    Err(())
}

pub async fn fetch_process_with_retry<'a, 'b, T, FutT, F>(
    req: Request<'a>,
    logging_msg: &LogPair<'a>,
    retry_lim: Option<usize>,
    process_res: F
) -> Result<T, ()>
    where
        FutT: std::future::Future<Output = FetchResult<T>>,
        F: Fn(&Response) -> FutT,
{
    let retry_lim = retry_lim.unwrap_or(RETRY_LIM);
    let mut retry_cnt = 0;
    while retry_cnt < retry_lim {
        if retry_cnt != 0 {
            let next_retry = ordinal::Ordinal(retry_cnt + 1);
            log::debug!("Performing {} retry of {}.", next_retry, logging_msg.pre_completion);
        }

        let RetryResult {
            response: res,
            retries,
        } = fetch_with_retry(req.clone(), logging_msg, Some(retry_lim - retry_cnt)).await?;
        retry_cnt += retries + 1; // An extra one for the count, since the successful request didn't count.

        let process_attempt = process_res(&res)
            .await
            .map_err(|e| error::process_fetch_err(e, logging_msg.post_completion, error::FailSource::Parsing));
        let res = match process_attempt {
            Ok(obj) => obj,
            Err(AllowRetry::Allow) => {
                continue;
            },
            Err(AllowRetry::Disallow) => {
                break;
            },
        };
        return Ok(res);
    };
    log::error!("Hit retry limit or abort while {}, force aborting.", logging_msg.pre_completion);
    Err(())
}

#[deprecated = "Should use `fetch_process_with_rety` once it's bug free."]
pub async fn fetch_json_with_retry<'a, T: 'static + serde::de::DeserializeOwned>(
    req: Request<'a>,
    logging_msg: &LogPair<'a>,
    retry_lim: Option<usize>,
) -> Result<T, ()> {
    let retry_lim = retry_lim.unwrap_or(RETRY_LIM);
    let mut retry_cnt = 0;
    while retry_cnt < retry_lim {
        if retry_cnt != 0 {
            let next_retry = ordinal::Ordinal(retry_cnt + 1);
            log::debug!("Performing {} retry of {}.", next_retry, logging_msg.pre_completion);
        }

        let RetryResult {
            response: res,
            retries,
        } = fetch_with_retry(req.clone(), logging_msg, Some(retry_lim - retry_cnt)).await?;
        retry_cnt += retries + 1; // An extra one for the count, since the successful request didn't count.

        let process_attempt = res.json()
            .await
            .map_err(|e| error::process_fetch_err(e, logging_msg.post_completion, error::FailSource::Parsing));
        let res = match process_attempt {
            Ok(obj) => obj,
            Err(AllowRetry::Allow) => {
                continue;
            },
            Err(AllowRetry::Disallow) => {
                break;
            },
        };
        return Ok(res);
    };
    log::error!("Hit retry limit or abort while {}, force aborting.", logging_msg.pre_completion);
    Err(())
}

#[deprecated = "Should use `fetch_process_with_rety` once it's bug free."]
pub async fn fetch_text_with_retry<'a>(
    req: Request<'a>,
    logging_msg: &LogPair<'a>,
    retry_lim: Option<usize>,
) -> Result<String, ()> {
    let retry_lim = retry_lim.unwrap_or(RETRY_LIM);
    let mut retry_cnt = 0;
    while retry_cnt < retry_lim {
        if retry_cnt != 0 {
            let next_retry = ordinal::Ordinal(retry_cnt + 1);
            log::debug!("Performing {} retry of {}.", next_retry, logging_msg.pre_completion);
        }

        let RetryResult {
            response: res,
            retries,
        } = fetch_with_retry(req.clone(), logging_msg, Some(retry_lim - retry_cnt)).await?;
        retry_cnt += retries + 1; // An extra one for the count, since the successful request didn't count.

        let process_attempt = res.text()
            .await
            .map_err(|e| error::process_fetch_err(e, logging_msg.post_completion, error::FailSource::Parsing));
        let res = match process_attempt {
            Ok(obj) => obj,
            Err(AllowRetry::Allow) => {
                continue;
            },
            Err(AllowRetry::Disallow) => {
                break;
            },
        };
        return Ok(res);
    };
    log::error!("Hit retry limit or abort while {}, force aborting.", logging_msg.pre_completion);
    Err(())
}

#[cfg(test)]
mod test {
    #[test]
    fn lifetime_wackiness() {
        let res = super::fetch_process_with_retry(
            "www.google.com",
            &super::LogPair {
                pre_completion: "pinging google",
                post_completion: "parsing res",
            }, None,
            |res| res.text(),
        ).await;
    }
}