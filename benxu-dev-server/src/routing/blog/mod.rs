use rocket::Route;

#[get("/")]
pub fn get() -> &'static str {
    "hi there, this will eventually be a blog page"
}

pub mod posts {
    use rocket::{response::status, http::Status};
    #[post("/posts")]
    pub fn post() -> status::Custom<()> {
        status::Custom(Status::new(501, "Not yet implemented"), ())
    }
    pub mod post {
        use rocket::{response::status, http::{RawStr, Status}};
        #[get("/posts/<id>")]
        pub fn get(id: &RawStr) -> String {
            format!("blog post {}! eventually", id)
        }
        #[patch("/posts/<id>")]
        pub fn patch(id: &RawStr) -> status::Custom<()> {
            status::Custom(Status::new(501, "Not yet implemented"), ())
        }
        #[delete("/posts/<id>")]
        pub fn delete(id: &RawStr) -> status::Custom<()> {
            status::Custom(Status::new(501, "Not yet implemented"), ())
        }
    }
}

pub mod editor {
    #[get("/editor")]
    pub fn get() -> &'static str {
        "editor, eventually"
    }
}

pub mod accounts {
    use rocket::{response::status, http::Status};
    #[post("/accounts")]
    pub fn post() -> status::Custom<()> {
        status::Custom(Status::new(501, "Not yet implemented"), ())
    }
    pub mod account {
        use rocket::{response::status, http::{RawStr, Status}};
        #[get("/accounts/<id>")]
        pub fn get(id: &RawStr) {
        }
        #[patch("/accounts/<id>")]
        pub fn patch(id: &RawStr) -> status::Custom<()> {
            status::Custom(Status::new(501, "Not yet implemented"), ())
        }
        #[delete("/accounts/<id>")]
        pub fn delete(id: &RawStr) -> status::Custom<()> {
            status::Custom(Status::new(501, "Not yet implemented"), ())
        }
    }
}

pub mod auth {
    use rocket::{response::status, http::Status};
    #[get("/login")]
    pub fn get() -> &'static str {
        "a login screen, eventually"
    }
    #[post("/login")]
    pub fn post() -> status::Custom<()> {
        status::Custom(Status::new(501, "Not yet implemented"), ())
    }
    #[delete("/login")]
    pub fn delete() -> status::Custom<()> {
        status::Custom(Status::new(501, "Not yet implemented"), ())
    }
}


pub fn routes() -> Vec<Route> {
    routes![
        get,
        posts::post,
        posts::post::get,
        posts::post::patch,
        posts::post::delete,
        editor::get,
        accounts::post,
        accounts::account::get,
        accounts::account::patch,
        accounts::account::delete,
        auth::get,
        auth::post,
        auth::delete,
    ]
}
