/* The time band region (d-03ca; was time.js): the log's ticks, a head, and the
   two labels. The ticks are drawn on one canvas, not as thousands of elements:
   a long ledger has one write per pixel and the band must not make the screen
   slow (r-2d02). It draws inside `.scrub`; the drag is events.js's. */
(function () {
  const TALL = ['need', 'question', 'criterion', 'req'];
  const COLOUR = { decide: '#5fd3c7', need: '#e9a63a', question: '#b79cff', criterion: '#8fd18f' };
  const height = kind => (kind === 'decide' ? 26 : TALL.includes(kind) ? 18 : 10);
  const BASELINE = 38;
  const PAD = 4;

  const dayOf = at => {
    const date = new Date(at * 1000);
    return `${date.getMonth() + 1}/${date.getDate()}`;
  };

  function render(state, el, ui) {
    if (!el || !state.band) return;
    const track = el.querySelector('#track');
    const canvas = el.querySelector('#ticks');
    const head = el.querySelector('#head');
    const shade = el.querySelector('#shade');
    const now = el.querySelector('#scrub-now');
    const band = state.band;
    const width = Math.max(1, track.clientWidth);
    const seqAt = state.at === null ? band.max : state.at;
    /* The point in view, for the eye and for what reads the screen (n-5e43). */
    el.dataset.at = state.at === null ? 'now' : String(state.at);
    el.dataset.ticks = String(band.ticks.length);
    track.setAttribute('role', 'slider');
    track.setAttribute('aria-label', ui.t('rewind'));
    track.setAttribute('aria-valuemin', String(band.min));
    track.setAttribute('aria-valuemax', String(band.max));
    track.setAttribute('aria-valuenow', String(seqAt));
    const x = seq => ((seq - band.min) / Math.max(1, band.max - band.min)) * width;

    draw(canvas, band, width, x, ui);
    if (canvas.dataset.unpushed) {
      el.dataset.unpushed = canvas.dataset.unpushed;
      el.title = `${window.GySync.pending} ${ui.t('notPushed')}`;
    } else {
      delete el.dataset.unpushed;
      el.removeAttribute('title');
    }
    head.style.left = `${x(seqAt)}px`;
    shade.style.width = `${Math.max(0, width - x(seqAt))}px`;

    el.querySelector('#t-rewind').textContent = ui.t('rewind');
    now.textContent = ui.t('nowBtn');
    now.classList.toggle('off', state.at === null);
    const tick = ui.tickAt(seqAt);
    el.querySelector('#scrub-at').textContent = state.at === null ? '' : tick ? ui.when(tick.at) : ui.t('start');
    el.querySelector('#scrub-seq').textContent = `seq ${seqAt} / ${band.max}`;
  }

  /* The whole band on one canvas: the baseline, the unpushed span, one bar per
     write, and a day label only where it has room to read. */
  function draw(canvas, band, width, x, ui) {
    const ratio = window.devicePixelRatio || 1;
    const tall = 56;
    canvas.width = Math.round(width * ratio);
    canvas.height = Math.round(tall * ratio);
    canvas.style.width = `${width}px`;
    canvas.style.height = `${tall}px`;
    const ctx = canvas.getContext('2d');
    ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
    ctx.clearRect(0, 0, width, tall);

    const sync = window.GySync;
    if (sync && sync.pending && (sync.last_ok_seq ?? 0) < band.max) {
      const reached = sync.last_ok_seq ?? 0;
      const left = x(Math.max(band.min, reached));
      ctx.fillStyle = 'rgba(126,122,112,.22)';
      ctx.fillRect(left, 6, Math.max(1, x(band.max) - left), 32);
      canvas.dataset.unpushed = String(reached);
    } else {
      delete canvas.dataset.unpushed;
    }
    ctx.strokeStyle = '#2a3450';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, BASELINE + 0.5);
    ctx.lineTo(width, BASELINE + 0.5);
    ctx.stroke();

    let lastLabelX = -Infinity;
    let seenDay = null;
    for (const tick of band.ticks) {
      const at = x(tick.seq);
      const bar = height(tick.kind);
      ctx.fillStyle = COLOUR[tick.kind] || '#7e7a70';
      ctx.fillRect(at, BASELINE - bar, 1.4, bar);
      const day = dayOf(tick.at);
      if (day === seenDay) continue;
      seenDay = day;
      if (at - lastLabelX < 58) continue;
      lastLabelX = at;
      ctx.fillStyle = '#7e7a70';
      ctx.font = '10px "SF Mono", Menlo, monospace';
      ctx.fillText(day, at + PAD, tall - 4);
    }
    if (band.ticks.length && width - lastLabelX > 58) {
      const last = band.ticks[band.ticks.length - 1];
      ctx.fillStyle = '#7e7a70';
      ctx.font = '10px "SF Mono", Menlo, monospace';
      ctx.fillText(dayOf(last.at), width - 30, tall - 4);
    }
  }

  /* The sequence under a pointer x, for the root's drag (it may not read the
     band's geometry itself). */
  function seqFrom(state, el, clientX) {
    const box = el.querySelector('#track').getBoundingClientRect();
    const band = state.band;
    return band.min + ((clientX - box.left) / box.width) * (band.max - band.min);
  }

  window.GyBand = { render, seqFrom };
})();
