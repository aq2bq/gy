/* The crossing screen (d-c302, d-647f): a word, a period, or some scopes
   narrow the record; the set is drawn with rows for scopes and time across, or
   as a scope × week grid while nothing is narrowed or the set is too large
   (d-6d90). The placement is lanes.js's, and it is finite by construction. */
(function () {
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));
  const DAY = 86400;
  const KINDS = ['Need', 'Question', 'Decision', 'Requirement', 'Criterion'];
  const SHORT = { Need: 'Need', Question: 'Question', Decision: 'Decision', Requirement: 'Requirement', Criterion: 'Criterion' };
  const HEX = { Need: '#e9a63a', Question: '#b79cff', Decision: '#5fd3c7', Requirement: '#f28ba0', Criterion: '#8fd18f' };
  const REL = {
    'spawned-by': 'spawned-by', spawned: 'spawns', 'depends-on': 'depends-on', 'relies-on': 'relies-on',
    'waits-on': 'waits-on', serves: 'serves', closes: 'closes', narrows: 'narrows', widens: 'widens',
    supersedes: 'supersedes', completes: 'completes', 'filed-as': 'filed-as', raises: 'raised', targets: 'targets', files: 'files',
  };
  const pad = value => String(value).padStart(2, '0');
  const dateOf = at => {
    const date = new Date(at * 1000);
    return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
  };
  const md = at => {
    const date = new Date(at * 1000);
    return `${date.getMonth() + 1}/${date.getDate()}`;
  };
  const trunc = (text, n) => (String(text).length > n ? `${String(text).slice(0, n - 1)}…` : String(text));
  const at = value => window.GyShape.at(value);

  /* What the route narrows: the words, the period, the scopes. */
  function narrowed(state) {
    const query = state.route.query || {};
    return {
      q: query.q || '',
      from: query.from || '',
      to: query.to || '',
      scope: Array.isArray(query.scope) ? query.scope : query.scope ? [query.scope] : [],
    };
  }
  const any = narrow => !!(narrow.q || narrow.from || narrow.scope.length);

  /* The hash for a narrowing; an empty one is the entry grid. */
  function hash(query) {
    const params = new URLSearchParams();
    if (query.q) params.set('q', query.q);
    if (query.from) {
      params.set('from', query.from);
      params.set('to', query.to || query.from);
    }
    for (const scope of query.scope || []) params.append('scope', scope);
    const text = params.toString();
    return `#/cross${text ? `?${text}` : ''}`;
  }

  function render(state, el, ui) {
    if (!el) return;
    const data = state.cross ? window.GyShape.cross(state.cross) : null;
    const query = narrowed(state);
    const open = any(query);
    const ledger = ui.ledger;
    const sub = fill(ui.t('crossSub'), { ledger: esc(ledger) });
    el.innerHTML = `<div class="sub">${sub}</div><h1 class="hd">${esc(ui.t('crossTitle'))}</h1>` +
      `<div class="cross"><div class="panel" id="filters"></div><div class="stage" id="stage"></div><div class="panel" id="summary"></div></div>` +
      status(state, 'cross', !!state.cross, ui);
    el.querySelector('#filters').innerHTML = filters(state, data, query, open, ui);
    attachFilters(el, state, query, ui);
    if (!data) return;
    drawStage(el, data, query, open, ui);
    el.querySelector('#summary').innerHTML = summary(data, query, open, ui);
  }

  /* The left panel: the word, the period, and the scopes. */
  function filters(state, data, query, open, ui) {
    const names = scopeNames(state, data);
    const counts = {};
    for (const row of (data ? data.scopes : [])) counts[row.name] = row.kinds.reduce((sum, item) => sum + item.total, 0);
    const chips = names.map(name => `<button class="chip ${query.scope.includes(name) ? 'on' : ''}" style="--h:${ui.hue(name)}" data-scope="${esc(name)}" data-testid="scopeChip">${esc(name)}<small>${counts[name] || 0}</small></button>`).join('');
    const busiest = data ? busiestWeek(data) : null;
    return `<h3>${esc(ui.t('wordTitle'))} <span class="sub">${esc(ui.t('wordSub'))}</span></h3>` +
      `<input class="q" id="cq" data-testid="crossWord" value="${esc(query.q)}" placeholder="${esc(ui.t('wordPh'))}" aria-label="${esc(ui.t('wordTitle'))}">` +
      `<h3>${esc(ui.t('periodTitle'))} <span class="sub">${esc(ui.t('periodSub'))}</span></h3>` +
      `<div class="range"><input type="date" id="cf" data-testid="fromDay" value="${esc(query.from)}" aria-label="${esc(ui.t('fromDay'))}"><span>–</span><input type="date" id="ct" data-testid="toDay" value="${esc(query.to || query.from)}" aria-label="${esc(ui.t('toDay'))}"></div>` +
      `<div class="chips"><button class="chip" id="last7">${esc(ui.t('last7'))}</button>` +
      (query.from ? `<button class="chip" id="clearPeriod">${esc(ui.t('clearPeriod'))}</button>` : '') +
      (busiest ? `<button class="chip" id="busy">${esc(ui.t('busyWeek'))} ${md(busiest.week)}<small>${busiest.count}</small></button>` : '') + `</div>` +
      `<h3>${esc(ui.t('scopesTitle'))}</h3><div class="chips">${chips}</div>` +
      (open ? `<div class="chips" style="margin-top:16px"><a class="chip" href="#/cross">${esc(ui.t('clearAll'))}</a></div>` : '');
  }

  /* The scopes to offer: the ledger's own, then any the answer adds. */
  function scopeNames(state, data) {
    const found = [];
    const push = name => { if (name && !found.includes(name)) found.push(name); };
    for (const item of (state.shell ? state.shell.scopes : [])) push(item.name);
    for (const item of (data ? data.scopes : [])) push(item.name);
    return found;
  }

  /* The week with the most moves in the grid, for the one-click period. */
  function busiestWeek(data) {
    const byWeek = {};
    for (const cell of data.cells) {
      if (cell.week === null) continue;
      byWeek[cell.week] = (byWeek[cell.week] || 0) + cell.count;
    }
    const weeks = Object.keys(byWeek).map(Number).sort((a, b) => a - b);
    if (!weeks.length) return null;
    const week = weeks.reduce((best, one) => (byWeek[one] > byWeek[best] ? one : best), weeks[0]);
    return { week, count: byWeek[week], to: week + 6 * DAY };
  }

  function attachFilters(el, state, query, ui) {
    const word = el.querySelector('#cq');
    word.onkeydown = event => {
      if (event.key !== 'Enter') return;
      ui.navigate(hash({ ...query, q: word.value.trim() }));
    };
    el.querySelectorAll('[data-scope]').forEach(button => {
      button.onclick = () => {
        const name = button.dataset.scope;
        const scope = query.scope.includes(name) ? query.scope.filter(one => one !== name) : [...query.scope, name];
        ui.navigate(hash({ ...query, scope }));
      };
    });
    const from = el.querySelector('#cf');
    const to = el.querySelector('#ct');
    const period = () => {
      if (!from.value) { ui.navigate(hash({ ...query, from: '', to: '' })); return; }
      ui.navigate(hash({ ...query, from: from.value, to: to.value || from.value }));
    };
    from.onchange = period;
    to.onchange = period;
    const last7 = el.querySelector('#last7');
    if (last7) last7.onclick = () => {
      const now = state.cross ? window.GyShape.cross(state.cross).now : null;
      const end = now === null ? Math.floor(Date.now() / 1000) : now;
      ui.navigate(hash({ ...query, from: dateOf(end - 6 * DAY), to: dateOf(end) }));
    };
    const busy = el.querySelector('#busy');
    if (busy) busy.onclick = () => {
      const found = busiestWeek(window.GyShape.cross(state.cross));
      if (found) ui.navigate(hash({ ...query, from: dateOf(found.week), to: dateOf(found.to) }));
    };
    const clear = el.querySelector('#clearPeriod');
    if (clear) clear.onclick = () => ui.navigate(hash({ ...query, from: '', to: '' }));
  }

  /* The middle: the picture, or the grid that asks for a narrower cut. */
  function drawStage(el, data, query, open, ui) {
    const stage = el.querySelector('#stage');
    const total = data.capped ? '900+' : String(data.nodes.length);
    const what = open
      ? `<span class="what">${esc(describe(query, ui))} <b>${total}</b> ${esc(ui.t('nodesUnit'))}</span>`
      : `<span class="what">${esc(ui.t('notNarrowed'))}</span>`;
    stage.innerHTML = `<div class="bar">${what}</div><div id="canvas"></div>` +
      `<div class="legend">${KINDS.map(kind => `<span><i class="dotk o" style="color:${HEX[kind]}"></i>${esc(ui.t(SHORT[kind]))}</span>`).join('')}` +
      `<span><i class="dotk o" style="color:var(--paper-2)"></i>${esc(ui.t('legendOpen'))}</span>` +
      `<span><i class="dotk c" style="color:var(--paper-2)"></i>${esc(ui.t('legendClosed'))}</span>` +
      `<span>${esc(ui.t('legendSize'))}</span>` +
      `<span><svg width="26" height="8"><path d="M1 4 H25" stroke="#d9c8ff" stroke-width="1.6"/></svg>${esc(ui.t('legendAcross'))}</span></div>`;
    const canvas = stage.querySelector('#canvas');
    if (!open || data.capped) {
      heat(canvas, data, query, ui);
      if (open && data.capped) {
        canvas.insertAdjacentHTML('afterbegin', `<div class="empty-msg" data-testid="capped">${esc(fill(ui.t('capped'), { n: data.nodes.length || '900+' }))}</div>`);
      }
      return;
    }
    if (!data.nodes.length) {
      canvas.innerHTML = `<div class="empty-msg">${esc(ui.t('noHits'))}</div>`;
      return;
    }
    lanes(canvas, data, query, ui);
  }

  /* What the person narrowed, in one line. */
  function describe(query, ui) {
    const parts = [];
    if (query.q) parts.push(fill(ui.t('descWord'), { q: esc(query.q) }));
    if (query.from) parts.push(fill(ui.t('descPeriod'), { from: md(at(query.from)), to: md(at(query.to || query.from)) }));
    if (query.scope.length) parts.push(fill(ui.t('descScope'), { scopes: query.scope.map(esc).join(' ') }));
    return parts.join(' ');
  }

  /* Rows are scopes, the width is time; the placement is lanes.js's and every
     coordinate is finite by construction (ac-afba). */
  function lanes(canvas, data, query, ui) {
    const width = Math.max(360, canvas.clientWidth || canvas.parentElement.clientWidth || 900);
    const from = query.from ? at(query.from) : undefined;
    const to = query.from ? at(query.to || query.from) + DAY : undefined;
    const core = {};
    for (const node of data.nodes) core[node.id] = true;
    const lay = window.GyLanes.layout({
      nodes: data.nodes,
      edges: data.edges,
      width,
      from,
      to,
      scopes: data.scopes.map(row => row.name),
      core,
    });
    const step = lay.to - lay.from > 40 * DAY ? 7 * DAY : lay.to - lay.from > 10 * DAY ? 2 * DAY : DAY;
    let svg = `<svg width="${lay.width}" height="${lay.height}" role="img" aria-label="${esc(ui.t('crossTitle'))}">`;
    lay.rows.forEach((row, index) => {
      svg += `<rect x="0" y="${row.y}" width="${lay.width}" height="${row.h}" fill="hsl(${ui.hue(row.scope)} 40% 30% / ${index % 2 ? 0.1 : 0.17})"/>` +
        `<text class="lane-l" x="12" y="${row.y + 20}">${esc(row.scope)}</text>` +
        `<text class="lane-c" x="12" y="${row.y + 35}">${fill(esc(ui.t('rowCount')), { n: row.count, o: row.open })}</text>`;
    });
    svg += `<g class="axis">`;
    for (let tick = Math.ceil(lay.from / DAY) * DAY; tick < lay.to; tick += step) {
      const x = lay.xOf(tick);
      svg += `<line x1="${x}" x2="${x}" y1="26" y2="${lay.height}" stroke="#ffffff" stroke-opacity=".05"/>` +
        `<text x="${x + 3}" y="20">${md(tick)}</text>`;
    }
    svg += `</g>`;
    for (const edge of lay.edges) {
      const a = edge.a;
      const b = edge.b;
      const middle = (a.x + b.x) / 2;
      const lift = (a.y + b.y) / 2 - Math.min(40, Math.abs(a.x - b.x) * 0.15);
      const color = edge.cross ? '#d9c8ff' : '#8f9bbd';
      const opacity = edge.cross ? 0.7 : 0.28;
      svg += `<path d="M${a.x},${a.y} Q${middle},${lift} ${b.x},${b.y}" fill="none" stroke="${color}" stroke-opacity="${opacity}" stroke-width="${edge.cross ? 1.6 : 1}"><title>${esc(ui.t(`rel.${EDGE[edge.relation] || edge.relation}`))}</title></path>`;
    }
    for (const point of lay.points) {
      const node = point.node;
      const openState = window.GyLanes.isOpen(node.state);
      svg += `<a href="#/n/${encodeURIComponent(node.id)}"><circle class="nd" data-testid="laneNode" cx="${point.x.toFixed(1)}" cy="${point.y.toFixed(1)}" r="${point.r.toFixed(1)}" fill="${openState ? HEX[node.kind] : '#0d1220'}" stroke="${HEX[node.kind] || '#8f9bbd'}" stroke-width="${openState ? 0 : 1.6}"><title>${esc(`${node.id} ${node.title} · ${ui.t(`st.${node.kind}.${node.state}`)}`)}</title></circle></a>`;
    }
    canvas.innerHTML = `${svg}</svg>`;
  }

  /* The relation names the server stores, as the dictionary's keys. */
  const EDGE = { spawned: 'spawns', raised: 'raises' };

  /* The grid: a scope × week count, pressed to narrow that scope and week. */
  function heat(canvas, data, query, ui) {
    const weeks = [...new Set(data.cells.map(cell => cell.week).filter(week => week !== null))].sort((a, b) => a - b);
    const scopes = [...new Set(data.cells.map(cell => cell.scope))];
    const order = scopeNamesFrom(data, scopes, query);
    const byKey = {};
    let max = 1;
    for (const cell of data.cells) {
      if (cell.week === null) continue;
      byKey[`${cell.scope}:${cell.week}`] = (byKey[`${cell.scope}:${cell.week}`] || 0) + cell.count;
      max = Math.max(max, byKey[`${cell.scope}:${cell.week}`]);
    }
    if (!weeks.length) {
      canvas.insertAdjacentHTML('beforeend', `<div class="empty-msg">${esc(ui.t('noGrid'))}</div>`);
      return;
    }
    let html = `<div class="heatwrap"><table class="heat"><tr><th></th>${weeks.map(week => `<th>${md(week)}〜</th>`).join('')}</tr>`;
    for (const scope of order) {
      html += `<tr><th class="s">${ui.scopeTag(scope)}</th>`;
      for (const week of weeks) {
        const value = byKey[`${scope}:${week}`] || 0;
        const alpha = value ? 0.12 + 0.75 * Math.sqrt(value / max) : 0;
        const href = hash({ ...query, scope: [scope], from: dateOf(week), to: dateOf(week + 6 * DAY) });
        html += `<td>${value ? `<a data-testid="heatCell" href="${href}" style="background:hsl(${ui.hue(scope)} 55% 50% / ${alpha})" title="${md(week)} ${scope}: ${value}">${value}</a>` : '<span class="z"></span>'}</td>`;
      }
      html += `</tr>`;
    }
    html += `</table></div><div class="heatnote">${esc(ui.t('heatSub'))}</div>`;
    canvas.insertAdjacentHTML('beforeend', html);
  }

  function scopeNamesFrom(data, scopes, query) {
    const wanted = query.scope;
    const found = [];
    const push = name => { if (name && !found.includes(name)) found.push(name); };
    for (const name of wanted) push(name);
    for (const name of scopes) push(name);
    for (const row of data.scopes) push(row.name);
    return found;
  }

  /* The right panel: the scale, what happened in the period, and the nodes. */
  function summary(data, query, open, ui) {
    if (!open) {
      return `<h3>${esc(ui.t('crossHintTitle'))}</h3><div class="hint">${ui.t('crossHint')}</div>`;
    }
    const scopes = data.scopes;
    const table = `<table class="sumtab"><tr><th>${esc(ui.t('scopeCol'))}</th>${KINDS.map(kind => `<th style="color:${HEX[kind]}">${esc(ui.t(SHORT[kind]))}</th>`).join('')}</tr>` +
      scopes.map(scopeRow => `<tr><td>${ui.scopeTag(scopeRow.name)}</td>${KINDS.map(kind => {
        const found = scopeRow.kinds.find(item => item.kind === kind);
        if (!found) return `<td class="dim">·</td>`;
        const openText = found.open && kind !== 'Decision' ? `<span class="c" style="color:var(--need)">·${found.open}</span>` : '';
        return `<td><span class="o">${found.total}</span>${openText}</td>`;
      }).join('')}</tr>`).join('') + `</table>`;
    const counts = Object.fromEntries(data.events.map(event => [event.kind, event.count]));
    const order = ['decided', 'question-closed', 'question-raised', 'requirement-approved', 'requirement-done', 'criterion-satisfied', 'need-closed'];
    const happened = query.from
      ? `<h3>${esc(ui.t('eventsTitle'))}</h3><div class="evsum">${order.filter(key => counts[key]).map(key => `<b>${counts[key]}</b><span>${esc(ui.t(`ev.${key}`))}</span>`).join('')}</div>`
      : '';
    const grouped = groupNodes(data.nodes);
    const list = grouped.map(group => `<div class="gh">${esc(group.scope)} · ${group.nodes.length}</div>` +
      group.nodes.slice(0, 30).map(node => `<a data-testid="crossNode" href="#/n/${encodeURIComponent(node.id)}"><span class="dotk ${window.GyLanes.isOpen(node.state) ? 'o' : 'c'}" style="color:${HEX[node.kind]}"></span><span class="mono dim">${esc(node.id)}</span><span class="t">${esc(node.title)}</span></a>`).join('') +
      (group.nodes.length > 30 ? `<div class="dim" style="font-size:11px">${fill(esc(ui.t('moreNodes')), { n: group.nodes.length - 30 })}</div>` : '')).join('');
    const total = data.capped ? '900+' : String(data.nodes.length);
    const across = data.edges.filter(edge => scopeOf(data, edge.from) !== scopeOf(data, edge.to)).length;
    return `<h3>${esc(ui.t('scaleTitle'))}</h3><div class="big">${total}<small>${esc(fill(ui.t('scaleSub'), { s: scopes.length, e: across }))}</small></div>${table}` +
      `<div class="dim" style="font-size:11px;margin-top:4px">${esc(ui.t('scaleNote'))}</div>` +
      happened + (data.nodes.length ? `<h3>${esc(ui.t('nodeList'))}</h3><div class="nlist">${list}</div>` : '');
  }

  function scopeOf(data, id) {
    const node = data.nodes.find(item => item.id === id);
    return node ? node.scope : null;
  }

  /* The nodes by scope, open first, then the kind order the API uses. */
  function groupNodes(nodes) {
    const groups = [];
    const seen = new Map();
    for (const node of nodes) {
      if (!seen.has(node.scope)) {
        const group = { scope: node.scope, nodes: [] };
        seen.set(node.scope, group);
        groups.push(group);
      }
      seen.get(node.scope).nodes.push(node);
    }
    for (const group of groups) {
      group.nodes.sort((a, b) => (Number(window.GyLanes.isOpen(b.state)) - Number(window.GyLanes.isOpen(a.state))) ||
        KINDS.indexOf(a.kind) - KINDS.indexOf(b.kind) || (b.created || 0) - (a.created || 0));
    }
    return groups;
  }

  function status(state, kind, loaded, ui) {
    if (state.failed[kind]) {
      return `<div class="load fail" data-testid="failed"><b>${esc(ui.t('loadFailed'))}</b> <span>${esc(state.failed[kind])}</span><button data-act="retry">${esc(ui.t('retry'))}</button></div>`;
    }
    if (!loaded) return `<div class="load" data-testid="loading">${esc(ui.t('loading'))}</div>`;
    return '';
  }

  window.GyCross = { render };
})();
