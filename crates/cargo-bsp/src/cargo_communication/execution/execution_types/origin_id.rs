//! OriginId trait implementation for the Compile/Run/TestParams. The trait allows getting
//! origin id regardless if it is the compile, run or test request.

use bsp4rs::bsp::Identifier;
use bsp4rs::bsp::{CompileParams, RunParams, TestParams};

pub trait WithOriginId {
    fn origin_id(&self) -> Option<Identifier>;
}

impl WithOriginId for CompileParams {
    fn origin_id(&self) -> Option<Identifier> {
        self.origin_id.clone()
    }
}

impl WithOriginId for RunParams {
    fn origin_id(&self) -> Option<Identifier> {
        self.origin_id.clone()
    }
}

impl WithOriginId for TestParams {
    fn origin_id(&self) -> Option<Identifier> {
        self.origin_id.clone()
    }
}
