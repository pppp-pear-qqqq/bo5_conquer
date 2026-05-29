use crate::actor::Actor;
use crate::licences::{Licences, SkillType, Weapon};
use crate::{Error, Output, Pattern, ROUND};
use rustc_hash::{FxHashMap, FxHashSet};

pub fn duel(p1_weapon: &Weapon, p1_pattern: &Pattern, p2_weapon: &Weapon, p2_pattern: &Pattern, quiet: bool) -> (i32, i32) {
	const VOLTAGE: [f32; 5] = [1.0, 1.1, 1.25, 1.5, 2.0];
	let (mut p1_voltage, mut p2_voltage) = (FxHashSet::default(), FxHashSet::default());
	let (mut p1_score, mut p2_score) = (0, 0);
	for i in 0..ROUND {
		let p1 = p1_weapon.skill(p1_pattern[i] as usize);
		let p2 = p2_weapon.skill(p2_pattern[i] as usize);
		p1_voltage.insert(p1.ty);
		p2_voltage.insert(p2.ty);
		let p1_damage = match p1.ty {
			SkillType::Sp | SkillType::Ex => (p1.atk as f32 * VOLTAGE[p1_voltage.len() - 1]) as i32,
			_ => p1.atk,
		};
		let p2_damage = match p2.ty {
			SkillType::Sp | SkillType::Ex => (p2.atk as f32 * VOLTAGE[p2_voltage.len() - 1]) as i32,
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
	}
	(p1_score, p2_score)
}

pub fn find(out: Output, weapon: &Weapon, opponent: Actor) -> Result<(), Error> {
	let (win, draw, results) = opponent.find_best_pattern(weapon);

	// 表示
	if results.is_empty() {
		println!("pattern not found");
	} else {
		println!("ok");
		out.make_dir()?;
		let mut out = out.gen_write(&format!("{}.csv", opponent.eno))?;
		for pattern in results {
			let line = pattern.into_iter().map(|p| weapon.skill(p as usize).name.clone()).collect::<Vec<_>>().join(",");
			writeln!(out, "{:03.0},{:03.0},{line}", win * 100.0, draw * 100.0)?;
		}
	}

	Ok(())
}

pub fn find_all(out: Output, dict: &Licences, weapon: &Weapon, input_dir: String) -> Result<(), Error> {
	out.make_dir()?;
	// 全件探索
	for entry in std::fs::read_dir(&input_dir)? {
		let path = entry?.path();
		if path.is_file() {
			let opponent = Actor::load(path, &dict)?;
			let (win, draw, results) = opponent.find_best_pattern(&weapon);
			if results.is_empty() {
				println!("{}: pattern not found", opponent.eno);
			} else {
				let mut out = match out.gen_write(&format!("{}.csv", opponent.eno)) {
					Ok(out) => {
						println!("{}: ok", opponent.eno);
						out
					}
					Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
					Err(err) => return Err(Error::Io(err)),
				};
				for pattern in results {
					let line = pattern.into_iter().map(|p| weapon.skill(p as usize).name.clone()).collect::<Vec<_>>().join(",");
					writeln!(out, "{:03.0},{:03.0},{line}", win * 100.0, draw * 100.0)?;
				}
			}
		}
	}
	Ok(())
}

pub fn consistents(out: Output, weapon: &Weapon, input_dir: String, recursive: bool) -> Result<(), Error> {
	const UNCERTAIN_COLOR: &str = "#DB99FF";
	let (opponent_matrix, all_opponents) = match consistents_load(input_dir, recursive)? {
		Some(result) => result,
		None => {
			println!("files not found");
			return Ok(());
		}
	};
	println!("load complete: {}", all_opponents.len());

	let mut perfect_coverage: FxHashMap<String, FxHashSet<String>> = FxHashMap::default();
	for (opponent, records) in &opponent_matrix {
		for rec in records {
			perfect_coverage.entry(rec.clone()).or_insert_with(FxHashSet::default).insert(opponent.clone());
		}
	}

	let mut uncovered = all_opponents.clone();
	let mut selected_perfect_patterns: Vec<(String, Vec<String>)> = Vec::new();

	while !uncovered.is_empty() {
		let mut best_pattern: Option<String> = None;
		let mut best_covered_opponents: FxHashSet<String> = FxHashSet::default();
		let mut max_count = 0;

		for (pattern, covered_set) in &perfect_coverage {
			let current_covered: FxHashSet<String> = covered_set.intersection(&uncovered).cloned().collect();
			if current_covered.len() > max_count {
				max_count = current_covered.len();
				best_pattern = Some(pattern.clone());
				best_covered_opponents = current_covered;
			}
		}

		if max_count == 0 {
			break;
		}

		let chosen_pattern = best_pattern.unwrap();
		for op in &best_covered_opponents {
			uncovered.remove(op);
		}

		let mut covered_list: Vec<String> = best_covered_opponents.into_iter().collect();
		covered_list.sort();
		selected_perfect_patterns.push((chosen_pattern, covered_list));
	}

	// 出力先の用意
	out.make_dir()?;
	let mut out = out.gen_write("results.json")?;

	// 出力
	let mut is_first = true;
	write!(out, "[")?;
	for (pattern, targets) in selected_perfect_patterns {
		if is_first {
			is_first = false;
		} else {
			write!(out, ",")?;
		}
		let mut iter = pattern.split(',');
		let win = iter.next().unwrap_or("");
		let draw = iter.next().unwrap_or("");
		write!(out, "{{\"w_id_slug\":\"{}\",\"btst_name\":\"{win}|{}\"", weapon.id, targets.join(","))?;
		if !(win == "100" || draw == "100") {
			write!(out, ",\"color\":\"{UNCERTAIN_COLOR}\"")?;
		}
		for (idx, part) in iter.enumerate() {
			write!(out, ",\"skill_r{}\":\"{}_{:02}\"", idx + 1, weapon.id, weapon.skill_idx(part)? + 1)?;
		}
		write!(out, "}}")?;
	}
	write!(out, "]")?;
	out.flush()?;
	Ok(())
}
fn consistents_load<P: AsRef<std::path::Path>>(dir_path: P, recursive: bool) -> Result<Option<(FxHashMap<String, Vec<String>>, FxHashSet<String>)>, std::io::Error> {
	let mut opponent_matrix: FxHashMap<String, Vec<String>> = FxHashMap::default();
	let mut all_opponents = FxHashSet::default();

	// 探索予定のディレクトリを保持するスタック
	let mut dirs_to_visit = vec![dir_path.as_ref().to_path_buf()];

	// スタックにディレクトリが残っている限りループ
	while let Some(current_dir) = dirs_to_visit.pop() {
		let entries = std::fs::read_dir(current_dir)?;

		for entry in entries {
			let path = entry?.path();

			if path.is_file() {
				let opponent_name = path.file_stem().and_then(|s| s.to_str()).unwrap().to_string();
				all_opponents.insert(opponent_name.clone());

				let content = std::fs::read_to_string(&path)?;
				let mut records = Vec::new();

				for line in content.lines() {
					let line = line.trim();
					if line.is_empty() {
						continue;
					}
					// let fields: Vec<&str> = line.split(',').collect();
					// if fields.len() != 7 {
					// 	continue;
					// }
					// let filtered_fields: String = fields.into_iter().skip(2).collect::<Vec<&str>>().join(",");
					// records.push(filtered_fields);
					records.push(line.to_string());
				}

				opponent_matrix.insert(opponent_name, records);
			} else if path.is_dir() && recursive {
				// ディレクトリかつ再帰モードがtrueなら、スタックに追加して後で探索する
				dirs_to_visit.push(path);
			}
		}
	}

	if all_opponents.is_empty() {
		return Ok(None);
	}

	Ok(Some((opponent_matrix, all_opponents)))
}

// pub fn simulate(out: Output, al_weapon: &Weapon, op_weapon: &Weapon, al_pattern: Pattern, op_pattern: Pattern) -> Result<(), Error> {
// 	todo!();
// }
