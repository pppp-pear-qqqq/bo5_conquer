mod actor;
mod error;
mod greedy;
mod licences;

use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::str::FromStr;

use clap::{Parser, Subcommand};

use self::actor::Actor;
use self::error::Error;
use self::greedy::greedy;
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
		#[arg(short, long)]
		out_file: Option<String>,
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
		#[arg(short, long)]
		out_file: Option<String>,
	},
	Simulate {
		#[arg(short, long)]
		al_weapon: Option<String>,
		#[arg(short, long)]
		op_weapon: Option<String>,
		al_pattern: Option<String>,
		op_pattern: Option<String>,
	},
	Registration,
}

fn main() -> Result<(), self::error::Error> {
	let args = Args::parse();

	// ライセンス読み込み
	let dict = Licences::load("data/licences.json")?;
	println!("loaded {} weapons", dict.len());

	match args.mode.unwrap_or_input("select mode([f]ind/[s]imulate/[a]ll-find): ")? {
		// 1件探索
		Mode::Find { weapon, opponent, min_rate, out_file } => {
			// 引数処理
			let weapon = dict.get_weapon(&weapon.unwrap_or_input("select your weapon: ")?)?;
			let opponent = Actor::load(format!("data/patterns/{}.json", opponent.unwrap_or_input("select your opponent: ")?), &dict)?;
			let min_rate = min_rate.unwrap_or_input("min rate: ")?;

			// 探索
			let results = opponent.find_pattern(weapon, min_rate, true);

			// 表示
			if results.is_empty() {
				println!("pattern not found");
			} else {
				let mut file = if let Some(path) = out_file { Some(fs::File::create(path)?) } else { None };
				for (win, draw, pattern) in results {
					let line = pattern.into_iter().map(|p| weapon.skill(p as usize).name.clone()).collect::<Vec<_>>().join(",");
					println!("{win:.2}({draw:.2})\t{line}");
					if let Some(file) = &mut file {
						writeln!(file, "{win:.2},{draw:.2},{line}")?;
					}
				}
			}

			Ok(())
		}
		// 全件探索
		Mode::AllFind { weapon, overwrite } => {
			// 引数処理
			let weapon = dict.get_weapon(&weapon.unwrap_or_input("select your weapon: ")?)?;

			// 対象ディレクトリ作成
			let dir = format!("result/{}", weapon.id);
			fs::create_dir_all(&dir)?;

			// 全件探索
			for entry in fs::read_dir("data/patterns")? {
				let path = entry?.path();
				if path.is_file() {
					let opponent = Actor::load(path, &dict)?;
					if !overwrite && fs::exists(format!("{}/{}.csv", dir, opponent.eno))? {
						continue;
					}
					let results = opponent.find_pattern(&weapon, 1.0, false);
					if results.is_empty() {
						println!("{}: pattern not found", opponent.eno);
					} else {
						println!("{}: ok", opponent.eno);
						let mut file = fs::File::create(format!("{}/{}.csv", dir, opponent.eno))?;
						for (win, draw, pattern) in results {
							let line = pattern.into_iter().map(|p| weapon.skill(p as usize).name.clone()).collect::<Vec<_>>().join(",");
							writeln!(file, "{win:.2},{draw:.2},{line}")?;
						}
					}
				}
			}

			Ok(())
		}
		// 一貫するパターン探索
		Mode::Greedy { weapon, out_file } => {
			greedy(format!("result/{weapon}"), out_file)?;

			Ok(())
		}
		// 闘技シミュレーション
		Mode::Simulate {
			al_weapon,
			op_weapon,
			al_pattern,
			op_pattern,
		} => {
			// 引数処理
			let al_weapon = dict.get_weapon(&al_weapon.unwrap_or_input("select your weapon: ")?)?;
			let op_weapon = dict.get_weapon(&op_weapon.unwrap_or_input("select opponent weapon: ")?)?;
			let al_pattern = parse_pattern(&al_pattern.unwrap_or_input("your pattern: ")?)?;
			let op_pattern = parse_pattern(&op_pattern.unwrap_or_input("opponent pattern: ")?)?;

			// シミュレート実行
			let (p1_score, p2_score) = simulate_duel(&al_weapon, &al_pattern, op_weapon, &op_pattern, false);
			match p1_score.cmp(&p2_score) {
				std::cmp::Ordering::Greater => println!("win: left\t({} : {})", p1_score, p2_score),
				std::cmp::Ordering::Less => println!("win: right\t({} : {})", p1_score, p2_score),
				std::cmp::Ordering::Equal => println!("draw\t({} : {})", p1_score, p2_score),
			}

			Ok(())
		}
		// データ登録
		Mode::Registration => todo!(),
	}
}

trait OptionUnwrap {
	type Output;
	fn unwrap_or_input(self, prompt: &str) -> Result<Self::Output, Error>;
}
impl<T: FromStr> OptionUnwrap for Option<T> {
	type Output = T;
	fn unwrap_or_input(self, prompt: &str) -> Result<T, Error> {
		match self {
			Some(value) => Ok(value),
			None => {
				print!("{}", prompt);
				std::io::stdout().flush()?;
				let mut input = String::new();
				std::io::stdin().read_line(&mut input)?;
				Ok(input.trim().parse::<T>().map_err(|_| Error::InvalidInput(input.into()))?)
			}
		}
	}
}

impl FromStr for Mode {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"f" | "find" => Self::Find {
				weapon: None,
				opponent: None,
				min_rate: None,
				out_file: None,
			},
			"s" | "simulate" => Self::Simulate {
				al_weapon: None,
				op_weapon: None,
				al_pattern: None,
				op_pattern: None,
			},
			"a" | "all-find" => Self::AllFind { weapon: None, overwrite: false },
			_ => return Err(Error::InvalidInput(s.into())),
		})
	}
}

fn parse_pattern(input: &str) -> Result<Pattern, Error> {
	let parts: Vec<&str> = input.trim().split_whitespace().collect();
	if parts.len() != ROUND {
		return Err(Error::InvalidInput(format!("require {} numbers", ROUND)));
	}
	let mut pattern = [u8::MAX; ROUND];
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
