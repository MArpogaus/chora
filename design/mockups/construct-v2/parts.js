// Shared bits of the construct v2 mocks: veins, ring, pointer, keycaps.
const A = '#2680B4';
const RINGS = {
  listening: `<circle cx="12" cy="12" r="9" fill="none" stroke="${A}" stroke-opacity="0.35" stroke-width="2"/><circle cx="12" cy="12" r="4" fill="${A}"/>`,
  working: `<circle cx="12" cy="12" r="9" fill="none" stroke="${A}" stroke-opacity="0.22" stroke-width="2.5"/><path d="M12 3a9 9 0 0 1 9 9" fill="none" stroke="${A}" stroke-width="2.5" stroke-linecap="round"/>`,
};
const VEINS = `<svg class="veins" viewBox="0 0 1280 400"><path d="M-20 140 l90 -6 l70 14 l110 -10 l60 18 l140 -8 l90 -22 l120 6 l80 -14 l160 20 l120 -6 l140 10"/><path d="M120 400 l40 -60 l70 -30 l50 -42 l110 -20 l80 10"/><path d="M700 400 l30 -50 l90 -40 l120 -6 l70 -30 l160 10 l140 -24"/><path d="M420 60 l70 8 l50 -10 l90 16 l40 -6"/><path d="M880 200 l60 12 l90 -4 l70 22 l120 6"/><path d="M40 260 l80 -10 l60 20 l90 -6"/></svg>`;
const POINTER = (x, y, theme) => `<svg class="pointer" style="left:${x}px;top:${y}px" viewBox="0 0 18 26"><path d="M1 1 L1 20 L6 15.5 L9.5 23.5 L12.5 22.2 L9.2 14.4 L15.5 14.4 Z" fill="${theme === 'dark' ? '#fff' : '#111'}" stroke="${theme === 'dark' ? '#111' : '#fff'}" stroke-width="1.4" stroke-linejoin="round"/></svg>`;
const caps = (...k) => `<span class="keys">${k.map(x => `<span class="key">${x}</span>`).join('')}</span>`;
function space() {
  document.body.insertAdjacentHTML('afterbegin', `<div class="space"><div class="horizon"></div>${VEINS}</div>`);
}
function island(kind, word) {
  document.body.insertAdjacentHTML('beforeend', `<div id="island" class="glass pill"><svg class="ring" viewBox="0 0 24 24">${RINGS[kind]}</svg><span class="label" style="color:var(--text)">${word}</span></div>`);
}
function hints(items) {
  const parts = items.map(it => it === '|' ? '<i class="sep"></i>' :
    `<span class="hint ${it.cls || ''}">${it.after ? `<span class="small">${it.w}</span>${caps(...it.k)}` : `${caps(...it.k)}<span class="small">${it.w}</span>`}</span>`);
  document.body.insertAdjacentHTML('beforeend', `<div class="hints glass pill">${parts.join('')}</div>`);
}
function tag(t) { document.body.insertAdjacentHTML('beforeend', `<div class="mocktag">mock · ${t}</div>`); }
