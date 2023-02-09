extern crate core;

mod openai_call;
mod logger;



//
//
// ------------------------------------------------------------------------------------------------
// Test
//
#[cfg(test)]
mod tests {
    use super::*;

}
//
//
// ------------------------------------------------------------------------------------------------
// Error
//

use error_stack::{Context,Report};

#[derive(Debug,Copy, Clone)]
pub enum EGeneral {

    LogSys


}
//
impl std::fmt::Display for EGeneral {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        match self {

            Self::LogSys =>     write!(f, "Log System Error"),

        }


    }


}
//
impl Context for EGeneral {}
//
impl EGeneral {

    pub(crate) fn as_report(&self) -> Report<Self> { Report::new(*self) }


}