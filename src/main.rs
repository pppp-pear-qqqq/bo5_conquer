mod actor;
mod error;
mod greedy;
mod licences;

use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};

use clap::{Parser, Subcommand};

use crate::greedy::greedy;

use self::actor::Actor;
use self::error::Error;
use self::licences::{Licences, SkillType, Weapon};

const ROUND: usize = 5;
type Pattern = [u8; ROUND];

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
	#[command(subcommand)]
	mode: Option<Mode>,
}
#[derive(Subcommand)]
enum Mode {
	Find {
		#[arg(short, long)]
		weapon: Option<String>,
		#[arg(short, long)]
		opponent: Option<String>,
		#[arg(short, long)]
		min_rate: Option<f32>,
	},
	Simulate {
		#[arg(short, long)]
		al_weapon: Option<String>,
		#[arg(short, long)]
		op_weapon: Option<String>,
		al_pattern: Option<String>,
		op_pattern: Option<String>,
	},
	AllFind {
		#[arg(short, long)]
		weapon: Option<String>,
		#[arg(long, default_value = "false")]
		overwrite: bool,
	},
	Greedy {
		#[arg(short, long)]
		weapon: String,
	},
	Registration,
}

fn main() -> Result<(), self::error::Error> {
	let prompt_input = |msg: &str| -> Result<String, io::Error> {
		print!("{}", msg);
		std::io::stdout().flush()?;
		let mut input = String::new();
		std::io::stdin().read_line(&mut input)?;
		Ok(input.trim().to_string())
	};

	let args = Args::parse();

	let dict = Licences::load("data/licences.json")?;
	println!("loaded {} weapons", dict.len());

	let mode = match args.mode {
		Some(mode) => mode,
		None => {
			let input = prompt_input("select mode([f]ind/[s]imulate/[a]ll-find): ")?;
			match input.as_str() {
				"f" | "find" => Mode::Find { weapon: None, opponent: None, min_rate: None },
				"s" | "simulate" => Mode::Simulate { al_weapon: None, op_weapon: None, al_pattern: None, op_pattern: None },
				"a" | "all-find" => Mode::AllFind { weapon: None, overwrite: false },
				_ => return Err(Error::InvalidInput(input)),
			}
		}
	};

	match mode {
		Mode::Find { weapon, opponent, min_rate } => {
			println!("find mode: weapon={:?} opponent={:?} min_rate={:?}", weapon, opponent, min_rate);
			let weapon = dict.get(&if let Some(weapon) = weapon { weapon } else { prompt_input("select your weapon: ")? })?;
			let opponent = Actor::load(format!("data/patterns/{}.json", if let Some(opponent) = opponent { opponent } else { prompt_input("select your opponent: ")? }), &dict)?;
			let min_rate = if let Some(min_rate) = min_rate { min_rate } else { prompt_input("min rate: ")?.parse().map_err(|_| Error::InvalidInput("{rate}".into()))? };

			let results = opponent.find_pattern(weapon, min_rate, true);
			if results.is_empty() {
				println!("pattern not found");
			} else {
				for (score, draw_rate, pattern) in results {
					print!("{:.2}({:.2})\t", score, draw_rate);
					for i in 0..pattern.len() {
						print!("{}", weapon.skill(pattern[i] as usize).name);
						if i < pattern.len() - 1 {
							print!(", ");
						}
					}
					println!();
				}
			}
			Ok(())
		}
		Mode::Simulate { al_weapon, op_weapon, al_pattern, op_pattern } => {
			let al_weapon = dict.get(&if let Some(al_weapon) = al_weapon { al_weapon } else { prompt_input("select your weapon: ")? })?;
			let op_weapon = dict.get(&if let Some(op_weapon) = op_weapon { op_weapon } else { prompt_input("select opponent weapon: ")? })?;
			let al_pattern = parse_pattern(&if let Some(al_pattern) = al_pattern { al_pattern } else { prompt_input("your pattern: ")? })?;
			let op_pattern = parse_pattern(&if let Some(op_pattern) = op_pattern { op_pattern } else { prompt_input("opponent pattern: ")? })?;

			let (p1_score, p2_score) = simulate_duel(&al_weapon, &al_pattern, op_weapon, &op_pattern, false);
			match p1_score.cmp(&p2_score) {
				std::cmp::Ordering::Greater => println!("win: left\t({} : {})", p1_score, p2_score),
				std::cmp::Ordering::Less => println!("win: right\t({} : {})", p1_score, p2_score),
				std::cmp::Ordering::Equal => println!("draw\t({} : {})", p1_score, p2_score),
			}
			Ok(())
		}
		Mode::AllFind { weapon, overwrite } => {
			let weapon = dict.get(&if let Some(weapon) = weapon { weapon } else { prompt_input("select weapon: ")? })?;
			let dir = format!("result/{}", weapon.id);
			fs::create_dir_all(&dir)?;
			for entry in fs::read_dir("data/patterns")? {
				let entry = entry?;
				let path = entry.path();
				if path.is_file() {
					match Actor::load(path, &dict) {
						Ok(opponent) => {
							if !overwrite && fs::exists(format!("{}/{}.csv", dir, opponent.eno))? {
								println!("{}: skipped", opponent.eno);
								continue;
							}
							let results = opponent.find_pattern(&weapon, 1.0, false);
							if results.is_empty() {
								println!("{}: pattern not found", opponent.eno);
							} else {
								let mut file = fs::File::create(format!("{}/{}.csv", dir, opponent.eno))?;
								for (_, _, pattern) in results {
									let line = pattern.into_iter().map(|p| weapon.skill(p as usize).name.clone()).collect::<Vec<_>>().join(",");
									println!("{line}");
									writeln!(file, "{line}")?;
								}
							}
						}
						Err(Error::WeaponNoData(id)) => println!("weapon undefined: {}", id),
						Err(err) => return Err(err),
					}
				}
			}
			Ok(())
		}
		Mode::Greedy { weapon } => greedy(format!("result/{weapon}")),
		Mode::Registration => todo!(),
	}
}

fn parse_pattern(input: &str) -> Result<Pattern, Error> {
	let parts: Vec<&str> = input.trim().split_whitespace().collect();
	if parts.len() != ROUND {
		return Err(Error::InvalidInput(format!("require {} numbers", ROUND)));
	}
	let mut pattern = [0; ROUND];
	for i in 0..ROUND {
		pattern[i] = parts[i].parse::<u8>().map_err(|e| Error::InvalidInput(e.to_string()))?;
	}
	Ok(pattern)
}

fn simulate_duel(p1_weapon: &Weapon, p1_pattern: &Pattern, p2_weapon: &Weapon, p2_pattern: &Pattern, quiet: bool) -> (i32, i32) {
	const VOLTAGE: [f32; 5] = [1.0, 1.1, 1.25, 1.5, 2.0];
	let (mut p1_voltage, mut p2_voltage) = (HashSet::new(), HashSet::new());
	let (mut p1_score, mut p2_score) = (0, 0);
	for i in 0..ROUND {
		let p1 = p1_weapon.skill(p1_pattern[i] as usize);
		let p2 = p2_weapon.skill(p2_pattern[i] as usize);
		let p1_damage = match p1.ty {
			SkillType::Sp | SkillType::Ex => (p1.atk as f32 * VOLTAGE[p1_voltage.len()]) as i32,
			_ => p1.atk,
		};
		let p2_damage = match p2.ty {
			SkillType::Sp | SkillType::Ex => (p2.atk as f32 * VOLTAGE[p2_voltage.len()]) as i32,
			_ => p2.atk,
		};
		let (p1_round_score, p2_round_score) = match p1.ty.cmp(&p2.ty) {
			std::cmp::Ordering::Greater => ((p1_damage - p2.def).max(0), 0),
			std::cmp::Ordering::Less => (0, (p2_damage - p1.def).max(0)),
			std::cmp::Ordering::Equal => ((p1_damage / 2 - p2.def).max(0), (p2_damage / 2 - p1.def).max(0)),
		};
		p1_score += p1_round_score;
		p2_score += p2_round_score;
		if !quiet {
			println!("round {}: {}\t| {}", i + 1, p1_round_score, p2_round_score);
		}
		p1_voltage.insert(p1.ty);
		p2_voltage.insert(p2.ty);
	}
	(p1_score, p2_score)
}
