use crate::bsp_types::notifications::Notification;

#[derive(Debug)]
pub enum ExitBuild {}

impl Notification for ExitBuild {
    type Params = ();
    const METHOD: &'static str = "build/exit";
}
