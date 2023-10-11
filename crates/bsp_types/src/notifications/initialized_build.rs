use crate::notifications::Notification;

/// Like the language server protocol, the initialized notification is sent from the
/// client to the server after the client received the result of the initialize
/// request but before the client is sending any other request or notification to
/// the server. The server can use the initialized notification for example to
/// initialize intensive computation such as dependency resolution or compilation.
/// The initialized notification may only be sent once.
#[derive(Debug)]
pub enum OnBuildInitialized {}

impl Notification for OnBuildInitialized {
    type Params = ();
    const METHOD: &'static str = "build/initialized";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialized_build_method() {
        assert_eq!(OnBuildInitialized::METHOD, "build/initialized");
    }
}
