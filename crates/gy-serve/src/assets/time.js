/* The time band (n-32a9): one tick per write, a draggable head, and ←→ keys.
   It fetches /api/ticks once and drives GyShell.at, so every page answers for
   the chosen point. */
(function () {
  const TRACK = document.getElementById('track');
  const SCALE = document.getElementById('ticks');
  const HEAD = document.getElementById('head');
  const SHADE = document.getElementById('shade');
  const REWIND = document.getElementById('t-rewind');
  const AT = document.getElementById('scrub-at');
  const SEQ = document.getElementById('scrub-seq');
  const NOW = document.getElementById('scrub-now');

  const TALL = ['need', 'question', 'criterion', 'req'];
  const COLOUR = {
    decide: 'var(--decision)',
    need: 'var(--need)',
    question: 'var(--question)',
    criterion: 'var(--criterion)',
  };

  let ticks = [];
  let max = 0;
  let min = 1;
  let dragging = false;
  let timer = null;

  const t = key => window.GyShell.t(key);
  const lang = () => window.GyShell.lang();
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));
  const height = kind => (kind === 'decide' ? 26 : TALL.includes(kind) ? 18 : 10);
  const x = seq => ((seq - min) / Math.max(1, max - min)) * TRACK.clientWidth;

  /* The tick at or before a sequence, so a point between writes has a time. */
  function tickAt(seq) {
    let found = null;
    for (const tick of ticks) {
      if (tick.seq > seq) break;
      found = tick;
    }
    return found;
  }

  function when(at) {
    return new Date(at * 1000).toLocaleString(lang() === 'ja' ? 'ja-JP' : 'en-GB', {
      month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit',
    });
  }

  function day(at) {
    return new Date(at * 1000).toLocaleDateString(lang() === 'ja' ? 'ja-JP' : 'en-GB');
  }

  function draw() {
    const width = TRACK.clientWidth;
    const tall = 46;
    SCALE.setAttribute('viewBox', `0 0 ${width} ${tall}`);
    let svg = `<line x1="0" y1="${tall - 8}" x2="${width}" y2="${tall - 8}" stroke="var(--line)"/>`;
    const days = new Set();
    for (const tick of ticks) {
      const label = day(tick.at);
      if (days.has(label)) continue;
      days.add(label);
      svg += `<text x="${x(tick.seq) + 3}" y="${tall + 2}" font-family="var(--mono)" font-size="10" fill="var(--paper-3)">${label}</text>`;
    }
    for (const tick of ticks) {
      const bar = height(tick.kind);
      svg += `<rect x="${x(tick.seq) - 0.8}" y="${tall - 8 - bar}" width="1.6" height="${bar}" fill="${COLOUR[tick.kind] || 'var(--paper-3)'}" opacity=".9"/>`;
    }
    SCALE.innerHTML = svg;
    place();
  }

  /* The head line and the shade, at the chosen sequence. */
  function place() {
    const seq = window.GyShell.at ?? max;
    const left = x(seq);
    HEAD.style.left = `${left}px`;
    SHADE.style.width = `${Math.max(0, TRACK.clientWidth - left)}px`;
  }

  /* The two bar labels and the clock; also called when the language flips. */
  function labels() {
    REWIND.textContent = t('rewind');
    NOW.textContent = t('nowBtn');
    show();
  }

  function show() {
    const seq = window.GyShell.at;
    const tick = tickAt(seq ?? max);
    const time = tick ? when(tick.at) : t('start');
    AT.textContent = seq === null ? '' : time;
    SEQ.textContent = `seq ${seq ?? max} / ${max}`;
    NOW.classList.toggle('off', seq === null);
    if (seq === null) return;
    document.getElementById('clock').innerHTML = fill(t('viewing'), { t: time });
  }

  /* A fetch and a redraw at most once every 60 ms while the head moves. */
  function reload() {
    if (timer) return;
    timer = setTimeout(() => {
      timer = null;
      window.GyShell.reload();
    }, 60);
  }

  function setAt(seq) {
    const head = window.GyShell.at ?? max;
    const next = Math.max(min, Math.min(max, Math.round(seq)));
    window.GyShell.at = next >= max ? null : next;
    show();
    place();
    reload();
    return head;
  }

  const seqFrom = event => {
    const box = TRACK.getBoundingClientRect();
    return min + ((event.clientX - box.left) / box.width) * (max - min);
  };

  TRACK.addEventListener('pointerdown', event => {
    dragging = true;
    TRACK.setPointerCapture(event.pointerId);
    setAt(seqFrom(event));
  });
  TRACK.addEventListener('pointermove', event => {
    if (dragging) setAt(seqFrom(event));
  });
  TRACK.addEventListener('pointerup', () => {
    dragging = false;
  });
  NOW.addEventListener('click', () => setAt(max));
  document.addEventListener('keydown', event => {
    if (window.GyShell.composing(event)) return;
    if (event.target.tagName === 'INPUT') return;
    const head = window.GyShell.at ?? max;
    if (event.key === 'ArrowLeft') setAt(head - 1);
    if (event.key === 'ArrowRight') setAt(head + 1);
  });
  window.addEventListener('resize', () => {
    draw();
  });

  async function load() {
    const res = await fetch('/api/ticks');
    const body = await res.json();
    ticks = body.ticks || [];
    max = body.max || 0;
    min = ticks.length ? ticks[0].seq : 1;
  }

  /* Take the ticks again and redraw (n-478f). The head follows the end when it
     was there; while a past point is chosen `at` does not move, so only the
     right end of the band grows. */
  async function refresh() {
    await load();
    labels();
    draw();
  }

  window.GyTime = { labels, refresh };
  (async () => {
    await load();
    labels();
    draw();
  })();
})();
