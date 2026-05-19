#[derive(Debug)]
pub enum Error {
	Io(std::io::Error),
	Serde(serde_json::Error),
	WeaponUndefined(String),
	SkillUndefined(String),
	InvalidInput(String),
}

impl From<std::io::Error> for Error {
	fn from(e: std::io::Error) -> Self {
		Error::Io(e)
	}
}
impl From<serde_json::Error> for Error {
	fn from(e: serde_json::Error) -> Self {
		Error::Serde(e)
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::Io(e) => write!(f, "IO error: {}", e),
			Error::Serde(e) => write!(f, "Serde error: {}", e),
			Error::WeaponUndefined(e) => write!(f, "Weapon unauthorized({})", e),
			Error::SkillUndefined(e) => write!(f, "Skill undefined({})", e),
			Error::InvalidInput(e) => write!(f, "Invalid input({})", e),
		}
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Error::Io(e) => Some(e),
			Error::Serde(e) => Some(e),
			Error::WeaponUndefined(_) => None,
			Error::SkillUndefined(_) => None,
			Error::InvalidInput(_) => None,
		}
	}
}
