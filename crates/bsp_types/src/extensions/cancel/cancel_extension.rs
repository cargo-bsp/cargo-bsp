use crate::extensions::CancelRequestParams;
use crate::notifications::Notification;

/// Like the language server protocol, a notification to ask the server to cancel a request.
#[derive(Debug)]
pub enum CancelRequest {}

impl Notification for CancelRequest {
    type Params = CancelRequestParams;
    const METHOD: &'static str = "$/cancelRequest";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancel() {
        assert_eq!(CancelRequest::METHOD, "$/cancelRequest");
    }
}
