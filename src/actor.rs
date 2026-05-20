use std::collections::HashSet;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::command::duel;
use crate::licences::{Licences, Weapon};
use crate::{Error, Pattern, ROUND};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Eno {
	Int(i32),
	Str(String),
}

#[derive(Debug)]
pub struct Actor {
	pub eno: Eno,
	// pub name: String,
	weapon: Weapon,
	patterns: Vec<Pattern>,
}

impl std::fmt::Display for Eno {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Eno::Int(n) => write!(f, "{}", n),
			Eno::Str(s) => write!(f, "{}", s),
		}
	}
}

impl Actor {
	pub fn load<P: AsRef<std::path::Path>>(path: P, dict: &Licences) -> Result<Self, Error> {
		let data = fs::read_to_string(path)?;
		let player: ActorSerializeTemp = serde_json::from_str(&data)?;
		Ok(player.into_player(dict)?)
	}
	// pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
	// 	let data = serde_json::to_string(self)?;
	// 	fs::write(path, data)?;
	// 	Ok(())
	// }

	pub fn find_pattern(&self, weapon: &Weapon, min_score: f32, allow_draw: bool) -> Vec<(f32, f32, Pattern)> {
		let mut results = Vec::new();
		for p1_pattern in weapon.enumerate_skill_patterns() {
			let mut win = 0;
			let mut draw = 0;
			for p2_pattern in &self.patterns {
				let (p1_score, p2_score) = duel(weapon, &p1_pattern, &self.weapon, p2_pattern, true);
				if p1_score > p2_score {
					win += 1;
				} else if p1_score == p2_score {
					draw += 1;
				}
			}
			let den = self.patterns.len() as f32;
			results.push((win as f32 / den, (win + draw) as f32 / den, p1_pattern));
		}
		results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap().then_with(|| b.1.partial_cmp(&a.1).unwrap()));
		results.into_iter().filter(|(win, draw, _)| if !allow_draw { *win >= min_score } else { *draw >= min_score }).collect()
	}
}

#[derive(Debug, Deserialize, Serialize)]
struct ActorSerializeTemp {
	eno: Eno,
	// name: String,
	weapon: String,
	#[serde(default)]
	unlimited: bool,
	patterns: Vec<[PatternSerializeTemp; ROUND]>,
}
#[derive(Debug, Deserialize, Serialize)]
struct PatternSerializeTemp {
	#[serde(rename = "type")]
	ty: String,
	name: String,
}
impl ActorSerializeTemp {
	pub fn into_player(self, dict: &Licences) -> Result<Actor, Error> {
		let weapon = if self.unlimited {
			let mut set = HashSet::new();
			for p in &self.patterns {
				for s in p {
					set.insert(s.name.as_str());
				}
			}
			let mut skill_list = Vec::new();
			for s in &set {
				skill_list.push(dict.get_skill(s)?.clone());
			}
			Weapon::new(&self.weapon, &dict.get_weapon(&self.weapon)?.name, skill_list)
		} else {
			dict.get_weapon(&self.weapon)?.clone()
		};
		let mut patterns = Vec::new();
		for p in self.patterns {
			let mut pattern = [255; ROUND];
			for i in 0..ROUND {
				pattern[i] = weapon.skill_idx(&p[i].name)?;
			}
			patterns.push(pattern);
		}
		Ok(Actor {
			eno: self.eno,
			// name: self.name,
			weapon,
			patterns,
		})
	}
}
