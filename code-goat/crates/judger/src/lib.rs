mod ffi;
mod judger;
mod runner;
mod sandbox;
mod spec;

pub use ffi::{c_free, c_grade_output, c_judge};
pub use judger::judge;
pub use spec::{JudgeResult, ResourceLimit, RunSpec};
