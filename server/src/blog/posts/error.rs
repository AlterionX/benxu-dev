use rocket::{response::status, http::Status};

    enum ConvError {
        Parse(uuid::ParseError),
        Version(usize),
    }
    impl From<uuid::ParseError> for ConvError {
        fn from(parse_err: uuid::ParseError) -> Self {
            Self::Parse(parse_err)
        }
    }
    impl From<usize> for ConvError {
        fn from(version_num: usize) -> Self {
            Self::Version(version_num)
        }
    }
    impl From<ConvError> for status::Custom<()> {
        fn from(e: ConvError) -> Self {
            status::Custom(Status::BadRequest, ())
        }
    }
