use std::fs;

use rustc_hash::FxHashSet;
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

	pub fn find_best_pattern(&self, weapon: &Weapon) -> (f32, f32, Vec<Pattern>) {
		let mut max_win = 0;
		let mut max_draw = 0;
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
			if results.is_empty() || win > max_win || (win == max_win && draw > max_draw) {
				max_win = win;
				max_draw = draw;
				results.clear();
				results.push(p1_pattern);
			} else if win == max_win && draw == max_draw {
				results.push(p1_pattern);
			}
		}
		// 最後に計算して指定の戻り値の型に変換する
		let den = self.patterns.len() as f32;
		let win_rate = max_win as f32 / den;
		let win_draw_rate = (max_win + max_draw) as f32 / den;
		(win_rate, win_draw_rate, results)
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
			let mut set = FxHashSet::default();
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
