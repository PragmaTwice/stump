use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryListingInput {
	pub path: Option<String>,
}

impl Default for DirectoryListingInput {
	fn default() -> Self {
		Self {
			path: Some("/".to_string()),
		}
	}
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryListing {
	pub parent: Option<String>,
	pub directories: Vec<FsPath>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]

pub struct FsPath {
	pub is_directory: bool,
	pub name: String,
	pub path: String,
}

impl FsPath {
	pub fn new(is_directory: bool, name: &str, path: &str) -> FsPath {
		FsPath {
			is_directory,
			name: name.to_string(),
			path: path.to_string(),
		}
	}

	pub fn file(name: &str, path: &str) -> FsPath {
		FsPath::new(false, name, path)
	}

	pub fn directory(name: &str, path: &str) -> FsPath {
		FsPath::new(true, name, path)
	}
}
