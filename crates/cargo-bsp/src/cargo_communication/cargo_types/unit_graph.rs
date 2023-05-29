use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
pub struct UnitGraph {
    #[serde(rename = "version")]
    _version: i64,
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
    use crate::cargo_communication::cargo_types::unit_graph::{Unit, UnitGraph};
    use bsp_types::tests::test_deserialization;

    #[test]
    fn unit_graph() {
        let test_unit_graph = UnitGraph {
            _version: 1,
            units: vec![Unit::default(); 3],
        };

        test_deserialization(
            r#"{"version":1,"units":[{"data":"test_data"}, {"data":"test_data"}, {"data":"test_data"}],"roots":[0]}"#,
            &test_unit_graph,
        );
    }
}
