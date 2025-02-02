use std::collections::HashMap;

use serde::Deserialize;

// #[derive(Debug, Deserialize)]
// pub struct Output {
//     pub path: String,
// }

#[derive(Debug, Deserialize)]
pub struct InputDrv {
    // pub outputs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Derivation {
    #[serde(rename = "inputDrvs")]
    pub dependencies: HashMap<String, InputDrv>,
    // pub outputs: HashMap<String, Output>,
}

#[derive(Debug, Deserialize)]
pub struct RecursiveDerivation(pub HashMap<String, Derivation>);
