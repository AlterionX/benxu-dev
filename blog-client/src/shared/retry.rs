use seed::browser::fetch::{fetch, FetchError, Response, Request, Result as FetchResult};

pub struct LogPair<'a> {
    pre_completion: &'a str,
    post_completion: &'a str,
}

pub enum AllowRetry {
    Allow,
    Disallow,
}

async fn fetch_conditional<'a>(req: Request<'a>, logging_msg: &LogPair<'a>) -> Result<Response, AllowRetry> {
    match fetch(req.clone()).await {
        Err(e) =>  match e {
            FetchError::SerdeError(e) => {
                log::error!("Encountered serialization error {:?} while {}. Should be impossible. Aborting.", e, logging_msg.pre_completion);
                Err(AllowRetry::Disallow)
            },
            FetchError::DomException(e) => {
                log::error!("Encountered DomException {:?} while {}. Aborting.", e, logging_msg.pre_completion);
                Err(AllowRetry::Disallow)
            },
            FetchError::PromiseError(e) => {
                log::error!("Encountered PromiseError {:?} while {}. Retrying.", e, logging_msg.pre_completion);
                Err(AllowRetry::Allow)
            },
            FetchError::NetworkError(e) => {
                log::error!("Encountered network error {:?} while {}. Retrying.", e, logging_msg.pre_completion);
                Err(AllowRetry::Allow)
            },
            FetchError::RequestError(e) => {
                log::error!("Failed to construct request for {}. Aborting.", logging_msg.pre_completion);
                Err(AllowRetry::Disallow)
            },
            FetchError::StatusError(e) => {
                // TODO Further consider what error, since you want to retry some HTTP error codes.
                log::error!("Server returned status error {:?} while {}. Aborting.", e, logging_msg.pre_completion);
                Err(AllowRetry::Disallow)
            },
        },
        Ok(res) => {
            Ok(res)
        }
    }
}

const RETRY_LIM: usize = 10;

struct RetryResult {
    retries: usize,
    response: Response,
}

pub async fn fetch_with_retry<'a>(req: Request<'a>, logging_msg: &LogPair<'a>, retry_lim: Option<usize>) -> Result<RetryResult, ()> {
    // TODO Figure out a good default retry limit.
    let retry_lim = retry_lim.unwrap_or(RETRY_LIM);
    for retry_cnt in 0..retry_lim {
        if retry_cnt != 0 {
            let next_retry = ordinal::Ordinal(retry_cnt + 1);
            log::debug!("Performing {} retry of {}.", next_retry, logging_msg.pre_completion);
        }
        match fetch_conditional(req.clone(), logging_msg).await {
            Err(e) =>  match e {
                AllowRetry::Allow => {
                    continue;
                },
                AllowRetry::Disallow => {
                    break;
                },
            },
            Ok(res) => {
                let res = match res.check_status() {
                    Ok(res) => res,
                    Err(e) => {
                        match e {
                            FetchError::SerdeError(e) => {
                                log::error!("Encountered serialization error {:?} after {}. Should be impossible. Aborting.", e, logging_msg.pre_completion);
                            },
                            FetchError::DomException(e) => {
                                log::error!("Encountered DomException {:?} after {}. Should be impossible. Aborting.", e, logging_msg.pre_completion);
                            },
                            FetchError::PromiseError(e) => {
                                log::error!("Encountered PromiseError {:?} after {}. Should be impossible. Aborting.", e, logging_msg.pre_completion);
                            },
                            FetchError::NetworkError(e) => {
                                log::error!("Encountered network error {:?} after {}. Should be impossible. Aborting.", e, logging_msg.pre_completion);
                            },
                            FetchError::RequestError(e) => {
                                log::error!("Failed to construct request after {}. Should be impossible. Aborting.", logging_msg.pre_completion);
                            },
                            FetchError::StatusError(e) => {
                                // TODO Further consider what error, since we might want to retry some HTTP error codes.
                                log::error!("Server returned status error {:?} after {}. Aborting.", e, logging_msg.pre_completion)
                            },
                        }
                        break;
                    }
                };

                return Ok(RetryResult {
                    retries: retry_cnt,
                    response: res,
                });
            }
        };
    }
    log::error!("Hit retry limit or abort while {}, force aborting.", logging_msg.pre_completion);
    Err(())
}

pub async fn fetch_process_with_retry<'a, T, FutT, F>(
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

        // TODO: Consider future additions to the retries here.
        let obj = match process_res(&res).await {
            Ok(obj) => return Ok(obj),
            Err(e) => {
                match e {
                    FetchError::SerdeError(e) => {
                        log::error!("Encountered serialization error {:?} while {}. Aborting.", e, logging_msg.post_completion);
                    },
                    FetchError::DomException(e) => {
                        log::error!("Encountered DomException {:?} while {}. Should be impossible. Aborting.", e, logging_msg.post_completion);
                    },
                    FetchError::PromiseError(e) => {
                        log::error!("Encountered PromiseError {:?} while {}. Should be impossible. Aborting.", e, logging_msg.post_completion);
                    },
                    FetchError::NetworkError(e) => {
                        log::error!("Encountered network error {:?} while {}. Should be impossible. Aborting.", e, logging_msg.post_completion);
                    },
                    FetchError::RequestError(e) => {
                        log::error!("Failed to construct request for {}. Should be impossible. Aborting.", logging_msg.post_completion);
                    },
                    FetchError::StatusError(e) => {
                        // TODO Further consider what error, since we might want to retry some HTTP error codes.
                        log::error!("Server returned status error {:?} while {}. Should be impossible. Aborting.", e, logging_msg.post_completion)
                    },
                }
                break;
            }
        };
    }
    log::error!("Hit retry limit or abort while {}, force aborting.", logging_msg.pre_completion);
    Err(())
}
