use std::fmt::{self, Display, Formatter};
use rocket::{request::FromParam, http::RawStr};

#[get("/projects")]
pub fn get() -> &'static str {
    "Hello, projects"
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Project {
    RayTracer,
    ThermalLilette,
    Totality,
    Shatter,
    Bifrost,
}
impl<'a> FromParam<'a> for Project {
    type Error = ();
    fn from_param(param: &'a RawStr) -> Result<Self, Self::Error> {
        match &*(param.percent_decode().map_err(|_| ())?) {
            "ray-tracer" => Ok(Project::RayTracer),
            "totality" => Ok(Project::Totality),
            "shatter" => Ok(Project::Shatter),
            "bifrost" => Ok(Project::Bifrost),
            "thermal-lilette" => Ok(Project::ThermalLilette),
            _ => Err(()),
        }
    }
}
impl Display for Project {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Project::RayTracer => write!(f, "ray-tracer"),
            Project::ThermalLilette => write!(f, "thermal-lilette"),
            Project::Totality => write!(f, "totality"),
            Project::Shatter => write!(f, "shatter"),
            Project::Bifrost => write!(f, "Bifrost"),
        }
    }
}

pub mod project {
    use super::Project;
    #[get("/projects/<project>")]
    pub fn get(project: Project) -> String {
        format!("Hello, project {}", project)
    }
}

