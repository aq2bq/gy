/* The node page (n-5ca6): the show view of one node, its connections, and its
   own writes. The facts come from the typed data in the answer. */
(function () {
  const SIDE = { 'spawned-by': 'L', 'spawns': 'R', 'depends-on': 'L', 'depended-on-by': 'R', 'targets': 'R', 'targeted-by': 'L', 'closes': 'R', 'closed-by': 'L', 'narrows': 'L', 'narrowed-by': 'R', 'supersedes': 'L', 'superseded-by': 'R', 'widens': 'L', 'widened-by': 'R', 'completes': 'L', 'completed-by': 'R', 'relies-on': 'L', 'relied-on-by': 'R', 'filed-as': 'R', 'filed-from': 'L', 'raised': 'R', 'raised-by': 'L', 'waits-on': 'R', 'waited-on-by': 'L', 'awaited-by': 'L', 'files': 'L' };
  const COLOUR = { Need: 'need', Question: 'question', Decision: 'decision', Requirement: 'requirement', Criterion: 'criterion' };

  let node = null;

  const t = key => window.GyShell.t(key);
  const lang = () => window.GyShell.lang();
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));
  const card = (heading, body) => `<div class="card"><h3>${heading}</h3>${body}</div>`;
  const rel = edge => t(`rel.${edge.name}`);

  /* The state word the meta and the cards print, from the typed data. */
  function state() {
    const data = node.data;
    switch (node.kind) {
      case 'Need':
        return data.Need.closed ? 'closed' : 'open';
      case 'Question':
        return data.Question.closure ? 'closed' : 'open';
      case 'Criterion':
        return data.Criterion.satisfied ? 'satisfied' : 'unsatisfied';
      case 'Requirement':
        return data.Requirement.state.toLowerCase();
      default:
        return '';
    }
  }

  const stateWord = () => t(`st.${node.kind}.${state()}`);

  async function draw(id) {
    const res = await window.GyShell.read(`/api/node/${encodeURIComponent(id)}`);
    if (!res.ok) {
      crumb(id, null);
      document.getElementById('main').innerHTML = `<div class="hero"><h1>${esc(id)} — ${t('notFound')}</h1></div>`;
      return;
    }
    node = await res.json();
    crumb(node.id, node);
    document.getElementById('main').innerHTML = `<div class="node"><div>${left()}</div><div>${right()}</div></div>`;
    window.scrollTo(0, 0);
  }

  /* The trail: now › this kind's list › the scope (the node gives the last two). */
  function crumb(id, found) {
    const kind = found ? `<a href="#/list/${found.kind}">${window.GyShell.plural(found.kind)}</a><span>›</span><span>${esc(found.scope)}</span>` : '';
    document.getElementById('crumb').innerHTML = `<a href="#/">${t('now')}</a><span>›</span>${kind || esc(id)}`;
  }

  function left() {
    return `${head()}${facts()}${body()}`;
  }

  function head() {
    const word = state() ? stateWord() : '';
    const meta = [
      `<span class="al">${esc((node.aliases || [])[0] || node.id)}</span>`,
      `<span>${esc(node.id)}</span>`,
      `<span>${esc(node.scope)}</span>`,
      `<span>${fill(t('createdOn'), { d: node.created })}</span>`,
      word ? `<span>${esc(word)}</span>` : '',
    ];
    return `<div class="nh"><span class="k k-${node.kind}">${t(node.kind)}</span><h1>${esc(node.title)}</h1><div class="meta">${meta.join('')}</div></div>`;
  }

  function facts() {
    const data = node.data;
    if (node.kind === 'Decision') {
      const text = data.Decision.scope.text || t('unrecorded');
      return card(t('scopeOf'), `<div class="scope">${esc(text)}</div>`);
    }
    if (node.kind === 'Question') {
      const question = data.Question;
      const who = [question.decider, stateWord()].filter(Boolean).join(' · ');
      const options = question.options.map((option, index) => `<li><span class="tag">${String.fromCharCode(65 + index)}</span>${esc(option)}</li>`).join('');
      const closed = question.closure
        ? card(t('closedHow'), `<div class="fact"><b>${esc(question.closure)}</b> ${esc(question.evidence || '')}</div>`)
        : '';
      return card(who, `<ul class="opts">${options}</ul>`) + closed;
    }
    if (node.kind === 'Criterion') {
      const criterion = data.Criterion;
      const heading = criterion.satisfied ? t('met') : t('unmetH');
      const text = criterion.satisfied ? criterion.evidence || t('measure') : t('measure');
      return card(heading, `<div class="scope c">${esc(text)}</div>`);
    }
    if (node.kind === 'Requirement') {
      return requirement(data.Requirement);
    }
    const closed = data.Need.closed;
    const text = closed ? `<b>${esc(closed.by)}</b> ${esc(closed.evidence)}` : t('keepGoing');
    return card(stateWord(), `<div class="fact">${text}</div>`);
  }

  function requirement(data) {
    const reference = data.reference
      ? `<a href="${esc(data.reference)}" target="_blank" style="color:var(--requirement);text-decoration:underline;text-underline-offset:3px">${esc(data.reference)}</a>`
      : '—';
    const step = (label, at) => `<div class="hi"><span class="when">${at ? when(at) : t('pendingOn')}</span><span class="who"></span><span class="src">${label}</span></div>`;
    const heard = data.approval && data.approval.heard_by ? ` · ${t('heardBy')} ${esc(data.approval.heard_by)}` : '';
    const path =
      step(t('filedOn'), node.created) +
      step(t('approvedOn') + heard, data.approval && data.approval.at) +
      (data.cancellation ? step(t('cancelledOn'), data.cancellation.at) : step(t('doneOn'), data.completion && data.completion.at));
    const revisions = (data.revisions || []).length
      ? `<div class="fact" style="margin-top:8px">${fill(t('revisions'), { n: data.revisions.length })}</div>`
      : '';
    return card(t('reqRef'), `<div class="fact">${reference}</div>`) + card(t('reqPath'), `<div class="hist">${path}</div>${revisions}`);
  }

  function body() {
    const text = (node.body || '').trim() || t('noBody');
    return `<details class="body"><summary>${t('readBody')}</summary><pre>${esc(text)}</pre></details>`;
  }

  function right() {
    const rows = node.history || [];
    const history = rows.length
      ? rows.map(entry => `<div class="hi"><span class="when">${when(entry.at)} · ${entry.seq}</span><span class="who">${esc(entry.actor)}</span><span><div class="src">${esc(entry.source)}</div><div class="why">${esc(entry.why)}</div></span></div>`).join('')
      : `<div class="empty">${t('migrated')}</div>`;
    return `<div class="card" style="margin-top:0"><h3>${t('connections')}</h3><div class="map">${map()}</div></div>` +
      card(fill(t('hist'), { n: rows.length }), `<div class="hist">${history}</div>`);
  }

  /* The ego map: what points here on the left, what this points at on the
     right, from the edges the answer names (the prototype's egoSvg). */
  function map() {
    const left = [];
    const right = [];
    (node.edges || []).forEach(edge => {
      (SIDE[edge.name] === 'L' ? left : right).push(edge);
    });
    const rows = Math.max(left.length, right.length, 1);
    const rh = 72, width = 640, height = Math.max(200, rows * rh + 48);
    const cx = width / 2, cy = height / 2, bw = 196, bh = 48;
    const short = text => (text.length > 14 ? `${text.slice(0, 14)}…` : text);
    const box = (edge, x, y) => `<a href="#/n/${edge.to}"><g transform="translate(${x - bw / 2},${y - bh / 2})"><rect class="box" width="${bw}" height="${bh}" rx="7" stroke="var(--${COLOUR[edge.kind] || 'paper'})"/><text class="al" x="10" y="17">${esc(edge.alias || edge.to)}${edge.kind ? ` · ${t(edge.kind)}` : ''}</text><text x="10" y="36">${esc(short(edge.title || edge.to))}</text></g></a>`;
    const line = (x1, y1, x2, y2, label, side) => {
      const mx = (x1 + x2) / 2;
      const lx = side === 'L' ? (x1 + cx - bw / 2) / 2 : (cx + bw / 2 + x2) / 2;
      return `<path class="edge" d="M${x1},${y1} C${mx},${y1} ${mx},${y2} ${x2},${y2}"/><text class="rel" x="${lx}" y="${(y1 + y2) / 2 - 6}" text-anchor="middle">${esc(label)}</text>`;
    };
    let svg = '';
    left.forEach((edge, index) => {
      const y = cy + (index - (left.length - 1) / 2) * rh;
      svg += line(bw / 2 + 16, y, cx - bw / 2, cy, rel(edge), 'L') + box(edge, bw / 2 + 16, y);
    });
    right.forEach((edge, index) => {
      const y = cy + (index - (right.length - 1) / 2) * rh;
      svg += line(cx + bw / 2, cy, width - bw / 2 - 16, y, rel(edge), 'R') + box(edge, width - bw / 2 - 16, y);
    });
    svg += `<g transform="translate(${cx - bw / 2},${cy - bh / 2})"><rect class="box center" width="${bw}" height="${bh}" rx="7" stroke="var(--${COLOUR[node.kind]})"/><text class="al" x="10" y="17">${esc((node.aliases || [])[0] || node.id)} · ${t(node.kind)}</text><text x="10" y="36">${esc(short(node.title))}</text></g>`;
    if (!left.length && !right.length) {
      svg += `<text class="rel" x="${cx}" y="${height - 14}" text-anchor="middle">${t('noConn')}</text>`;
    }
    return `<svg viewBox="0 0 ${width} ${height}">${svg}</svg>`;
  }

  function when(at) {
    if (!at) return '';
    const date = String(at).includes('T') ? new Date(at) : new Date(at * 1000);
    if (Number.isNaN(date.getTime())) return esc(at);
    return date.toLocaleString(lang() === 'ja' ? 'ja-JP' : 'en-GB', {
      month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit',
    });
  }

  window.GyNode = { draw };
})();
