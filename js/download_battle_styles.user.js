// ==UserScript==
// @name         戦闘設定保存
// @namespace    http://tampermonkey.net/
// @version      0.1.0
// @description  戦闘設定をダウンロードする　なんかこれ課金機能らしいんで大人しく課金してください
// @author       なしなし
// @match        https://wdrb.work/bo5/profile.php?eno=*&type=npc
// @icon         https://www.google.com/s2/favicons?sz=64&domain=wdrb.work/bo5
// @grant        none
// ==/UserScript==

(function () {
	'use strict';
	function bake(tagName, f) {
		const e = document.createElement(tagName);
		f(e);
		return e;
	}

	const list = document.querySelector('.style_list');
	if (list.lastElementChild.childElementCount <= 1) return;

	list.appendChild(bake('li', e => {
		e.appendChild(bake('button', e => {
			e.type = 'button';
			e.style.color = '#202020';
			e.style.borderColor = '#20202022';
			e.textContent = '戦闘設定をダウンロードする';
			e.addEventListener('click', () => {
				const id = new URLSearchParams(location.search).get('eno');
				const img = list.querySelector('img');
				const name = img.dataset.tippyContent;
				const weapon = img.src.match(/imgs\/w_(\w+).svg/)[1];
				const patterns = [];
				for (const e of list.children) {
					if (e === list.lastElementChild) continue;
					const style = [];
					e.querySelectorAll('p').forEach(p => {
						const type = p.querySelector('img').src.match(/imgs\/ic_(\w+).svg/)[1];
						const name = p.textContent;
						style.push({ type, name });
					});
					patterns.push(style);
				}
				const blob = new Blob([JSON.stringify({
					eno: id,
					name,
					weapon,
					patterns,
				})], { type: 'application/json' });
				const url = URL.createObjectURL(blob);
				bake('a', e => {
					e.href = url;
					e.download = `${id}.json`;
				}).click();
				URL.revokeObjectURL(url);
			});
		}))
	}))
})();
