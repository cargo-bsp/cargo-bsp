//! OriginId trait implementation for the Compile/Run/TestParams. The trait allows getting
//! origin id regardless if it is the compile, run or test request.

use bsp_types::requests::{CompileParams, RunParams, TestParams};
use bsp_types::Identifier;

pub trait OriginId {
    fn origin_id(&self) -> Option<Identifier>;
}

impl OriginId for CompileParams {
    fn origin_id(&self) -> Option<Identifier> {
        self.origin_id.clone()
    }
}

impl OriginId for RunParams {
    fn origin_id(&self) -> Option<Identifier> {
        self.origin_id.clone()
    }
}

impl OriginId for TestParams {
    fn origin_id(&self) -> Option<Identifier> {
        self.origin_id.clone()
    }
}
