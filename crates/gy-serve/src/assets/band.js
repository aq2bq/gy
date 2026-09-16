/* The time band region (d-03ca, 第 2 段; was time.js): one tick per write, a
   head, and the two labels. It draws inside `.scrub` and never touches #clock,
   which is the sidebar's. */
(function () {
  const TALL = ['need', 'question', 'criterion', 'req'];
  const COLOUR = { decide: 'var(--decision)', need: 'var(--need)', question: 'var(--question)', criterion: 'var(--criterion)' };
  const height = kind => (kind === 'decide' ? 26 : TALL.includes(kind) ? 18 : 10);

  function render(state, el, ui) {
    if (!el || !state.band) return;
    const track = el.querySelector('#track');
    const scale = el.querySelector('#ticks');
    const head = el.querySelector('#head');
    const shade = el.querySelector('#shade');
    const now = el.querySelector('#scrub-now');
    const band = state.band;
    const width = track.clientWidth;
    const seqAt = state.at === null ? band.max : state.at;
    const x = seq => ((seq - band.min) / Math.max(1, band.max - band.min)) * width;

    scale.setAttribute('viewBox', `0 0 ${width} 46`);
    scale.innerHTML = ticks(band, x, ui);
    head.style.left = `${x(seqAt)}px`;
    shade.style.width = `${Math.max(0, width - x(seqAt))}px`;

    el.querySelector('#t-rewind').textContent = ui.t('rewind');
    now.textContent = ui.t('nowBtn');
    now.classList.toggle('off', state.at === null);
    const tick = ui.tickAt(seqAt);
    el.querySelector('#scrub-at').textContent = state.at === null ? '' : tick ? ui.when(tick.at) : ui.t('start');
    el.querySelector('#scrub-seq').textContent = `seq ${seqAt} / ${band.max}`;
  }

  /* The baseline, one day label per day, then a bar per write. */
  function ticks(band, x, ui) {
    let svg = `<line x1="0" y1="38" x2="100%" y2="38" stroke="var(--line)"/>`;
    const days = new Set();
    for (const tick of band.ticks) {
      const label = ui.day(tick.at);
      if (days.has(label)) continue;
      days.add(label);
      svg += `<text x="${x(tick.seq) + 3}" y="48" font-family="var(--mono)" font-size="10" fill="var(--paper-3)">${label}</text>`;
    }
    for (const tick of band.ticks) {
      const bar = height(tick.kind);
      svg += `<rect x="${x(tick.seq) - 0.8}" y="${38 - bar}" width="1.6" height="${bar}" fill="${COLOUR[tick.kind] || 'var(--paper-3)'}" opacity=".9"/>`;
    }
    return svg;
  }

  window.GyBand = { render };
})();