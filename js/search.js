(() => {
	let id = 0;
	let list = [];
	top: while (id <= 100) {
		id += 1;
		const e = document.getElementById(id);
		if (!e) continue;
		const key = e.querySelector('img').src.match(/imgs\/w_(.+).svg/)[1];
		const category = e.querySelector('[class*="wptag_"]').className.match(/wptag_(.+)/)[1];
		const name = e.querySelector('b').firstChild.textContent.trim();
		const desc = e.querySelector('p').textContent.trim();
		const price = Number(e.querySelector('img[src="imgs/fm_1.svg"]+span').textContent.trim().replaceAll(',', ''));
		const skills = e.querySelector('ul');
		const skill_list = [];
		for (const skill of skills.children) {
			const type = skill.querySelector('img').src.match(/imgs\/ic_(.+).svg/)[1];
			const name = skill.querySelector('b').textContent.trim();
			const desc = skill.querySelector('p').textContent.trim();
			const atk = Number(skill.querySelector('.red').textContent.trim());
			const def = Number(skill.querySelector('.blue').textContent.trim());
			const slash = Number(skill.querySelector('.gray').textContent.trim());
			if (Number.isNaN(atk) || Number.isNaN(def) || Number.isNaN(slash)) continue top;
			skill_list.push({ type, name, desc, atk, def, slash });
		}
		list.push({ id, id_name: key, category, name, desc, skill_list, price });
	}
	console.table(list);
})();
