// Builds the cube twice: the cube and its mirror image in the floor.
const S = 'faces/';
function cube(yaw, faces, lit) {
  const one = faces.map((f, i) => `<div class="face f${i}${lit === i ? ' lit' : ''}" style="${f.startsWith('#') || f.startsWith('linear') ? `background:${f}` : `background-image:url(${S}${f})`}"></div>`).join('');
  const c = `<div class="stage"><div class="cube" style="transform: rotateY(${yaw}deg)">${one}</div></div>`;
  document.body.insertAdjacentHTML('afterbegin', `<div class="mirror">${c.replace(' lit', '')}</div>` + c);
}
function facelabel(x, y, name, words, dots) {
  const d = dots.map(k => `<i class="${k}" ${k.startsWith('p') ? `style="background:#2680B4;opacity:${k.slice(1)}"` : ''}></i>`).join('');
  document.body.insertAdjacentHTML('beforeend', `<div class="facelabel glass pill" style="left:${x}px;top:${y}px"><span class="name">${name}</span><span class="words">${words}</span><span class="dots">${d}</span></div>`);
}
