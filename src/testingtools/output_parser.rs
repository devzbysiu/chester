use crate::result::CoverageParseErr;
use crate::use_cases::output_parser::{OutputParser, Parser};

use std::sync::{Arc, Mutex};

pub fn working(coverage: f32) -> Parser<f32, CoverageParseErr> {
    WorkingOutputParser::make(coverage)
}

pub struct WorkingOutputParser {
    coverage: f32,
}

impl WorkingOutputParser {
    fn make(coverage: f32) -> Parser<f32, CoverageParseErr> {
        Arc::new(Mutex::new(Self { coverage }))
    }
}

impl OutputParser for WorkingOutputParser {
    type Output = f32;
    type Error = CoverageParseErr;

    fn parse(&self, _output: String) -> Result<Self::Output, Self::Error> {
        Ok(self.coverage)
    }
}

pub fn failing() -> Parser<f32, CoverageParseErr> {
    FailingOutputParser::make()
}

pub struct FailingOutputParser;

impl FailingOutputParser {
    fn make() -> Parser<f32, CoverageParseErr> {
        Arc::new(Mutex::new(Self))
    }
}

impl OutputParser for FailingOutputParser {
    type Output = f32;
    type Error = CoverageParseErr;

    fn parse(&self, _output: String) -> Result<Self::Output, Self::Error> {
        Err(CoverageParseErr::NoLastLine)
    }
}
