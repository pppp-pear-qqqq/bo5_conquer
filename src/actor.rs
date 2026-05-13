use std::fs;

use serde::{Deserialize, Serialize};

use crate::licences::{Licences, Weapon};
use crate::{Pattern, simulate_duel};

#[derive(Debug)]
pub struct Actor {
	// eno: i32,
	// name: String,
	weapon: Option<Weapon>,
	patterns: Vec<Pattern>,
}

impl Actor {
	pub fn load<P: AsRef<std::path::Path>>(path: P, dict: &Licences) -> Result<Self, Box<dyn std::error::Error>> {
		let data = fs::read_to_string(path)?;
		let player: ActorSerializeTemp = serde_json::from_str(&data)?;
		Ok(player.into_player(dict))
	}
	// pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
	// 	let data = serde_json::to_string(self)?;
	// 	fs::write(path, data)?;
	// 	Ok(())
	// }

	pub fn find_pattern(&self, weapon: &Weapon, min_score: f32) -> Vec<(f32, Pattern)> {
		let mut results = Vec::new();
		if let Some(p2_weapon) = &self.weapon {
			for p1_pattern in weapon.enumerate_skill_patterns() {
				let mut win = 0;
				for p2_pattern in &self.patterns {
					let (result, _, _) = simulate_duel(weapon, &p1_pattern, p2_weapon, p2_pattern);
					if result {
						win += 1;
					}
				}
				results.push((win as f32 / self.patterns.len() as f32, p1_pattern));
			}
		}
		results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
		results.into_iter().filter(|(score, _)| *score >= min_score).collect()
	}
}

#[derive(Debug, Deserialize, Serialize)]
struct ActorSerializeTemp {
	eno: i32,
	name: String,
	weapon: String,
	patterns: Vec<Pattern>,
}
impl ActorSerializeTemp {
	pub fn into_player(self, dict: &Licences) -> Actor {
		Actor {
			// eno: self.eno,
			// name: self.name,
			weapon: dict.get(&self.weapon).cloned(),
			patterns: self.patterns,
		}
	}
}
