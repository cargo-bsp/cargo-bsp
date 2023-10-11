use crate::*;

#[derive(Debug)]
pub enum WorkspaceLibraries {}

impl Request for WorkspaceLibraries {
    type Params = ();
    type Result = WorkspaceLibrariesResult;
    const METHOD: &'static str = "workspace/libraries";
}

#[derive(Debug)]
pub enum WorkspaceDirectories {}

impl Request for WorkspaceDirectories {
    type Params = ();
    type Result = WorkspaceDirectoriesResult;
    const METHOD: &'static str = "workspace/directories";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_libraries_method() {
        assert_eq!(WorkspaceLibraries::METHOD, "workspace/libraries");
    }
}
