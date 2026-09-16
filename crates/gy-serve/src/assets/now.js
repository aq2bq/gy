/* The now page (d-3e8f): three eyes side by side — who waits, what is ready,
   and how to resume — then the sky of the nodes that matter and the pulse. It
   reads /api/now and /api/graph; the words come from GyShell. */
(function () {
  const VERBS = [
    ['need add', 'needadd'],
    ['need close', 'needclose'],
    ['question add', 'qadd'],
    ['question close', 'qclose'],
    ['criterion satisfy', 'acsat'],
    ['criterion add', 'acadd'],
    ['decide', 'decide'],
    ['edit', 'edit'],
    ['req ', 'req'],
  ];

  let data = null;
  let actors = [];

  const t = key => window.GyShell.t(key);
  const lang = () => window.GyShell.lang();
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));

  /* The label a row shows: the first alias, else the id. */
  const label = row => row.alias || row.id;
  const status = row => t(`st.${row.kind}.${row.status}`);
  const when = at =>
    new Date(at * 1000).toLocaleTimeString(lang() === 'ja' ? 'ja-JP' : 'en-GB', {
      hour: '2-digit', minute: '2-digit',
    });

  const head = (cls, n, name, sub) =>
    `<div class="head"><span class="n">${n}</span><span class="l">${esc(name)}</span><span class="h">${esc(sub)}</span></div>`;

  const rowHtml = (row, st) =>
    `<a class="row" href="#/n/${row.id}"><span class="dot dot-${row.kind}"></span><span class="id">${esc(label(row))}</span><span class="t" title="${esc(row.title)}">${esc(row.title)}</span><span class="st">${st}</span></a>`;

  const questions = () => data.waiting.filter(item => item.waiting === 'question');
  const requirements = () => data.waiting.filter(item => item.waiting === 'requirement');

  /* One question as a card, as the old page drew it. */
  function card(item) {
    const row = item.row;
    const options = item.options.map(option => `<li>${esc(option)}</li>`).join('');
    return `<a class="wait" href="#/n/${row.id}"><div class="who">${esc(label(row))} · ${t('decider')} ${esc(item.decider)}</div><div class="t">${esc(row.title)}</div><ol class="opts">${options}</ol></a>`;
  }

  /* The first eye: the questions that name the master, then the filed
     requirements. An empty column says so in one large line. */
  function waitColumn() {
    const qs = questions();
    const rs = requirements();
    const sub = `${t('Questions')} ${qs.length} · ${t('Requirements')} ${rs.length}`;
    const eye = head('hot', data.waiting.length, t('eyeWait'), sub);
    if (!data.waiting.length) {
      return `<section class="eye hot" data-eye="wait">${eye}<div class="nothing">${t('nothing')}</div></section>`;
    }
    const cards = qs.slice(0, 2).map(card).join('');
    const more = qs.length > 2
      ? `<a class="more" href="#/list/Question">${fill(t('moreQuestions'), { n: qs.length - 2 })}</a>`
      : '';
    const rows = rs.length
      ? `<div class="rows">${rs.map(item => rowHtml(item.row, status(item.row))).join('')}</div>`
      : '';
    return `<section class="eye hot" data-eye="wait">${eye}${cards}${more}${rows}</section>`;
  }

  /* The second eye: the needs ready to work, with what is left to meet. */
  function readyColumn() {
    const rows = data.ready
      .map(item => rowHtml(item.row, fill(t('remaining'), { n: item.targets - item.satisfied, m: item.targets })))
      .join('');
    const tail = `<a class="more" href="#/list/Need">${fill(t('allNeeds'), { n: data.in_progress.length })}</a>`;
    return `<section class="eye next" data-eye="next">${head('next', data.ready.length, t('readyHead'), t('readySub'))}<div class="rows">${rows}</div>${tail}</section>`;
  }

  /* The third eye: the in-progress requirements, then the counts. The outward
     reference is a span, not a link: the row itself is a link to the node page
     (a nested anchor would break the grid). */
  function resumeColumn() {
    const rows = data.resume.in_progress.map(item => {
      const tail = referenceTail(item.reference);
      const ref = tail ? ` · <span class="ref">${esc(tail)}</span>` : '';
      return rowHtml({ id: item.id, kind: 'Requirement', title: item.title, alias: null }, esc(t(`st.Requirement.${item.state}`)) + ref);
    }).join('');
    const last = data.resume.last;
    const lastText = last ? `${when(last.at)} ${esc(last.actor)} · seq ${last.seq}` : t('none');
    const kv = `<div class="kv"><span>${t('openQuestions')}</span><b>${data.resume.open_questions}</b><span>${t('warnings')}</span><b>${data.resume.warnings || t('none')}</b><span>${t('lastWrite')}</span><b>${lastText}</b></div>`;
    return `<section class="eye prog" data-eye="resume">${head('prog', data.resume.in_progress.length, t('resumeHead'), t('resumeSub'))}<div class="rows">${rows}</div>${kv}</section>`;
  }

  /* The number a requirement's outward reference ends with, as a link's word. */
  function referenceTail(reference) {
    if (!reference) return '';
    return String(reference).split('/').filter(Boolean).pop() || '';
  }

  const legend = () =>
    `<span><i style="background:var(--requirement)"></i>${t('skyLegendWait')}</span><span><i style="background:var(--need)"></i>${t('skyLegendNext')}</span><span><i style="background:var(--decision)"></i>${t('skyLegendResume')}</span>`;

  /* The three sets the sky lights, by column colour. */
  function groups() {
    return {
      '#f28ba0': new Set(data.waiting.map(item => item.row.id)),
      '#e9a63a': new Set(data.ready.map(item => item.row.id)),
      '#5fd3c7': new Set(data.resume.in_progress.map(item => item.id)),
    };
  }

  /* The sky: /api/graph shrunk, the nodes that matter lit, a click opening the
     nearest lit node in the graph page (d-3e8f). The canvas is measured after
     a frame, so its laid-out size is the one the fit uses. */
  async function drawSky() {
    const canvas = document.getElementById('sky');
    if (!canvas) return;
    await new Promise(resolve => requestAnimationFrame(resolve));
    const graph = await (await window.GyShell.read('/api/graph')).json();
    const rect = canvas.getBoundingClientRect();
    const w = Math.round(rect.width) || canvas.clientWidth;
    const h = Math.round(rect.height) || canvas.clientHeight;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = w * dpr; canvas.height = h * dpr;
    const ctx = canvas.getContext('2d');
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);
    const sets = groups();
    const colour = id => Object.keys(sets).find(key => sets[key].has(id)) || null;
    const lit = graph.nodes.filter(node => colour(node.id));
    const fit = lit.length ? lit : graph.nodes;
    let minx = 1e9, maxx = -1e9, miny = 1e9, maxy = -1e9;
    for (const node of fit) {
      minx = Math.min(minx, node.x); maxx = Math.max(maxx, node.x);
      miny = Math.min(miny, node.y); maxy = Math.max(maxy, node.y);
    }
    const pad = 40;
    const k = Math.min((w - pad * 2) / Math.max(1, maxx - minx), (h - pad * 2) / Math.max(1, maxy - miny));
    const ox = w / 2 - ((minx + maxx) / 2) * k, oy = h / 2 - ((miny + maxy) / 2) * k;
    const P = new Map(graph.nodes.map(node => [node.id, [node.x * k + ox, node.y * k + oy]]));
    ctx.strokeStyle = 'rgba(185,179,166,.10)'; ctx.lineWidth = 0.6;
    for (const [a, b] of graph.edges) {
      const A = P.get(a), B = P.get(b);
      if (!A || !B) continue;
      ctx.beginPath(); ctx.moveTo(A[0], A[1]); ctx.lineTo(B[0], B[1]); ctx.stroke();
    }
    const ring = { '#f28ba0': 'rgba(242,139,160,.20)', '#e9a63a': 'rgba(233,166,58,.20)', '#5fd3c7': 'rgba(95,211,199,.20)' };
    const points = {};
    for (const node of graph.nodes) {
      const [px, py] = P.get(node.id);
      const col = colour(node.id);
      if (col) {
        ctx.beginPath(); ctx.arc(px, py, 7, 0, Math.PI * 2); ctx.fillStyle = ring[col]; ctx.fill();
        ctx.beginPath(); ctx.arc(px, py, 3, 0, Math.PI * 2); ctx.fillStyle = col; ctx.fill();
        points[node.id] = [px, py];
      } else {
        ctx.beginPath(); ctx.arc(px, py, 1.4, 0, Math.PI * 2); ctx.fillStyle = 'rgba(185,179,166,.28)'; ctx.fill();
      }
    }
    canvas.onclick = event => {
      const rect = canvas.getBoundingClientRect();
      const px = event.clientX - rect.left, py = event.clientY - rect.top;
      let best = null, close = 400;
      for (const [id, point] of Object.entries(points)) {
        const distance = Math.hypot(point[0] - px, point[1] - py);
        if (distance < close) { close = distance; best = id; }
      }
      if (best) location.hash = `#/graph/${best}`;
    };
  }

  /* The write's verb, from what the writer asked for (the why). */
  function verb(entry) {
    for (const [prefix, key] of VERBS) {
      if (entry.why.startsWith(prefix)) return t(`why.${key}`);
    }
    if (entry.why.startsWith('link')) {
      return `${t('why.link')} · ${entry.why.replace(/^link \S+ /, '')}`;
    }
    return entry.why;
  }

  /* Who did what to which node, in this language. */
  function what(entry) {
    const actor = esc(entry.actor);
    const doing = esc(verb(entry));
    const node = entry.node ? `<a href="#/n/${entry.node}">${esc(entry.node)}</a>` : '';
    if (!node) return `${actor} ${doing}`;
    return lang() === 'ja' ? `${actor} が ${node} ${doing}` : `${actor} ${doing} ${node}`;
  }

  function line(entry) {
    return `<div class="when">${when(entry.at)} · seq ${entry.seq} · ${esc(entry.actor)}</div><div class="what">${what(entry)}</div><div class="src" title="${esc(entry.source)}">${esc(entry.source)}</div>`;
  }

  /* The dot colour: the four actor colours in the order the writers appear. */
  function actorCls(actor) {
    let index = actors.indexOf(actor);
    if (index < 0) {
      actors.push(actor);
      index = actors.length - 1;
    }
    return `a-${Math.min(3, index)}`;
  }

  /* The pulse, in two columns. */
  function pulse() {
    const rows = data.recent
      .map((entry, index) => `<div class="pi ${actorCls(entry.actor)}${index === 0 ? ' new' : ''}">${line(entry)}</div>`)
      .join('');
    const upTo = data.recent.length ? when(data.recent[0].at) : '';
    return `<section class="pulse2"><h2>${t('pulse')}</h2><div class="sub">${fill(t('pulseSub'), { n: data.recent.length, t: upTo })}</div><div class="pl2">${rows}</div></section>`;
  }

  async function draw() {
    const scope = window.GyShell.scope();
    const only = scope && scope !== 'all' ? `?scope=${encodeURIComponent(scope)}` : '';
    data = await (await window.GyShell.read(`/api/now${only}`)).json();
    actors = [];
    document.getElementById('main').innerHTML =
      `<div class="eyes">${waitColumn()}${readyColumn()}${resumeColumn()}</div>` +
      `<div class="sky"><canvas id="sky"></canvas><div class="cap">${t('skyCap')}</div><div class="leg">${legend()}</div></div>` +
      pulse();
    window.GyShell.setEyes(data.waiting.length, data.ready.length, data.resume.in_progress.length);
    await drawSky();
  }

  window.GyNow = { draw };
})();