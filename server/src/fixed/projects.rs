//! Handle project and project demo code together.
//!
//!

use std::fmt::{self, Display, Formatter};
use rocket::{request::FromParam, http::{RawStr, Status}};

/// Returns the "projects" page, which will have brief demos or videos (eventually). Not yet
/// implemented.
#[get("/projects")]
pub fn get() -> Status {
    Status::NotImplemented
}

/// Enum representing all possible projects to query for.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Project {
    /// A simple ray tracer, written with C++ and FLTK.
    ///
    /// Can be found on [github](https://github.com/AlterionX/cs378hgraphics-raytracer).
    RayTracer,
    /// A gas simulation model, written with C++, GLFW, GLUT, and OpenGL.
    ///
    /// Can be found on [github](https://github.com/AlterionX/thermal-lilette).
    ThermalLilette,
    /// A graphics/physics engine aiming to model the shattering of trimeshes and tearing of
    /// soft bodies, written with Rust, and gfx-hal.
    ///
    /// Can be found on [github](https://github.com/AlterionX/totality-rs).
    Totality,
    /// A physics engine modeling the shattering of trimeshes.
    ///
    /// Can be found on [github](https://github.com/AlterionX/physical-sim/tree/master/final-project).
    Shatter,
    /// A modular compiler.
    ///
    /// Can be found on [github](https://github.com/AlterionX/Bifrost).
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

/// All handlers for handling per-project information.
pub mod project {
    use super::Project;
    use rocket::http::Status;

    /// Retrieve information for an individual project.
    #[get("/projects/<_project>")]
    pub fn get(_project: Project) -> Status {
        Status::NotImplemented
    }
}

