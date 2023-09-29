//! Unit graph structure, used to store results of cargo command with
//! `--unit-graph` flag.
//!
//! The only information we need from unit graph is the number of units, so
//! we do not have to store the whole structure that we obtain from the Cargo command.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
pub struct UnitGraph {
    r#version: i64,
    units: Vec<Unit>,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
pub struct Unit {}

impl UnitGraph {
    pub fn get_compilation_steps(&self) -> i64 {
        self.units.len() as i64
    }
}

#[cfg(test)]
mod tests {
    use crate::cargo_communication::execution::execution_types::unit_graph::{Unit, UnitGraph};
    use bsp_types::tests::test_deserialization;
    use insta::assert_json_snapshot;

    #[test]
    fn unit_graph() {
        let test_unit_graph = UnitGraph {
            r#version: 1,
            units: vec![Unit::default(); 3],
        };

        test_deserialization(
            r#"{"version":1,"units":[{"data":"test_data"}, {"data":"test_data"}, {"data":"test_data"}],"roots":[0]}"#,
            &test_unit_graph,
        );

        assert_json_snapshot!(UnitGraph::default(),
            @r#"
        {
          "version": 0,
          "units": []
        }
        "#
        );
    }
}
