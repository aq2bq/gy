/* The node page (n-5ca6, a region since d-03ca 第 3 段): the show view of one
   node, its connections, and its own writes. It draws inside #main from the
   state the root read; the facts come from the typed data in the answer. */
(function () {
  const COLOUR = { Need: 'need', Question: 'question', Decision: 'decision', Requirement: 'requirement', Criterion: 'criterion' };

  let node = null;
  let t = key => key;
  let lang = 'en';
  let tag = () => '';
  let copy = () => '';
  /* The map's camera, the state's one, kept here for the draw (n-9ca9). */
  let cam = { x: 0, y: 0, k: 1 };
  /* The nodes this tab has opened, for the "came from" mark (n-...). */
  let trail = [];

  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));
  const card = (heading, body) => `<div class="card"><h3>${heading}</h3>${body}</div>`;

  /* The state word the meta and the cards print, from the typed data. */
  function condition() {
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

  const stateWord = () => t(`st.${node.kind}.${condition()}`);

  /* The region: #main only, from the state the root read. A node that is not
     there (a null answer) shows its id and says so. */
  function render(state, el, ui) {
    if (!el) return;
    /* The node the element still shows: the mark is read before the draw
       replaces it, so a redraw of one node keeps the reader's place and a
       fresh page or another node opens at the top (n-9ca9). */
    const shown = el.querySelector('[data-shown]');
    t = ui.t;
    lang = state.lang;
    tag = ui.scopeTag;
    copy = text => window.GyCopy.tag(text, state, ui);
    node = state.page;
    cam = (state.ego && state.ego.cam) || { x: 0, y: 0, k: 1 };
    trail = state.trail || [];
    if (!node) {
      el.innerHTML = `<div class="hero"><h1>${esc(state.route.arg)} — ${t('notFound')}</h1></div>`;
      return;
    }
    const moved = !shown || shown.dataset.shown !== node.id;
    el.innerHTML = `<div class="node" data-shown="${esc(node.id)}"><div>${left()}</div><div>${right()}</div></div>`;
    if (moved) window.scrollTo(0, 0);
  }

  function left() {
    return `${head()}${retraction()}${facts()}${body()}`;
  }

  /* The retractions the view judged (n-0c6f): the decisions that narrowed a
     passage of this one, or replaced it whole. The marks themselves are already
     in the body and the scope the answer carries; this only names who did it. */
  function retraction() {
    const cancelled = node.cancellation;
    if (!cancelled) return '';
    const rows = [
      ...(cancelled.superseded_by || []).map(id => `${t('rel.superseded-by')} <b>${esc(id)}</b>`),
      ...(cancelled.narrowed || []).map(narrow => `${t('rel.narrowed-by')} <b>${esc(narrow.by)}</b> (${esc(narrow.mark)})`),
    ];
    if (!rows.length) return '';
    return `<div class="fact" style="margin-top:22px;border-left:3px solid var(--need);padding:2px 0 2px 14px">${rows.map(row => `<div>${row}</div>`).join('')}</div>`;
  }

  function head() {
    const word = condition() ? stateWord() : '';
    const meta = [
      `<span class="al">${esc((node.aliases || [])[0] || node.id)}</span>`,
      copy(node.id),
      `<span>${tag(node.scope)}</span>`,
      `<span data-testid="created">${fill(t('createdOn'), { d: window.GyTime.date(node.created) })}</span>`,
      word ? `<span>${esc(word)}</span>` : '',
    ];
    return `<div class="nh"><span class="k k-${node.kind}">${t(node.kind)}</span><h1>${esc(node.title)}</h1><div class="meta">${meta.join('')}</div></div>`;
  }

  function facts() {
    const data = node.data;
    if (node.kind === 'Decision') {
      const text = node.decision_scope_marked || data.Decision.scope.text || t('unrecorded');
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
      /* The heading names what the card holds: the evidence when there is one,
         the way it will be measured when there is not (n-cb74). */
      const evidence = criterion.evidence || '';
      return card(evidence ? t('metEvidence') : t('howMeasured'), `<div class="scope c">${esc(evidence || t('measure'))}</div>`);
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
    const step = (label, at) => `<div class="hi"><span class="when">${at ? window.GyTime.dateTime(at, lang) : t('pendingOn')}</span><span class="who"></span><span class="src">${label}</span></div>`;
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
    const text = (node.body_marked || node.body || '').trim() || t('noBody');
    return `<div class="body"><div class="bt">${t('bodyTitle')}</div><pre>${esc(text)}</pre></div>`;
  }

  function right() {
    const rows = node.history || [];
    const history = rows.length
      ? rows.map(entry => `<div class="hi" role="listitem"><span class="when">${window.GyTime.dateTime(entry.at, lang)} · ${entry.seq}</span><span class="who">${esc(entry.actor)}</span><span><div class="src">${esc(entry.source)}</div><div class="why">${esc(entry.why)}</div></span></div>`).join('')
      : `<div class="empty">${t('migrated')}</div>`;
    const from = referrer();
    const heading = `${t('connections')}${from ? `<a href="#/n/${esc(from.id)}">◂ ${esc(from.name)}</a>` : ''}`;
    return `<div class="card" style="margin-top:0"><h3>${heading}</h3><div class="map">${map()}</div></div>` +
      card(fill(t('hist'), { n: rows.length }), `<div class="hist" role="list" aria-label="${esc(t('histTitle'))}">${history}</div>`);
  }

  /* The node this tab came from: the step before this one in the visit trail,
     so the card can name it and mark its box (n-...). */
  function referrer() {
    let at = trail.length - 1;
    while (at >= 0 && trail[at] === node.id) at--;
    if (at < 0) return null;
    const id = trail[at];
    const known = ((node.neighborhood || {}).nodes || []).find(item => item.id === id);
    return { id, name: (known && (known.alias || known.id)) || id };
  }

  /* The ego map (n-...): the focus at the centre, its direct peers on the inner
     ring, and their peers on the outer ring. The answer's neighbourhood fixes
     what is drawn; an edge among the drawn nodes is one line, and only the
     focus's own edges carry a relation name so the picture stays readable. */
  const BW = 116, BH = 40, R1 = 140, R2 = 236;
  const short = text => (text && text.length > 12 ? `${text.slice(0, 12)}…` : text || '');

  /* The two rings' positions: the focus at the centre, hop 1 and hop 2 around. */
  function ringPositions(hood) {
    const width = 660, height = 560, cx = width / 2, cy = height / 2;
    const at = new Map([[hood.root, { x: cx, y: cy }]]);
    const ring = (list, radius) => list.forEach((item, index) => {
      const angle = -Math.PI / 2 + (2 * Math.PI * index) / Math.max(list.length, 1);
      at.set(item.id, { x: cx + Math.cos(angle) * radius, y: cy + Math.sin(angle) * radius });
    });
    ring(hood.nodes.filter(item => item.hop === 1), R1);
    ring(hood.nodes.filter(item => item.hop === 2), R2);
    return { at, width, height };
  }

  function edgeLine(a, b, label) {
    if (!a || !b) return '';
    const mx = (a.x + b.x) / 2;
    const text = label ? `<text class="rel" x="${mx}" y="${(a.y + b.y) / 2 - 6}" text-anchor="middle">${esc(label)}</text>` : '';
    return `<path class="edge" d="M${a.x},${a.y} C${mx},${a.y} ${mx},${b.y} ${b.x},${b.y}"/>${text}`;
  }

  /* One box of the map: the focus is a plain mark, a peer opens its page. */
  function egoBox(item, p, root, from) {
    const cls = ['box', item.hop === 2 ? 'far' : '', item.id === root ? 'center' : '', item.id === from ? 'from' : ''].filter(Boolean).join(' ');
    const stroke = item.kind ? ` stroke="var(--${COLOUR[item.kind] || 'paper'})"` : '';
    const label = item.id === root
      ? `${esc(item.alias || item.id)} · ${esc(t(item.kind))}`
      : `${esc(item.alias || item.id)}${item.kind ? ` · ${esc(t(item.kind))}` : ''}`;
    const inner = `<rect class="${cls}" data-hop="${item.hop}"${item.id === from ? ' data-from="1"' : ''} x="${p.x - BW / 2}" y="${p.y - BH / 2}" width="${BW}" height="${BH}" rx="7"${stroke}/><text class="al" x="${p.x - BW / 2 + 8}" y="${p.y - 5}">${label}</text><text x="${p.x - BW / 2 + 8}" y="${p.y + 13}">${esc(short(item.title))}</text>`;
    return item.id === root ? `<g>${inner}</g>` : `<a href="#/n/${esc(item.id)}"><g>${inner}</g></a>`;
  }

  /* The camera drawn: one transform inside the svg. Events turns a pointer and a
     drag into the camera; the region only draws what it is handed (n-9ca9). */
  const camAttrs = () => `data-k="${cam.k}" data-x="${cam.x}" data-y="${cam.y}"`;
  const camOpen = () => `<g id="mapcam" transform="translate(${cam.x} ${cam.y}) scale(${cam.k})">`;

  function map() {
    const hood = node.neighborhood;
    if (!hood || !hood.nodes || hood.nodes.length <= 1) {
      const open = `<svg id="ego" data-testid="ego" viewBox="0 0 640 110" ${camAttrs()}>${camOpen()}`;
      return `${open}<text class="rel" x="320" y="60" text-anchor="middle">${t('noConn')}</text></g></svg>`;
    }
    const from = (referrer() || {}).id || null;
    const { at, width, height } = ringPositions(hood);
    const edges = (hood.edges || [])
      .map(edge => {
        const touches = edge.from === hood.root || edge.to === hood.root;
        const name = edge.from === hood.root ? edge.name : edge.inverse;
        return edgeLine(at.get(edge.from), at.get(edge.to), touches ? t(`rel.${name}`) : '');
      })
      .join('');
    const boxes = hood.nodes.map(item => egoBox(item, at.get(item.id), hood.root, from)).join('');
    const more = hood.truncated > 0 ? `<text class="rel" x="${width / 2}" y="${height - 14}" text-anchor="middle">+${hood.truncated}</text>` : '';
    const open = `<svg id="ego" data-testid="ego" viewBox="0 0 ${width} ${height}" ${camAttrs()}>${camOpen()}`;
    return `${open}${edges}${boxes}${more}</g></svg>`;
  }

  window.GyNode = { render };
})();