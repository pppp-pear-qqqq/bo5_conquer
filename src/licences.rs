use std::fs;

use serde::{Deserialize, Serialize};

use crate::{Error, Pattern, ROUND};

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Licences(pub Vec<Weapon>);
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Weapon {
	#[serde(rename = "key")]
	pub id: String,
	pub name: String,
	skill_list: Vec<Skill>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Skill {
	pub name: String,
	#[serde(rename = "type")]
	pub ty: SkillType,
	pub atk: i32,
	pub def: i32,
	pub slash: i32,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillType {
	Ov,
	Md,
	Un,
	Sp,
	Ex,
}

impl Ord for SkillType {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		match self {
			Self::Ov => match other {
				Self::Ov => std::cmp::Ordering::Equal,
				Self::Md => std::cmp::Ordering::Greater,
				Self::Un => std::cmp::Ordering::Less,
				Self::Sp => std::cmp::Ordering::Less,
				Self::Ex => std::cmp::Ordering::Greater,
			},
			Self::Md => match other {
				Self::Ov => std::cmp::Ordering::Less,
				Self::Md => std::cmp::Ordering::Equal,
				Self::Un => std::cmp::Ordering::Greater,
				Self::Sp => std::cmp::Ordering::Less,
				Self::Ex => std::cmp::Ordering::Greater,
			},
			Self::Un => match other {
				Self::Ov => std::cmp::Ordering::Greater,
				Self::Md => std::cmp::Ordering::Less,
				Self::Un => std::cmp::Ordering::Equal,
				Self::Sp => std::cmp::Ordering::Less,
				Self::Ex => std::cmp::Ordering::Greater,
			},
			Self::Sp => match other {
				Self::Ov => std::cmp::Ordering::Greater,
				Self::Md => std::cmp::Ordering::Greater,
				Self::Un => std::cmp::Ordering::Greater,
				Self::Sp => std::cmp::Ordering::Equal,
				Self::Ex => std::cmp::Ordering::Less,
			},
			Self::Ex => match other {
				Self::Ov => std::cmp::Ordering::Less,
				Self::Md => std::cmp::Ordering::Less,
				Self::Un => std::cmp::Ordering::Less,
				Self::Sp => std::cmp::Ordering::Greater,
				Self::Ex => std::cmp::Ordering::Equal,
			},
		}
	}
}

impl Licences {
	pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
		let data = fs::read_to_string(path)?;
		Ok(serde_json::from_str(&data)?)
	}
	pub fn len(&self) -> usize {
		self.0.len()
	}
	pub fn get(&self, id: &str) -> Result<&Weapon, Error> {
		self.0.iter().find(|w| w.id == id).ok_or(Error::WeaponNoData(id.to_string()))
	}
}

impl Weapon {
	pub fn skill(&self, idx: usize) -> &Skill {
		&self.skill_list.get(idx).expect("error: skill_idx out of range")
	}

	pub fn skill_by_name(&self, name: &str) -> Result<u8, Error> {
		self.skill_list.iter().position(|s| s.name == name).map(|idx| idx as u8).ok_or(Error::SkillUndefined(name.into()))
	}

	pub fn enumerate_skill_patterns(&self) -> Vec<Pattern> {
		let mut results = Vec::new();
		let mut current_pattern = [255; ROUND];
		let mut stocks = self.skill_list.iter().map(|s| s.slash).collect::<Vec<_>>();
		Self::find_patterns(&mut results, &mut current_pattern, &mut stocks, 0);
		results
	}

	fn find_patterns(results: &mut Vec<Pattern>, current_pattern: &mut Pattern, stocks: &mut Vec<i32>, depth: usize) {
		if depth == ROUND {
			results.push(*current_pattern);
			return;
		}
		for i in 0..ROUND {
			if stocks[i] > 0 {
				current_pattern[depth] = i as u8;
				stocks[i] -= 1;
				Self::find_patterns(results, current_pattern, stocks, depth + 1);
				stocks[i] += 1;
			}
		}
	}
}
