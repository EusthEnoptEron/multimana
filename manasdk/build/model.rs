use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::io::Read;
use std::str::FromStr;

use regex::Regex;

mod raw {
    use std::collections::HashMap;

    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct RawEnumDump {
        pub data: Vec<HashMap<String, (Vec<HashMap<String, i64>>, String)>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    pub enum FieldDefinition {
        InheritInfo(Vec<String>),
        MDKClassSize(usize),
        Field((FieldSignature, usize, usize, usize)),
        FieldWithBitOffset((FieldSignature, usize, usize, usize, i64)),
    }

    #[derive(Deserialize, Debug)]
    pub struct FieldSignature(pub String, pub String, pub String, pub Vec<FieldSignature>);

    #[derive(Deserialize, Debug)]
    struct FieldType(FieldSignature, usize, usize, usize);

    #[derive(Deserialize, Debug)]
    struct InheritInfo {
        #[serde(rename = "__InheritInfo")]
        pub inherit_info: Vec<String>,
    }

    #[derive(Deserialize, Debug)]
    struct MDKClassSize {
        #[serde(rename = "__MDKClassSize")]
        pub size: usize,
    }


    #[derive(Deserialize, Debug)]
    pub struct JsonData {
        pub data: Vec<HashMap<String, Vec<HashMap<String, FieldDefinition>>>>,
    }
}
