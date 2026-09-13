(() => {
  // src/data.ts
  var D = window.GY_DATA;
  var NODES = D.nodes;
  var EDGES = D.edges;
  var DEP = D.dependencies;
  var byId = {};
  NODES.forEach((n) => byId[n.id] = n);
  var states = D.states || {};
  var lintArr = D.lint || [];

  // src/dom.ts
  function element(id) {
    return document.getElementById(id);
  }

  // src/state.ts
  var MODE_THRESHOLD = 60;
  var HOP_CAP = 5;
  var EDGE_LABEL_MAX = 20;
  var MAX_LABEL_W = 190;
  var searchText = "";
  var searchHits = null;
  var focusHistory = [{ id: null, radius: null }];
  function currentFocus() {
    return focusHistory[focusHistory.length - 1];
  }
  var genealogyMode = false;
  var selected = null;
  var scale = 1;
  var translate = { x: 0, y: 0 };
  var viewSource = "fit";
  var lod = "far";
  var positions = {};
  var genealogyLayout = {};
  var forceLayout = {};
  var forceKey = "";
  var adj = {};
  NODES.forEach((n) => adj[n.id] = {});
  EDGES.forEach((e) => {
    (adj[e.source] = adj[e.source] || {})[e.target] = e.label;
    (adj[e.target] = adj[e.target] || {})[e.source] = e.reverse;
  });
  var degree = Object.fromEntries(NODES.map((n) => [n.id, Object.keys(adj[n.id]).length]));
  function reachable(start, n) {
    let seen = new Set([start]);
    let frontier = [start];
    for (let i = 0;i < n; i++) {
      const next = [];
      frontier.forEach((id) => Object.keys(adj[id] || {}).forEach((t) => {
        if (!seen.has(t)) {
          seen.add(t);
          next.push(t);
        }
      }));
      frontier = next;
    }
    return seen;
  }
  function pickHop(start) {
    for (let n = HOP_CAP;n >= 1; n--) {
      const s2 = reachable(start, n);
      if (s2.size <= MODE_THRESHOLD)
        return { n, size: s2.size };
    }
    const s = reachable(start, 1);
    return { n: 1, size: s.size };
  }
  var typeState = { need: true, question: true, decision: true, requirement: true, criterion: true, gate: true };
  var filterState = { scopeSel: "", stateSel: "", qstatus: "", criterion: "" };
  var radiusChoice = "";
  var activeTab = "overview";
  var dragging = false;
  var dragStart = null;
  var dragMoved = false;
  var lastFitKey = "";
  var lastViewport = null;
  function enterFocus(id, radius) {
    if (currentFocus().id !== id || currentFocus().radius !== radius)
      focusHistory = [...focusHistory, { id, radius }];
  }
  function truncateFocus(length) {
    focusHistory = focusHistory.slice(0, length);
  }
  function restoreFocus(value) {
    const restored = [{ id: null, radius: null }];
    if (Array.isArray(value))
      value.forEach((f) => {
        if (f && typeof f.id === "string" && Object.hasOwn(byId, f.id) && Number.isInteger(f.radius) && f.radius >= 1 && f.radius <= HOP_CAP)
          restored.push({ id: f.id, radius: f.radius });
      });
    focusHistory = restored;
  }
  function setType(kind, enabled) {
    typeState = { ...typeState, [kind]: enabled };
  }
  function setFilter(field, value) {
    filterState = { ...filterState, [field]: value };
  }
  function setSearchText(value) {
    searchText = value;
  }
  function setSearchHits(value) {
    searchHits = value;
  }
  function setGenealogyMode(value) {
    genealogyMode = value;
  }
  function setSelected(value) {
    selected = value;
  }
  function setScale(value) {
    scale = value;
  }
  function setTranslate(value) {
    translate = value;
  }
  function setViewSource(value) {
    viewSource = value;
  }
  function setLod(value) {
    lod = value;
  }
  function setGenealogyLayout(value) {
    genealogyLayout = value;
  }
  function setForceLayout(value) {
    forceLayout = value;
  }
  function setForceKey(value) {
    forceKey = value;
  }
  function setRadiusChoice(value) {
    radiusChoice = value;
  }
  function setActiveTab(value) {
    activeTab = value;
  }
  function setDragging(value) {
    dragging = value;
  }
  function setDragStart(value) {
    dragStart = value;
  }
  function setDragMoved(value) {
    dragMoved = value;
  }
  function setLastFitKey(value) {
    lastFitKey = value;
  }
  function setLastViewport(value) {
    lastViewport = value;
  }
  function locationHash(state) {
    const prefix = "view=";
    if (state !== undefined)
      return "#" + prefix + encodeURIComponent(state);
    try {
      const hash = decodeURIComponent(location.hash.slice(1));
      const value = hash.startsWith(prefix) ? JSON.parse(hash.slice(prefix.length)) : hash ? { selected: hash } : {};
      return value && typeof value === "object" ? value : {};
    } catch (_) {
      return {};
    }
  }
  function focusPlan(id) {
    const radius = Number(radiusChoice);
    return radius ? { n: radius, size: reachable(id, radius).size } : pickHop(id);
  }
  function focusLabel(id) {
    if (!Object.hasOwn(byId, id))
      return "Show neighbors";
    const plan = focusPlan(id);
    return id + ": show " + plan.n + (plan.n === 1 ? " hop" : " hops") + " · " + plan.size + (plan.size === 1 ? " node" : " nodes");
  }
  function setPositions(value) {
    positions = value;
  }
  var listSort = { key: "id", direction: "asc" };
  function setListSort(value) {
    const v = value;
    listSort = v && ["id", "type", "title", "scope", "status", "created"].includes(v.key) && ["asc", "desc"].includes(v.direction) ? { key: v.key, direction: v.direction } : { key: "id", direction: "asc" };
  }
  function selectType(kind) {
    Object.keys(typeState).forEach((k) => setType(k, kind === "all" || k === kind));
  }
  var overviewSource = "";
  function setOverviewSource(value) {
    overviewSource = typeof value === "string" && ["next", "lint-error", "lint-warn"].includes(value) ? value : "";
  }

  // src/detail/markdown.ts
  function inline(source) {
    const s = String(source), out = [];
    for (let i = 0;i < s.length; ) {
      if (s[i] === "\\" && s[i + 1] && /[^\w\s]/.test(s[i + 1])) {
        out.push(esc(s[i + 1]));
        i += 2;
        continue;
      }
      if (s[i] === "<" && /^<(?:\/?[A-Za-z]|!--)/.test(s.slice(i))) {
        let end = i + 1, quote = "";
        for (;end < s.length; end++) {
          if (quote) {
            if (s[end] === quote)
              quote = "";
          } else if (s[end] === '"' || s[end] === "'")
            quote = s[end];
          else if (s[end] === ">")
            break;
        }
        if (end < s.length) {
          out.push(esc(s.slice(i, end + 1)));
          i = end + 1;
          continue;
        }
      }
      if (s[i] === "`") {
        const ticks = s.slice(i).match(/^`+/)[0], end = s.indexOf(ticks, i + ticks.length);
        if (end >= 0) {
          let code = s.slice(i + ticks.length, end).replace(/\n/g, " ");
          if (/^ .* $/.test(code) && code.trim())
            code = code.slice(1, -1);
          out.push("<code>" + esc(code) + "</code>");
          i = end + ticks.length;
          continue;
        }
      }
      if (s[i] === "[") {
        const middle = s.indexOf("](", i + 1);
        if (middle >= 0) {
          let end = middle + 2, depth = 1;
          for (;end < s.length; end++) {
            if (s[end] === "\\") {
              end++;
              continue;
            }
            if (s[end] === "(")
              depth++;
            if (s[end] === ")" && --depth === 0)
              break;
          }
          if (depth === 0) {
            const destination = s.slice(middle + 2, end).match(/^(?:<([^>]*)>|(\S+?))(?:\s+"([^"]*)")?$/);
            if (destination) {
              const href = (destination[1] ?? destination[2]).replace(/\\([()])/g, "$1");
              let safe = false;
              try {
                safe = ["http:", "https:", "mailto:", "file:"].includes(new URL(href, location.href).protocol) && !/[\u0000-\u0020]/.test(href);
              } catch (_) {}
              const label = inline(s.slice(i + 1, middle));
              out.push(safe ? '<a href="' + esc(href) + '" rel="noopener noreferrer"' + (destination[3] ? ' title="' + esc(destination[3]) + '"' : "") + ">" + label + "</a>" : label + " (" + esc(href) + ")");
              i = end + 1;
              continue;
            }
          }
        }
      }
      let matched = false;
      for (const [mark, open, close] of [["***", "<strong><em>", "</em></strong>"], ["___", "<strong><em>", "</em></strong>"], ["**", "<strong>", "</strong>"], ["__", "<strong>", "</strong>"], ["~~", "<del>", "</del>"], ["*", "<em>", "</em>"], ["_", "<em>", "</em>"]]) {
        if (!s.startsWith(mark, i) || /\s/.test(s[i + mark.length] || " ") || mark[0] === "_" && /\w/.test(s[i - 1] || ""))
          continue;
        let end = s.indexOf(mark, i + mark.length);
        while (end >= 0 && s[end - 1] === "\\")
          end = s.indexOf(mark, end + mark.length);
        if (end > i + mark.length && !/\s/.test(s[end - 1])) {
          out.push(open + inline(s.slice(i + mark.length, end)) + close);
          i = end + mark.length;
          matched = true;
          break;
        }
      }
      if (!matched) {
        out.push(esc(s[i]));
        i++;
      }
    }
    return out.join("");
  }
  function tableCells(line) {
    const cells = [], text = line.trim();
    let cell = "", ticks = 0;
    for (let i = 0;i < text.length; i++) {
      if (text[i] === "\\" && text[i + 1] === "|") {
        cell += "\\|";
        i++;
        continue;
      }
      if (text[i] === "`") {
        const run = text.slice(i).match(/^`+/)[0];
        ticks = ticks === run.length ? 0 : ticks || run.length;
        cell += run;
        i += run.length - 1;
        continue;
      }
      if (text[i] === "|" && !ticks) {
        cells.push(cell.trim());
        cell = "";
      } else
        cell += text[i];
    }
    cells.push(cell.trim());
    if (text.startsWith("|"))
      cells.shift();
    if (text.endsWith("|") && cells[cells.length - 1] === "")
      cells.pop();
    return cells;
  }
  function renderBody(md) {
    const lines = String(md || "").replace(/\r\n?/g, `
`).split(`
`);
    const fence = (line) => line.match(/^ {0,3}(`{3,}|~{3,})(.*)$/);
    const indentLine = (line) => line.replace(/^[ \t]*/, (prefix) => {
      let width = 0;
      for (const c of prefix)
        width += c === "\t" ? 4 - width % 4 : 1;
      return " ".repeat(width);
    });
    const item = (line) => indentLine(line).match(/^( *)([-+*]|\d+[.)]) +(.*)$/);
    const heading = (line) => line.match(/^ {0,3}(#{1,6})\s+(.*)$/);
    function blocks(lines2) {
      const html = [];
      let i = 0;
      const isTable = (at) => at + 1 < lines2.length && lines2[at].includes("|") && tableCells(lines2[at]).length > 0 && tableCells(lines2[at + 1]).length === tableCells(lines2[at]).length && tableCells(lines2[at + 1]).every((c) => /^:?-+:?$/.test(c));
      while (i < lines2.length) {
        const line = lines2[i];
        if (!line.trim()) {
          i++;
          continue;
        }
        const f = fence(line);
        if (f) {
          const code = [], close = new RegExp("^ {0,3}" + f[1][0] + "{" + f[1].length + ",}\\s*$");
          i++;
          while (i < lines2.length && !close.test(lines2[i]))
            code.push(lines2[i++]);
          if (i < lines2.length)
            i++;
          const language = f[2].trim().split(/\s/)[0];
          html.push("<pre><code" + (language ? ' class="language-' + esc(language) + '"' : "") + ">" + esc(code.join(`
`) + (code.length ? `
` : "")) + "</code></pre>");
          continue;
        }
        if (/^ {0,3}<(?:\/?[A-Za-z]|!--)/.test(line)) {
          const literal = [];
          while (i < lines2.length && lines2[i].trim())
            literal.push(lines2[i++]);
          html.push("<p>" + esc(literal.join(`
`)) + "</p>");
          continue;
        }
        const h = heading(line);
        if (h) {
          html.push("<h4>" + inline(h[2].replace(/\s+#+\s*$/, "")) + "</h4>");
          i++;
          continue;
        }
        if (isTable(i)) {
          const headers = tableCells(line), align = tableCells(lines2[i + 1]).map((c) => c.endsWith(":") ? c.startsWith(":") ? "center" : "right" : "left");
          const row = (cells, tag) => "<tr>" + headers.map((_, n) => "<" + tag + ' style="text-align:' + align[n] + '">' + inline(cells[n] || "") + "</" + tag + ">").join("") + "</tr>";
          html.push("<table><thead>" + row(headers, "th") + "</thead><tbody>");
          i += 2;
          while (i < lines2.length && lines2[i].trim() && lines2[i].includes("|"))
            html.push(row(tableCells(lines2[i++]), "td"));
          html.push("</tbody></table>");
          continue;
        }
        const first = item(line);
        if (first) {
          const indent = first[1].length, ordered = /^\d/.test(first[2]), tag = ordered ? "ol" : "ul";
          html.push("<" + tag + (ordered ? ' start="' + parseInt(first[2], 10) + '"' : "") + ">");
          while (i < lines2.length) {
            const entry = item(lines2[i]);
            if (!entry || entry[1].length !== indent || /^\d/.test(entry[2]) !== ordered)
              break;
            const contentIndent = indentLine(lines2[i]).length - entry[3].length, content = [entry[3]];
            i++;
            while (i < lines2.length) {
              const next = indentLine(lines2[i]), spaces = next.match(/^ */)[0].length;
              if (!next.trim()) {
                content.push("");
                i++;
                continue;
              }
              if (spaces > indent) {
                content.push(next.slice(Math.min(contentIndent, spaces)));
                i++;
                continue;
              }
              if (item(next) || heading(next) || fence(next) || content[content.length - 1] === "")
                break;
              content.push(next);
              i++;
            }
            html.push("<li>" + blocks(content) + "</li>");
          }
          html.push("</" + tag + ">");
          continue;
        }
        if (/^ {0,3}>/.test(line)) {
          const quoted = [];
          while (i < lines2.length && /^ {0,3}>/.test(lines2[i]))
            quoted.push(lines2[i++].replace(/^ {0,3}> ?/, ""));
          html.push("<blockquote>" + blocks(quoted) + "</blockquote>");
          continue;
        }
        const paragraph = [line];
        i++;
        while (i < lines2.length && lines2[i].trim() && !fence(lines2[i]) && !heading(lines2[i]) && !item(lines2[i]) && !/^ {0,3}>/.test(lines2[i]) && !isTable(i))
          paragraph.push(lines2[i++]);
        html.push("<p>" + inline(paragraph.join(`
`)) + "</p>");
      }
      return html.join(`
`);
    }
    return blocks(lines);
  }

  // src/detail/index.ts
  function relFor(id) {
    const out = [];
    EDGES.forEach((e) => {
      if (e.source === id)
        out.push({ other: e.target, label: e.label, dir: "→" });
      if (e.target === id)
        out.push({ other: e.source, label: e.reverse, dir: "←" });
    });
    return out;
  }
  var DETAIL_FIELDS = {
    created: "Created",
    closed_at: "Closed",
    satisfied_at: "Satisfied at",
    parent_issue: "Parent issue",
    pr_url: "Pull request",
    responsible: "Responsible",
    decider: "Decider",
    next_evidence: "Next evidence",
    closure_note: "Closure note",
    evidence: "Evidence",
    summary: "Summary",
    contracts_changed: "Contracts changed",
    artifacts: "Artifacts",
    production: "Production",
    deviations: "Deviations",
    residual: "Residual",
    compressed_from: "Archived record"
  };
  var LINEAGE_LABELS = new Set([
    "narrows",
    "narrowed-by",
    "widens",
    "widened-by",
    "supersedes",
    "superseded-by",
    "completes",
    "completed-by"
  ]);
  function detailValue(value) {
    return typeof value === "string" ? value : JSON.stringify(value, null, 2);
  }
  function valueHTML(value) {
    const text = detailValue(value);
    if (typeof value === "string" && /^https?:\/\/[^\s]+$/i.test(value)) {
      return '<a href="' + esc(value) + '" target="_blank" rel="noopener noreferrer">' + esc(value) + "</a>";
    }
    return esc(text);
  }
  function nodeLink(id) {
    return byId[id] ? '<button class="node-link" data-go="' + esc(id) + '">' + esc(id) + "</button>" : esc(id);
  }
  function detailBadge(field, label, value, tone = "") {
    return '<span class="detail-badge ' + tone + '" data-field="' + field + '"><span class="badge-label">' + label + "</span> " + esc(value) + "</span>";
  }
  function proseClass(text) {
    const letters = String(text).match(/\p{L}/gu) || [];
    const cjk = String(text).match(/[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}]/gu) || [];
    return cjk.length > letters.length / 2 ? "prose-ja" : "prose-latin";
  }
  function selectNode(id) {
    showDetail(id);
    redraw();
  }
  function showDetail(id) {
    const n = byId[id];
    if (!n)
      return;
    const changed = selected !== id;
    setSelected(id);
    element("hopFrom").value = id;
    const consumed = new Set(["id", "type", "scope", "title"]);
    let badges = detailBadge("type", "Type", n.type, "kind-" + n.type) + detailBadge("scope", "Scope", n.scope);
    if (typeof n.attrs.status === "string") {
      badges += detailBadge("status", "Status", n.state || n.status);
      if (n.state && n.status !== n.state)
        badges += '<span class="status-note">' + esc(n.status) + "</span>";
      consumed.add("status");
    }
    if (n.type === "criterion") {
      badges += detailBadge("satisfied", "Acceptance", n.attrs.satisfied === true ? "Satisfied" : "Not satisfied", n.attrs.satisfied === true ? "positive" : "");
      if (typeof n.attrs.satisfied === "boolean")
        consumed.add("satisfied");
    }
    if (n.type === "question" && typeof n.attrs.closed_by === "string") {
      badges += detailBadge("closed_by", "Closed by", n.attrs.closed_by);
      consumed.add("closed_by");
    }
    const successors = n.superseded_by || [];
    if (n.type === "decision")
      badges += detailBadge("superseded", "Decision", successors.length ? "Superseded" : "Current", successors.length ? "superseded" : "positive");
    let declarations = "";
    for (const [key, label] of Object.entries(DETAIL_FIELDS)) {
      if (!Object.prototype.hasOwnProperty.call(n.attrs, key))
        continue;
      consumed.add(key);
      const value = n.attrs[key];
      const rendered = ["created", "closed_at", "satisfied_at"].includes(key) && typeof value === "string" ? '<time datetime="' + esc(value) + '">' + esc(value) + "</time>" : valueHTML(value);
      declarations += '<div class="declaration" data-field="' + key + '"><dt>' + label + "</dt><dd>" + rendered + "</dd></div>";
    }
    let scope = "";
    if (Object.prototype.hasOwnProperty.call(n.attrs, "decision_scope")) {
      consumed.add("decision_scope");
      const text = detailValue(n.attrs.decision_scope);
      scope = '<section id="detailScope"><h2>Applicability</h2><div class="detail-prose ' + proseClass(text) + '">' + renderBody(text) + "</div></section>";
    }
    let additional = "";
    for (const [key, value] of Object.entries(n.attrs)) {
      if (consumed.has(key))
        continue;
      additional += '<div class="additional-attribute" data-key="' + esc(key) + '"><dt>' + esc(key) + "</dt><dd><pre>" + esc(detailValue(value)) + "</pre></dd></div>";
    }
    const relations = relFor(id);
    const group = (title, items) => items.length ? '<section class="detail-relations"><h2>' + title + "</h2>" + items.map((r) => '<div class="rel"><span class="relation-chip">' + esc(r.dir + " " + r.label) + "</span> " + nodeLink(r.other) + "</div>").join("") + "</section>" : "";
    const dependencies = DEP.filter((d) => d.requirement === id);
    const depHTML = dependencies.length ? '<section id="detailDependencies"><h2>Decision dependencies</h2><table><thead><tr><th>Decision</th><th>Role</th><th>Superseded by</th></tr></thead><tbody>' + dependencies.map((d) => "<tr><td>" + nodeLink(d.decision) + "</td><td>" + esc(d.role) + "</td><td>" + (d.superseded_by || []).map(nodeLink).join(", ") + "</td></tr>").join("") + "</tbody></table></section>" : "";
    let displayBody = n.body || "";
    const marks = n.body_marks || [];
    marks.forEach((m) => {
      if (m.found && m.mark)
        displayBody = displayBody.split(m.mark).join(m.mark + " ⟦" + m.label + ": " + m.source + "⟧");
    });
    const markHTML = marks.length ? '<section id="detailMarks"><h2>Affected passages</h2>' + marks.map((m) => '<div class="passage-note" data-found="' + String(m.found) + '"><span class="relation-chip">' + esc(m.label) + "</span> " + nodeLink(m.source) + (m.mark ? "<blockquote>" + esc(m.mark) + "</blockquote><p>" + (m.found ? "Found in the source body." : "Location in body could not be found.") + "</p>" : "<p>No mark identifies the affected passage.</p>") + "</div>").join("") + "</section>" : "";
    element("detailBody").innerHTML = '<div class="detail-heading"><div class="detail-id">' + esc(n.id) + "</div><h3>" + esc(n.title) + '</h3><div class="detail-badges">' + badges + "</div>" + (successors.length ? '<div class="successors">Superseded by ' + successors.map(nodeLink).join(", ") + "</div>" : "") + "</div>" + scope + '<section id="detailContent"><h2>Body</h2><div id="body" class="detail-prose ' + proseClass(n.body || "") + '">' + renderBody(displayBody) + "</div></section>" + markHTML + (declarations ? '<section id="detailDeclarations"><h2>Declarations</h2><dl>' + declarations + "</dl></section>" : "") + group("Decision lineage", relations.filter((r) => LINEAGE_LABELS.has(r.label))) + group("Relationships", relations.filter((r) => !LINEAGE_LABELS.has(r.label))) + depHTML + (additional ? '<section id="detailAdditional"><h2>Additional attributes</h2><dl>' + additional + "</dl></section>" : "");
    const detail = element("detail");
    detail.classList.add("on");
    if (changed)
      detail.scrollTop = 0;
  }
  function hideDetail() {
    element("detail").classList.remove("on");
    setSelected(null);
  }
  function closeDetail() {
    hideDetail();
    redraw();
  }
  function initDetail() {
    element("closeDetail").addEventListener("click", closeDetail);
    element("detail").addEventListener("click", (e) => {
      const t = e.target.closest("[data-go]");
      if (t)
        selectNode(t.dataset.go);
    });
  }

  // src/survey/blockers.ts
  function jamList(ulEl, items, tag) {
    const ul = element(ulEl);
    ul.innerHTML = "";
    items.forEach((it) => {
      const li = document.createElement("li");
      const a = document.createElement("a");
      a.dataset.go = it.id;
      a.href = locationHash(JSON.stringify({ selected: it.id }));
      a.innerHTML = '<span class="id">' + esc(it.id) + '</span> <span class="lbl">' + esc(it.label) + "</span>";
      li.appendChild(a);
      ul.appendChild(li);
    });
    if (!items.length) {
      ul.innerHTML = '<li class="small">none</li>';
    }
  }
  function initBlockers() {
    jamList("jamQ", (D.open_questions || []).flatMap((q) => [
      { id: q.id, label: (q.title || "") + (q.referencing && q.referencing.length ? " — referenced by " + q.referencing.join(", ") : "") }
    ]), "q");
    jamList("jamNext", D.next || [], "next");
    jamList("jamMissing", (D.missing || []).map((m) => ({ id: m.id, label: "next_evidence " + (m.next_evidence ? "set" : "empty") + "; responsible " + (m.responsible ? "set" : "empty") })), "missing");
    jamList("jamDangling", (D.dangling || []).map((d) => ({ id: d.source, label: "references missing node " + d.target })), "dangling");
    renderLint();
    element("left").addEventListener("click", (e) => {
      const t = e.target.closest("[data-go]");
      if (t && !e.ctrlKey && !e.metaKey && !e.shiftKey && !e.altKey) {
        e.preventDefault();
        selectNode(t.dataset.go);
      }
    });
  }
  function renderLint() {
    const lintUl = element("jamLint");
    lintUl.innerHTML = "";
    const rows = lintArr.filter((d) => !overviewSource.startsWith("lint-") || d.severity === overviewSource.slice(5));
    rows.forEach((d) => {
      const li = document.createElement("li");
      const a = document.createElement("a");
      a.dataset.go = d.id;
      a.href = locationHash(JSON.stringify({ selected: d.id }));
      a.innerHTML = '<span class="lint-sev" style="color:' + (d.severity === "error" ? "var(--err)" : "var(--warn)") + '">' + esc(d.severity) + "</span> " + '<span class="id">' + esc(d.rule) + '</span> <span class="lbl">' + esc(d.id) + " — " + esc(d.message) + "</span>";
      li.appendChild(a);
      lintUl.appendChild(li);
    });
    if (!rows.length)
      lintUl.innerHTML = '<li class="small">no findings</li>';
  }

  // src/graph/layout.ts
  function computeForce(ids) {
    const n = ids.length;
    const W = 1400, H = 900;
    const pos = {};
    ids.forEach((id, i) => {
      const angle = 2 * Math.PI * i / Math.max(1, n);
      const r = Math.min(W, H) * 0.35;
      pos[id] = { x: W / 2 + Math.cos(angle) * r, y: H / 2 + Math.sin(angle) * r, vx: 0, vy: 0 };
    });
    const idSet = new Set(ids);
    const spring = [];
    EDGES.forEach((e) => {
      if (idSet.has(e.source) && idSet.has(e.target))
        spring.push([e.source, e.target]);
    });
    for (let iter = 0;iter < 400; iter++) {
      for (let i = 0;i < n; i++) {
        const a = pos[ids[i]];
        for (let j = i + 1;j < n; j++) {
          const b = pos[ids[j]];
          let dx = a.x - b.x, dy = a.y - b.y;
          let d = Math.max(1, Math.hypot(dx, dy));
          const f = 160000 / (d * d);
          dx /= d;
          dy /= d;
          a.vx += dx * f;
          a.vy += dy * f;
          b.vx -= dx * f;
          b.vy -= dy * f;
        }
      }
      spring.forEach(([s, t]) => {
        const a = pos[s], b = pos[t];
        let dx = a.x - b.x, dy = a.y - b.y;
        const d = Math.max(1, Math.hypot(dx, dy));
        const f = 0.03 * (d - 110);
        dx /= d;
        dy /= d;
        a.vx -= dx * f;
        a.vy -= dy * f;
        b.vx += dx * f;
        b.vy += dy * f;
      });
      ids.forEach((id) => {
        const p = pos[id];
        p.vx += (W / 2 - p.x) * 0.001;
        p.vy += (H / 2 - p.y) * 0.001;
        const damp = 0.85;
        p.x += p.vx * damp;
        p.y += p.vy * damp;
        p.vx = 0;
        p.vy = 0;
      });
    }
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    ids.forEach((id) => {
      const p = pos[id];
      minX = Math.min(minX, p.x);
      minY = Math.min(minY, p.y);
      maxX = Math.max(maxX, p.x);
      maxY = Math.max(maxY, p.y);
    });
    const spanX = maxX - minX, spanY = maxY - minY;
    const target = Math.max(320, Math.sqrt(n) * 70);
    const k = target / Math.max(1, Math.max(spanX, spanY));
    const cx = (minX + maxX) / 2, cy = (minY + maxY) / 2;
    ids.forEach((id) => {
      const p = pos[id];
      p.x = (p.x - cx) * k;
      p.y = (p.y - cy) * k;
    });
    return pos;
  }
  function ensureForce(ids) {
    const key = [...ids].sort().join("|");
    if (forceKey !== key) {
      setForceLayout(computeForce(ids));
      setForceKey(key);
    }
  }
  function initLayout() {
    const order = [];
    const seen = new Set;
    NODES.forEach((n) => {
      const g = n.scope + "\x00" + n.type;
      if (!seen.has(g)) {
        seen.add(g);
        order.push(g);
      }
    });
    order.sort();
    const cols = Math.ceil(Math.sqrt(order.length));
    const colW = 150 * Math.sqrt(NODES.length / Math.max(1, order.length));
    const positions2 = {};
    order.forEach((g, gi) => {
      const [scope, type] = g.split("\x00");
      const members = NODES.filter((n) => n.scope === scope && n.type === type).sort((a, b) => parseInt(a.id.replace(/\D/g, ""), 10) - parseInt(b.id.replace(/\D/g, ""), 10));
      const perRow = Math.max(6, Math.ceil(Math.sqrt(members.length * 2)));
      const bx = gi % cols * (colW + 90);
      const by = Math.floor(gi / cols) * (120 + Math.ceil(members.length / perRow) * 36);
      members.forEach((n, i) => {
        positions2[n.id] = { x: bx + i % perRow * 26, y: by + Math.floor(i / perRow) * 34 };
      });
    });
    setPositions(positions2);
  }
  function buildGenealogy() {
    const decisionIds = NODES.filter((n) => n.type === "decision").map((n) => n.id);
    const dSet = new Set(decisionIds);
    const geneEdges = EDGES.filter((e) => e.label === "narrows" || e.label === "supersedes" || e.label === "completes" || e.label === "widens");
    const depends = {};
    geneEdges.forEach((e) => {
      (depends[e.source] = depends[e.source] || []).push(e.target);
    });
    const depth = {};
    function computeDepth(id, seen) {
      if (depth[id] !== undefined)
        return depth[id];
      if (seen.has(id))
        return 0;
      const deps = (depends[id] || []).filter((t) => dSet.has(t));
      let d = 0;
      deps.forEach((t) => {
        d = Math.max(d, 1 + computeDepth(t, new Set(seen).add(id)));
      });
      depth[id] = d;
      return d;
    }
    decisionIds.forEach((id) => computeDepth(id, new Set));
    const layers = {};
    decisionIds.forEach((id) => {
      const d = depth[id] === undefined ? 0 : depth[id];
      (layers[d] = layers[d] || []).push(id);
    });
    let maxW = 0;
    Object.keys(layers).forEach((k) => maxW = Math.max(maxW, layers[k].length));
    const layout = {};
    const spacing = 90, layerGap = 190;
    Object.keys(layers).sort((a, b) => Number(a) - Number(b)).forEach((k, li) => {
      const ids = layers[k].sort((a, b) => a.localeCompare(b));
      ids.forEach((id, j) => {
        layout[id] = { x: li * layerGap, y: (j - (ids.length - 1) / 2) * 64 };
      });
    });
    setGenealogyLayout(layout);
  }

  // src/filters/query.ts
  function haystack(n) {
    let parts = [n.id, n.title, n.type, n.scope, n.status || ""];
    for (const [k, v] of Object.entries(n.attrs))
      parts.push(k + "=" + String(v));
    parts.push(n.body || "");
    return parts.join(`
`).toLowerCase();
  }
  function computeMatches(q) {
    if (!q)
      return null;
    const lq = q.toLowerCase();
    return new Set(NODES.filter((n) => haystack(n).includes(lq)).map((n) => n.id));
  }
  var nextIds = new Set((D.next || []).map((n) => n.id));
  function setVisible(n) {
    if (overviewSource === "next" && (n.type !== "need" || !nextIds.has(n.id)))
      return false;
    if (typeState[n.type] !== true)
      return false;
    if (filterState.scopeSel && n.scope !== filterState.scopeSel)
      return false;
    if (n.type === "requirement") {
      if (filterState.stateSel && (n.state || "") !== filterState.stateSel)
        return false;
    }
    if (n.type === "question") {
      const qs = filterState.qstatus;
      if (qs && (n.status === "closed" ? "closed" : "open") !== qs)
        return false;
    }
    if (n.type === "criterion") {
      const c = filterState.criterion;
      if (c && String(!!n.attrs.satisfied) !== (c === "yes" ? "true" : "false"))
        return false;
    }
    return true;
  }
  var previousKey = "";
  var matches = [];
  function filteredNodes() {
    const key = JSON.stringify([overviewSource, typeState, filterState, searchText]);
    if (key !== previousKey) {
      const hits = computeMatches(searchText);
      setSearchHits(hits);
      matches = NODES.filter((n) => setVisible(n) && (!hits || hits.has(n.id)));
      previousKey = key;
    }
    return matches;
  }

  // src/graph/selection.ts
  function candidateNodes() {
    let ids = filteredNodes().map((n) => n.id);
    if (genealogyMode) {
      ids = ids.filter((id) => byId[id] && byId[id].type === "decision");
    }
    const focus = currentFocus();
    if (focus.id) {
      const reach = reachable(focus.id, focus.radius);
      ids = ids.filter((id) => reach.has(id));
    }
    return ids;
  }
  function visibleEdges(ids) {
    const s = new Set(ids);
    return EDGES.filter((e) => e.source && s.has(e.source) && e.target && s.has(e.target));
  }
  function lodFromScale() {
    return scale < 0.35 ? "far" : scale < 1.1 ? "mid" : "near";
  }
  var selectionKey = "";
  var selectionCache = null;
  function focusedSelection() {
    const candidates = candidateNodes();
    const focus = currentFocus();
    if (!focus.id || candidates.length <= MODE_THRESHOLD)
      return { ids: candidates, omitted: [] };
    const key = JSON.stringify([focus.id, focus.radius, candidates]);
    if (key === selectionKey)
      return selectionCache;
    const distance = new Map([[focus.id, 0]]), queue = [focus.id];
    for (let i = 0;i < queue.length; i++) {
      const id = queue[i], d = distance.get(id);
      if (d >= focus.radius)
        continue;
      for (const neighbor of Object.keys(adj[id] || {})) {
        if (!distance.has(neighbor)) {
          distance.set(neighbor, d + 1);
          queue.push(neighbor);
        }
      }
    }
    const ranked = [...candidates].sort((a, b) => distance.get(a) - distance.get(b) || degree[b] - degree[a] || (a < b ? -1 : a > b ? 1 : 0));
    selectionKey = key;
    selectionCache = { ids: ranked.slice(0, MODE_THRESHOLD), omitted: ranked.slice(MODE_THRESHOLD) };
    return selectionCache;
  }
  function visibleNodes() {
    return focusedSelection().ids;
  }

  // src/tokens.ts
  var values = getComputedStyle(document.documentElement);
  function token(name) {
    return values.getPropertyValue(name).trim();
  }
  var KIND_COLORS = { need: token("--need"), question: token("--question"), decision: token("--decision"), requirement: token("--requirement"), criterion: token("--criterion"), gate: token("--gate") };

  // src/graph/svg.ts
  var svg = element("svg");
  var NS = "http:" + "//www.w3.org/2000/svg";
  var root = document.createElementNS(NS, "g");
  function initSvg() {
    svg.appendChild(root);
  }
  var defs = null;
  function makeDefs() {
    defs = document.createElementNS(NS, "defs");
    svg.insertBefore(defs, root);
    const labels = [...new Set(EDGES.map((e) => e.label))];
    labels.forEach((l) => {
      const mk = document.createElementNS(NS, "marker");
      mk.setAttribute("id", String("arr-" + l.replace(/\W/g, "_")));
      mk.setAttribute("viewBox", "0 -4 8 8");
      mk.setAttribute("refX", "9");
      mk.setAttribute("refY", "0");
      mk.setAttribute("markerWidth", "7");
      mk.setAttribute("markerHeight", "7");
      mk.setAttribute("orient", "auto");
      const p = document.createElementNS(NS, "path");
      p.setAttribute("d", "M0,-4L8,0L0,4");
      p.setAttribute("fill", String(edgeColor(l)));
      mk.appendChild(p);
      defs.appendChild(mk);
    });
    const agg = document.createElementNS(NS, "marker");
    agg.setAttribute("id", "arr-_agg");
    agg.setAttribute("viewBox", "0 -4 8 8");
    agg.setAttribute("refX", "9");
    agg.setAttribute("refY", "0");
    agg.setAttribute("markerWidth", "7");
    agg.setAttribute("markerHeight", "7");
    agg.setAttribute("orient", "auto");
    const pa = document.createElementNS(NS, "path");
    pa.setAttribute("d", "M0,-4L8,0L0,4");
    pa.setAttribute("fill", String(token("--color-888")));
    agg.appendChild(pa);
    defs.appendChild(agg);
  }
  function edgeColor(label) {
    const map = { closes: token("--color-a03b3b"), narrows: token("--decision"), widens: token("--color-3f8f6b"), supersedes: token("--color-8a2b2b"), completes: token("--ok"), targets: token("--criterion"), "spawned-by": token("--gate"), "filed-as": token("--color-7d6c5f"), "depends-on": token("--need"), "relies-on": token("--color-5a5a6e"), raised: token("--color-a8608a"), "measured-by": token("--color-556b7a") };
    return map[label] || token("--color-666");
  }
  function shapeOf(type) {
    const s = {
      need: "M0,-7 C4,-7 7,-4 7,0 C7,4 4,7 0,7 C-4,7 -7,4 -7,0 C-7,-4 -4,-7 0,-7 Z",
      question: "M0,-9 L2.5,-2 L9,0 L2.5,2 L0,9 L-2.5,2 L-9,0 L-2.5,-2 Z",
      decision: "M-7,-7 L7,-7 L7,7 L-7,7 Z",
      requirement: "M-7,-4 L0,-7 L7,-4 L7,4 L0,7 L-7,4 Z",
      criterion: "M0,-7 L7,5 L-7,5 Z",
      gate: "M0,-8 L7.6,-2.5 L4.7,6.5 L-4.7,6.5 L-7.6,-2.5 Z"
    };
    return s[type] || s.decision;
  }

  // src/graph/viewport.ts
  function initViewport() {
    svg.addEventListener("wheel", (e) => {
      e.preventDefault();
      const factor = e.deltaY < 0 ? 1.12 : 1 / 1.12;
      const rect = svg.getBoundingClientRect();
      const cx = e.clientX - rect.left, cy = e.clientY - rect.top;
      const nx = (cx - translate.x) / scale, ny = (cy - translate.y) / scale;
      setViewSource("manual");
      setScale(scale * factor);
      setScale(Math.min(4, Math.max(0.1, scale)));
      setTranslate({ ...translate, x: cx - nx * scale });
      setTranslate({ ...translate, y: cy - ny * scale });
      applyTransform();
      updateLod();
      draw();
    }, { passive: false });
    svg.addEventListener("mousedown", (e) => {
      if (e.button !== 0)
        return;
      setDragging(true);
      setDragMoved(false);
      setDragStart({ x: e.clientX, y: e.clientY, tx: translate.x, ty: translate.y });
    });
    window.addEventListener("mousemove", (e) => {
      if (!dragging)
        return;
      if (!dragMoved && Math.hypot(e.clientX - dragStart.x, e.clientY - dragStart.y) < 4)
        return;
      setDragMoved(true);
      setViewSource("manual");
      setTranslate({ ...translate, x: dragStart.tx + e.clientX - dragStart.x });
      setTranslate({ ...translate, y: dragStart.ty + e.clientY - dragStart.y });
      applyTransform();
      updateLod();
      draw();
    });
    window.addEventListener("mouseup", () => {
      setDragging(false);
    });
  }
  function applyTransform() {
    root.setAttribute("transform", String("translate(" + translate.x + "," + translate.y + ") scale(" + scale + ")"));
  }
  function updateLod() {
    const next = lodFromScale();
    if (next !== lod) {
      setLod(next);
    }
  }
  function zoomBy(f) {
    const rect = svg.getBoundingClientRect();
    const cx = rect.width / 2, cy = rect.height / 2;
    const nx = (cx - translate.x) / scale, ny = (cy - translate.y) / scale;
    setViewSource("manual");
    setScale(scale * f);
    setScale(Math.min(4, Math.max(0.1, scale)));
    setTranslate({ ...translate, x: cx - nx * scale });
    setTranslate({ ...translate, y: cy - ny * scale });
    applyTransform();
    updateLod();
    draw();
  }
  function currentLayout() {
    const ids = visibleNodes();
    if (genealogyMode)
      return genealogyLayout;
    if (ids.length <= MODE_THRESHOLD) {
      ensureForce(ids);
      return forceLayout;
    }
    return positions;
  }
  function fitTransform() {
    const ids = visibleNodes();
    const over = !genealogyMode && ids.length > MODE_THRESHOLD;
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    if (over) {
      const { cluster, cells } = overviewGrid(ids);
      for (const g of Object.keys(cluster)) {
        const gp = cells[g] || { bx: 0, by: 0 };
        const w = Math.min(280, Math.max(120, 40 + cluster[g].length * 1.4));
        minX = Math.min(minX, gp.bx - w / 2);
        minY = Math.min(minY, gp.by - 20);
        maxX = Math.max(maxX, gp.bx + w / 2);
        maxY = Math.max(maxY, gp.by + 20);
      }
    } else {
      const layout = currentLayout();
      const ids2 = ids.filter((id) => layout[id]);
      const list = ids2.length ? ids2 : Object.keys(layout);
      list.forEach((id) => {
        const p = layout[id];
        if (!p)
          return;
        minX = Math.min(minX, p.x);
        minY = Math.min(minY, p.y);
        maxX = Math.max(maxX, p.x);
        maxY = Math.max(maxY, p.y);
      });
    }
    setViewSource("fit");
    if (minX > maxX)
      return;
    const rect = svg.getBoundingClientRect();
    const spanX = maxX - minX + 120;
    const spanY = maxY - minY + 120;
    const fit = Math.min(rect.width / Math.max(1, spanX), rect.height / Math.max(1, spanY));
    setScale(Math.min(over ? 4 : 2, fit));
    if (!over)
      setScale(Math.max(scale, 1));
    let center = { x: (minX + maxX) / 2, y: (minY + maxY) / 2 };
    const focus = currentFocus();
    if (!over && fit < 1 && focus.id && ids.includes(focus.id)) {
      center = currentLayout()[focus.id] || center;
    }
    setTranslate({ ...translate, x: rect.width / 2 - center.x * scale });
    setTranslate({ ...translate, y: rect.height / 2 - center.y * scale });
    applyTransform();
    updateLod();
  }
  function fitView() {
    setLastFitKey("");
    draw();
  }
  function resetView() {
    fitView();
  }

  // src/filters.ts
  var typeChips = element("typeChips");
  var scopeSel = element("scopeSel");
  var stateSel = element("stateSel");
  function initFilters() {
    ["all", "need", "question", "decision", "requirement", "criterion", "gate"].forEach((k) => {
      const chip = document.createElement("button");
      chip.type = "button";
      chip.setAttribute("aria-pressed", "true");
      chip.className = "chip on";
      chip.dataset.kind = k;
      chip.style.borderColor = KIND_COLORS[k] || "var(--border)";
      chip.textContent = k === "all" ? "All types" : k;
      chip.addEventListener("click", () => {
        setOverviewSource("");
        renderLint();
        selectType(k);
        renderTypeChips();
        redraw();
      });
      typeChips.appendChild(chip);
    });
    (D.scopes || []).forEach((s) => {
      const o = document.createElement("option");
      o.value = s;
      o.textContent = s;
      scopeSel.appendChild(o);
    });
    Object.keys(states).forEach((st) => {
      const o = document.createElement("option");
      o.value = st;
      o.textContent = st;
      stateSel.appendChild(o);
    });
    ["scopeSel", "stateSel", "qstatus", "criterion"].forEach((id) => element(id).addEventListener("change", () => {
      setFilter(id, element(id).value);
      redraw();
    }));
    element("q").addEventListener("input", () => {
      setSearchText(element("q").value.trim());
      redraw();
    });
    element("clearFilter").addEventListener("click", () => {
      Object.keys(typeState).forEach((k) => {
        setType(k, true);
      });
      document.querySelectorAll("#typeChips .chip").forEach((c) => {
        c.classList.add("on");
        c.setAttribute("aria-pressed", "true");
      });
      scopeSel.value = "";
      stateSel.value = "";
      element("qstatus").value = "";
      element("criterion").value = "";
      element("q").value = "";
      setSearchText("");
      setOverviewSource("");
      renderLint();
      ["scopeSel", "stateSel", "qstatus", "criterion"].forEach((id) => setFilter(id, ""));
      setRadiusChoice("");
      setListSort(null);
      renderTypeChips();
      truncateFocus(1);
      hideDetail();
      setGenealogyMode(false);
      element("hopRadius").value = "";
      element("hopFrom").value = "";
      redraw();
    });
    element("hopFrom").addEventListener("input", () => {
      element("applyHop").textContent = focusLabel(element("hopFrom").value.trim());
    });
    element("hopRadius").addEventListener("change", () => {
      setRadiusChoice(element("hopRadius").value);
      element("applyHop").textContent = focusLabel(element("hopFrom").value.trim());
    });
    element("applyHop").addEventListener("click", () => {
      const id = element("hopFrom").value.trim();
      if (!byId[id]) {
        alert("Unknown node id: " + id);
        return;
      }
      startHop(id);
    });
    element("genealogy").addEventListener("click", () => {
      setGenealogyMode(!genealogyMode);
      if (genealogyMode) {
        setLod("near");
      }
      redraw();
    });
    element("zin").addEventListener("click", () => zoomBy(1.6));
    element("zout").addEventListener("click", () => zoomBy(1 / 1.6));
    element("zfit").addEventListener("click", resetView);
  }
  function gotoTab(t) {
    setActiveTab(t);
    document.querySelectorAll("#tabs button").forEach((b) => b.classList.toggle("on", b.dataset.tab === t));
    document.querySelectorAll(".page").forEach((p) => p.classList.toggle("on", p.id === "page-" + t));
  }
  function renderTypeChips() {
    const all = Object.values(typeState).every(Boolean);
    document.querySelectorAll("#typeChips .chip").forEach((chip) => {
      const on = chip.dataset.kind === "all" ? all : !all && typeState[chip.dataset.kind];
      chip.classList.toggle("on", on);
      chip.setAttribute("aria-pressed", String(on));
    });
  }

  // src/navigation.ts
  function startHop(id) {
    if (!byId[id])
      return;
    const radius = focusPlan(id).n;
    if (currentFocus().id !== id || currentFocus().radius !== radius) {
      enterFocus(id, radius);
    }
    showDetail(id);
    redraw();
  }
  function returnFocus(index) {
    if (!Number.isInteger(index) || index < 0 || index >= focusHistory.length)
      return;
    truncateFocus(index + 1);
    if (currentFocus().id)
      showDetail(currentFocus().id);
    else
      hideDetail();
    redraw();
  }
  function renderNavigation(ids) {
    const focus = currentFocus();
    const path = element("focusPath");
    const pathKey = JSON.stringify(focusHistory);
    if (path.dataset.path !== pathKey) {
      path.dataset.path = pathKey;
      path.replaceChildren();
      focusHistory.forEach((entry, index) => {
        if (index)
          path.appendChild(document.createTextNode(" → "));
        const button = document.createElement("button");
        button.textContent = entry.id ? entry.id : "All nodes";
        button.title = entry.id ? entry.id + " · " + byId[entry.id].title : "All nodes";
        button.dataset.depth = String(index);
        if (index === focusHistory.length - 1)
          button.setAttribute("aria-current", "location");
        button.addEventListener("click", () => returnFocus(index));
        path.appendChild(button);
      });
      path.lastElementChild.scrollIntoView({ block: "nearest", inline: "nearest" });
    }
    element("focusBack").disabled = focusHistory.length === 1;
    element("focusAll").disabled = focusHistory.length === 1;
    element("focusDetail").disabled = !focus.id;
    element("focusNote").textContent = focus.id ? " · " + focus.radius + (focus.radius === 1 ? " hop" : " hops") + (Object.keys(adj[focus.id] || {}).length ? "" : " · No connections in this graph") + (ids.includes(focus.id) ? "" : " · Focus hidden by current filters, search, or lineage") : "";
    const active = [
      overviewSource ? "Source: " + overviewSource : "",
      Object.values(typeState).every(Boolean) ? "" : Object.keys(typeState).filter((k) => typeState[k]).join(", ") || "No types",
      scopeSel.value ? "Scope: " + scopeSel.value : "",
      stateSel.value ? "State: " + stateSel.value : "",
      element("qstatus").value ? "Questions: " + element("qstatus").value : "",
      element("criterion").value ? "Criteria: " + element("criterion").value : "",
      searchText ? "Search: " + searchText : ""
    ].filter(Boolean);
    element("displayStatus").textContent = active.length ? active.length + (active.length === 1 ? " filter" : " filters") : "No filters";
    element("displayStatus").title = active.join(" · ");
    const input = element("hopFrom"), focusKey = JSON.stringify(focus);
    if (input.dataset.focus !== focusKey) {
      input.dataset.focus = focusKey;
      input.value = selected || focus.id || "";
    }
    element("applyHop").textContent = focusLabel(input.value.trim());
    element("genealogy").textContent = genealogyMode ? "Lineage: on" : "Lineage: off";
    element("genealogy").setAttribute("aria-pressed", String(genealogyMode));
    element("genealogy").style.borderColor = genealogyMode ? "var(--hl)" : "";
  }
  function initNavigation() {
    element("focusBack").addEventListener("click", () => returnFocus(focusHistory.length - 2));
    element("focusAll").addEventListener("click", () => returnFocus(0));
    element("focusDetail").addEventListener("click", () => {
      if (currentFocus().id) {
        showDetail(currentFocus().id);
        redraw();
      }
    });
  }

  // src/graph/clusters.ts
  function openClusterList(group) {
    setOverviewSource("");
    renderLint();
    const [scope, type] = group.split("\x00");
    selectType(type);
    setFilter("scopeSel", scope);
    element("scopeSel").value = scope;
    renderTypeChips();
    redraw();
    element("listPane").scrollTop = 0;
  }
  function revealOmitted(members) {
    if (members.length)
      revealRow(members[0]);
  }

  // src/graph/draw.ts
  function overviewGrid(ids) {
    const cluster = {};
    ids.forEach((id) => {
      const n2 = byId[id];
      const g = n2.scope + "\x00" + n2.type;
      (cluster[g] = cluster[g] || []).push(id);
    });
    const gs = Object.keys(cluster);
    const cells = {};
    if (!gs.length)
      return { cluster, cells };
    const cellW = 320, cellH = 150;
    const vp = svg.getBoundingClientRect();
    const aspect = Math.max(0.2, Math.min(5, (vp.width || 800) / (vp.height || 600)));
    const n = gs.length;
    let cols = n, rows = 1, best = Infinity;
    for (let c = n > 1 ? 2 : 1;c <= n; c++) {
      const r = Math.ceil(n / c);
      const score = Math.abs(c * cellW / (r * cellH) - aspect);
      if (score < best) {
        best = score;
        cols = c;
        rows = r;
      }
    }
    const gridW = cols * cellW, gridH = rows * cellH;
    gs.forEach((g, i) => {
      const c = i % cols, r = Math.floor(i / cols);
      cells[g] = { bx: c * cellW + cellW / 2 - gridW / 2, by: r * cellH + cellH / 2 - gridH / 2 };
    });
    return { cluster, cells };
  }
  function dotPath(sx, sy, tx, ty) {
    const dx = tx - sx, dy = ty - sy, d = Math.max(1, Math.hypot(dx, dy));
    return { x1: sx, y1: sy, x2: tx - dx / d * 16, y2: ty - dy / d * 16 };
  }
  function draw() {
    const { ids, omitted } = focusedSelection();
    renderList();
    renderNavigation(ids);
    const viewport = svg.getBoundingClientRect();
    const focus = currentFocus();
    const key = JSON.stringify([genealogyMode, focus.id, focus.radius, [...ids].sort()]);
    if (key !== lastFitKey) {
      setLastFitKey(key);
      fitTransform();
    } else if (lastViewport && (viewport.width !== lastViewport.width || viewport.height !== lastViewport.height)) {
      if (viewSource === "fit")
        fitTransform();
      else {
        setTranslate({ ...translate, x: translate.x + (viewport.width - lastViewport.width) / 2 });
        setTranslate({ ...translate, y: translate.y + (viewport.height - lastViewport.height) / 2 });
        applyTransform();
      }
    }
    setLastViewport({ width: viewport.width, height: viewport.height });
    const showingAll = ids.length <= MODE_THRESHOLD || genealogyMode;
    if (!genealogyMode && showingAll && ids.length > 0)
      ensureForce(ids);
    const inPositions = genealogyMode ? genealogyLayout : showingAll ? forceLayout : positions;
    root.innerHTML = "";
    const culling = element("culling");
    const setMeta = (drawn, visCount, extra) => {
      const overview = !showingAll;
      element("viewCounts").textContent = overview ? "Graph: " + drawn + (drawn === 1 ? " cluster" : " clusters") : "Graph: " + drawn + " shown";
      culling.replaceChildren();
      const off2 = overview ? 0 : Number(visCount) - drawn;
      const hidden = off2 + omitted.length;
      if (!hidden)
        culling.textContent = "0 hidden";
      if (off2)
        culling.appendChild(document.createTextNode(off2 + " off-screen at readable zoom"));
      if (omitted.length) {
        if (off2)
          culling.appendChild(document.createTextNode(" · "));
        const button = document.createElement("button");
        button.id = "showOmitted";
        button.textContent = omitted.length + " nodes omitted";
        button.addEventListener("click", () => revealOmitted(omitted));
        culling.appendChild(button);
      }
    };
    if (genealogyMode) {
      drawEdgesLayer(inPositions, ids.filter((id) => inPositions[id]), visibleEdges(ids).filter((e) => (e.label === "narrows" || e.label === "supersedes" || e.label === "completes" || e.label === "widens") && inPositions[e.source] && inPositions[e.target]));
      const n2 = drawNodeLayer(inPositions, ids);
      const off2 = offscreenCount(inPositions, ids);
      setMeta(n2 - off2, n2, off2 > 0 ? "Showing " + (n2 - off2) + " of " + n2 + " nodes; " + off2 + " off-screen at readable zoom. Zoom out or pan to reach them." : "");
      return;
    }
    if (!showingAll) {
      const { cluster, cells } = overviewGrid(ids);
      const cg = Object.keys(cluster);
      const pair = {};
      EDGES.forEach((e) => {
        if (!ids.includes(e.source) || !ids.includes(e.target))
          return;
        const gs = byId[e.source].scope + "\x00" + byId[e.source].type;
        const gt = byId[e.target].scope + "\x00" + byId[e.target].type;
        if (gs === gt)
          return;
        const k = gs + "|" + gt;
        (pair[k] = pair[k] || []).push(e.label);
      });
      const edgeLayer = document.createElementNS(NS, "g");
      const pairKeys = Object.keys(pair);
      pairKeys.forEach((k, i) => {
        const labels = pair[k];
        const [gs, gt] = k.split("|");
        const a = cells[gs] || { bx: 0, by: 0 };
        const b = cells[gt] || { bx: 0, by: 0 };
        const { bx: sx, by: sy } = a, tx = b.bx, ty = b.by;
        const dx = tx - sx, dy = ty - sy, len = Math.max(1, Math.hypot(dx, dy));
        const ux = -dy / len, uy = dx / len;
        const side = i % 2 === 0 ? 1 : -1;
        const dist = 90 + Math.floor(i / 2) * 28;
        const mx = (sx + tx) / 2 + ux * dist * side;
        const my = (sy + ty) / 2 + uy * dist * side;
        const endX = tx - dx / len * 16, endY = ty - dy / len * 16;
        const path = document.createElementNS(NS, "path");
        path.setAttribute("d", String("M " + sx + " " + sy + " Q " + mx + " " + my + " " + endX + " " + endY));
        path.setAttribute("fill", "none");
        path.setAttribute("stroke", String(token("--color-888")));
        path.setAttribute("stroke-width", "1.4");
        path.setAttribute("marker-end", "url(#arr-_agg)");
        path.setAttribute("opacity", "0.75");
        path.setAttribute("data-label", String([...new Set(labels)].join(",")));
        path.setAttribute("data-s", String(gs));
        path.setAttribute("data-t", String(gt));
        edgeLayer.appendChild(path);
        const t = document.createElementNS(NS, "text");
        t.setAttribute("x", String(mx + ux * 14 * side));
        t.setAttribute("y", String(my + uy * 14 * side - 4));
        t.setAttribute("text-anchor", "middle");
        t.setAttribute("class", "elabel");
        t.setAttribute("data-count", String(labels.length));
        t.setAttribute("data-s", String(gs));
        t.setAttribute("data-t", String(gt));
        t.textContent = labels.length > 1 ? labels.length + "" : labels[0];
        edgeLayer.appendChild(t);
      });
      root.appendChild(edgeLayer);
      const internal = {};
      EDGES.forEach((e) => {
        if (!ids.includes(e.source) || !ids.includes(e.target))
          return;
        const g = byId[e.source].scope + "\x00" + byId[e.source].type;
        if (g !== byId[e.target].scope + "\x00" + byId[e.target].type)
          return;
        (internal[g] = internal[g] || []).push(e.label);
      });
      cg.forEach((g) => {
        const [scope, type] = g.split("\x00");
        const gp = cells[g] || { bx: 0, by: 0 };
        const members = cluster[g];
        const w = Math.min(280, Math.max(120, 40 + members.length * 1.4)), h = 34;
        const rect = document.createElementNS(NS, "rect");
        rect.setAttribute("x", String(gp.bx - w / 2));
        rect.setAttribute("y", String(gp.by - h / 2));
        rect.setAttribute("width", String(w));
        rect.setAttribute("height", String(h));
        rect.setAttribute("rx", "8");
        rect.setAttribute("fill", String(KIND_COLORS[type]));
        rect.setAttribute("opacity", "0.6");
        rect.setAttribute("class", "node");
        rect.setAttribute("role", "button");
        rect.setAttribute("tabindex", "0");
        rect.setAttribute("aria-label", "List " + scope + " / " + type + ": " + members.length + " records");
        rect.addEventListener("click", () => openClusterList(g));
        rect.addEventListener("keydown", (event) => {
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            openClusterList(g);
          }
        });
        root.appendChild(rect);
        const t1 = document.createElementNS(NS, "text");
        t1.setAttribute("x", String(gp.bx));
        t1.setAttribute("y", String(gp.by - 1));
        t1.setAttribute("text-anchor", "middle");
        t1.setAttribute("class", "nlabel");
        t1.style.fill = token("--color-111");
        t1.textContent = scope + " / " + type + " · " + members.length;
        root.appendChild(t1);
        const t2 = document.createElementNS(NS, "text");
        t2.setAttribute("x", String(gp.bx));
        t2.setAttribute("y", String(gp.by + 13));
        t2.setAttribute("text-anchor", "middle");
        t2.setAttribute("class", "elabel");
        t2.style.fill = token("--accent");
        t2.textContent = internal[g] ? "internal " + internal[g].length : "";
        root.appendChild(t2);
      });
      setMeta(cg.length, ids.length + " nodes", cg.length + " clusters · " + ids.length + " nodes available");
      return;
    }
    drawEdgesLayer(inPositions, ids, visibleEdges(ids));
    const n = drawNodeLayer(inPositions, ids);
    const off = offscreenCount(inPositions, ids);
    const noted = off > 0 ? "Showing " + (n - off) + " of " + n + " nodes; " + off + " off-screen at readable zoom. Zoom out or pan to reach them." : "";
    setMeta(n - off, n, noted);
  }
  function offscreenCount(layout, ids) {
    const rect = svg.getBoundingClientRect();
    let off = 0;
    ids.forEach((id) => {
      const p = layout[id];
      if (!p)
        return;
      const sx = p.x * scale + translate.x;
      const sy = p.y * scale + translate.y;
      if (sx < -20 || sx > rect.width + 20 || sy < -20 || sy > rect.height + 20)
        off++;
    });
    return off;
  }
  function drawEdgesLayer(inPositions, ids, drawEdges) {
    const edgeLayer = document.createElementNS(NS, "g");
    const wantLabels = !genealogyMode && ids.length <= EDGE_LABEL_MAX;
    drawEdges.forEach((e, idx) => {
      const a = inPositions[e.source], b = inPositions[e.target];
      if (!a || !b)
        return;
      const line = document.createElementNS(NS, "line");
      const p = dotPath(a.x, a.y, b.x, b.y);
      line.setAttribute("x1", String(p.x1));
      line.setAttribute("y1", String(p.y1));
      line.setAttribute("x2", String(p.x2));
      line.setAttribute("y2", String(p.y2));
      line.setAttribute("stroke", String(edgeColor(e.label)));
      line.setAttribute("stroke-width", "1.2");
      line.setAttribute("marker-end", String("url(#arr-" + e.label.replace(/\W/g, "_") + ")"));
      line.setAttribute("opacity", "0.5");
      line.setAttribute("data-lbl", String(e.label));
      line.setAttribute("data-idx", String(idx));
      edgeLayer.appendChild(line);
      if (wantLabels) {
        const t = document.createElementNS(NS, "text");
        t.setAttribute("x", String((p.x1 + p.x2) / 2));
        t.setAttribute("y", String((p.y1 + p.y2) / 2 - 3));
        t.setAttribute("text-anchor", "middle");
        t.setAttribute("class", "elabel");
        t.textContent = e.label;
        edgeLayer.appendChild(t);
      }
    });
    root.appendChild(edgeLayer);
  }
  function drawNodeLayer(inPositions, ids) {
    const nodeLayer = document.createElementNS(NS, "g");
    const placed = [];
    let drawn = 0;
    ids.forEach((id) => {
      const n = byId[id];
      const p = inPositions[id];
      if (!p)
        return;
      const isSelected = selected === id;
      const isHit = searchHits && searchHits.has(id);
      const g = document.createElementNS(NS, "g");
      g.setAttribute("transform", String("translate(" + p.x + "," + p.y + ")"));
      g.setAttribute("class", "node");
      if (isSelected) {
        const ring = document.createElementNS(NS, "circle");
        ring.setAttribute("r", "13");
        ring.setAttribute("class", "ring");
        ring.setAttribute("fill", "none");
        ring.setAttribute("stroke", "var(--hl)");
        ring.setAttribute("stroke-width", "3");
        g.appendChild(ring);
      }
      if (isHit) {
        const hit = document.createElementNS(NS, "circle");
        hit.setAttribute("r", "15");
        hit.setAttribute("fill", "none");
        hit.setAttribute("stroke", String(token("--color-ffb000")));
        hit.setAttribute("stroke-width", "2");
        g.appendChild(hit);
      }
      const r = lod === "near" ? 11 : 8;
      const path = document.createElementNS(NS, "path");
      path.setAttribute("d", String(shapeOf(n.type)));
      path.setAttribute("fill", String(isSelected ? token("--hl") : KIND_COLORS[n.type]));
      path.setAttribute("stroke", String(token("--color-222")));
      path.setAttribute("stroke-width", "0.8");
      path.setAttribute("transform", String("scale(" + r / 8 + ")"));
      g.appendChild(path);
      if (n.type === "requirement" && n.status === "complete") {
        const ck = document.createElementNS(NS, "text");
        ck.setAttribute("x", String(0));
        ck.setAttribute("y", String(3));
        ck.setAttribute("text-anchor", "middle");
        ck.style.fontSize = token("--font-9px");
        ck.style.fill = token("--color-fff");
        ck.textContent = "✓";
        g.appendChild(ck);
      }
      if (n.type === "decision" && n.superseded_by && n.superseded_by.length) {
        const x = document.createElementNS(NS, "text");
        x.setAttribute("x", String(0));
        x.setAttribute("y", String(3));
        x.setAttribute("text-anchor", "middle");
        x.style.fontSize = token("--font-9px");
        x.style.fill = token("--color-fff");
        x.textContent = "×";
        x.setAttribute("transform", "scale(0.8)");
        g.appendChild(x);
      }
      const probe = document.createElementNS(NS, "text");
      probe.setAttribute("text-anchor", "middle");
      probe.setAttribute("class", "nlabel");
      svg.appendChild(probe);
      const full = lod === "near" ? n.id + " · " + n.title + (n.status ? " · " + n.status : "") : n.id;
      probe.textContent = full;
      let tw = probe.getComputedTextLength() || full.length * 6;
      const budget = levelW();
      while (tw > budget && probe.textContent.length > 4) {
        probe.textContent = probe.textContent.slice(0, probe.textContent.length - 2) + "…";
        tw = probe.getComputedTextLength() || probe.textContent.length * 6;
      }
      const finalText = probe.textContent;
      svg.removeChild(probe);
      const pad = 6;
      const bb = { x: p.x - tw / 2, y: p.y + r + 3, w: tw + pad, h: 13 + pad };
      const collides = placed.some((q) => !(bb.x + bb.w < q.x || q.x + q.w < bb.x || bb.y + bb.h < q.y || q.y + q.h < bb.y));
      if (!collides && tw <= budget) {
        const t = document.createElementNS(NS, "text");
        t.setAttribute("x", String(0));
        t.setAttribute("y", String(r + 13));
        t.setAttribute("text-anchor", "middle");
        t.setAttribute("class", "nlabel");
        t.textContent = finalText;
        g.appendChild(t);
        placed.push(bb);
      }
      g.dataset.nodeId = id;
      g.setAttribute("tabindex", "0");
      g.setAttribute("role", "button");
      g.setAttribute("aria-label", String("Read " + id + " · " + n.title));
      g.addEventListener("click", (e) => {
        e.stopPropagation();
        if (!dragMoved)
          selectNode(id);
      });
      g.addEventListener("keydown", (e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          selectNode(id);
        }
      });
      nodeLayer.appendChild(g);
      drawn++;
    });
    root.appendChild(nodeLayer);
    return drawn;
  }
  function levelW() {
    return scale >= 1.1 ? MAX_LABEL_W : MAX_LABEL_W * 0.6;
  }

  // src/components.ts
  var esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
  function redraw() {
    draw();
  }

  // src/location.ts
  var lastLocationState = "";
  function locationState(value) {
    const fields = ["scopeSel", "stateSel", "qstatus", "criterion"];
    if (value === undefined) {
      return JSON.stringify({
        selected,
        focus: focusHistory,
        types: typeState,
        filters: filterState,
        overview: overviewSource,
        listSort,
        search: searchText,
        radiusChoice,
        tab: activeTab,
        genealogy: genealogyMode
      });
    }
    restoreFocus(value.focus);
    Object.keys(typeState).forEach((k) => {
      setType(k, value.types?.[k] !== false);
    });
    renderTypeChips();
    setListSort(value.listSort);
    setOverviewSource(value.overview);
    renderLint();
    fields.forEach((id) => {
      element(id).value = typeof value.filters?.[id] === "string" ? value.filters[id] : "";
      setFilter(id, element(id).value);
    });
    element("hopRadius").value = ["", "1", "2", "3", "4", "5"].includes(value.radiusChoice) ? value.radiusChoice : "";
    setRadiusChoice(element("hopRadius").value);
    setSearchText(typeof value.search === "string" ? value.search : "");
    element("q").value = searchText;
    setGenealogyMode(value.genealogy === true);
    const tabs = [...document.querySelectorAll("#tabs button")].map((b) => b.dataset.tab);
    gotoTab(tabs.includes(value.tab) ? value.tab : "overview");
    element("hopFrom").value = currentFocus().id || "";
    hideDetail();
    element("locationStatus").textContent = "";
    if (typeof value.selected === "string") {
      if (Object.hasOwn(byId, value.selected))
        showDetail(value.selected);
      else
        element("locationStatus").textContent = "Node not found: " + value.selected;
    }
  }
  function restoreLocation() {
    const state = locationHash();
    locationState(state);
    lastLocationState = locationState();
    resetView();
  }
  function saveLocation() {
    const state = locationState();
    if (state === lastLocationState)
      return;
    history.pushState(null, "", locationHash(state));
    lastLocationState = state;
    renderList();
    element("locationStatus").textContent = "";
  }

  // src/list.ts
  var columns = [["id", "ID"], ["type", "Type"], ["title", "Title"], ["scope", "Scope"], ["status", "State"], ["created", "Created"]];
  var compare = new Intl.Collator("en", { numeric: true, sensitivity: "base" });
  var renderedKey = "";
  var linkedState = "";
  function value(node, key) {
    if (key === "status") {
      if (node.type === "requirement")
        return node.state || node.status || "";
      if (node.type === "question")
        return node.status === "closed" ? "Closed" : "Open";
      if (node.type === "criterion")
        return node.attrs.satisfied ? "Satisfied" : "Not satisfied";
      return node.status || "";
    }
    return String(node[key] ?? "");
  }
  function initList() {
    element("recordList").querySelector("thead tr").innerHTML = columns.map(([key, label]) => '<th scope="col" data-column="' + key + '"><button data-sort="' + key + '" aria-label="Sort by ' + label + '">' + label + "</button></th>").join("");
    element("recordList").addEventListener("click", (event) => {
      const target = event.target;
      const sort = target.closest("[data-sort]");
      if (sort) {
        const key = sort.dataset.sort;
        setListSort({ key, direction: listSort.key === key && listSort.direction === "asc" ? "desc" : "asc" });
        redraw();
        return;
      }
      const link = target.closest("a[data-record]");
      if (link && event.button === 0 && !event.ctrlKey && !event.metaKey && !event.shiftKey && !event.altKey) {
        event.preventDefault();
        selectNode(link.dataset.record);
      }
    });
  }
  function renderList() {
    const nodes = filteredNodes();
    element("listCount").textContent = nodes.length + (nodes.length === 1 ? " result" : " results");
    const key = JSON.stringify([nodes.map((n) => n.id), listSort]);
    if (key !== renderedKey) {
      const rows = [...nodes].sort((a, b) => {
        const order = compare.compare(value(a, listSort.key), value(b, listSort.key));
        return order * (listSort.direction === "asc" ? 1 : -1) || compare.compare(a.id, b.id);
      });
      element("recordList").querySelector("tbody").innerHTML = rows.map((node) => '<tr data-record-id="' + esc(node.id) + '">' + columns.map(([column]) => '<td data-column="' + column + '">' + (column === "id" ? '<a data-record="' + esc(node.id) + '" href="#' + esc(encodeURIComponent(node.id)) + '">' + esc(node.id) + "</a>" : esc(value(node, column))) + "</td>").join("") + "</tr>").join("");
      element("listEmpty").hidden = nodes.length !== 0;
      renderedKey = key;
      linkedState = "";
    }
    element("recordList").querySelectorAll("th[data-column]").forEach((th) => {
      const direction = th.dataset.column === listSort.key ? listSort.direction === "asc" ? "ascending" : "descending" : "none";
      th.setAttribute("aria-sort", direction);
    });
    const snapshot = locationState();
    if (snapshot !== linkedState) {
      const view = JSON.parse(snapshot);
      element("recordList").querySelectorAll("a[data-record]").forEach((link) => {
        link.href = locationHash(JSON.stringify({ ...view, selected: link.dataset.record }));
        link.closest("tr").classList.toggle("selected", link.dataset.record === selected);
        if (link.dataset.record === selected)
          link.setAttribute("aria-current", "true");
        else
          link.removeAttribute("aria-current");
      });
      linkedState = snapshot;
    }
  }
  function revealRow(id) {
    const row = [...element("recordList").querySelectorAll("a[data-record]")].find((link) => link.dataset.record === id);
    if (row) {
      row.focus({ preventScroll: true });
      row.scrollIntoView({ block: "nearest", inline: "nearest" });
    }
  }

  // src/survey/overview.ts
  function initOverview() {
    element("metaGen").textContent = "Generated: " + D.generated_at;
    element("metaScope").textContent = "Scopes: " + (D.scope ? D.scope : "all (" + (D.scopes || []).join(", ") + ")");
    element("metaNodes").textContent = "Nodes: " + D.node_count;
    element("integrityNote").textContent = D.integrity_note;
    const lintErr = lintArr.filter((d) => d.severity === "error").length;
    const lintWarn = lintArr.filter((d) => d.severity === "warn").length;
    const openQs = D.open_questions || [];
    const waitRefs = openQs.reduce((a, q) => a + (q.referencing || []).length, 0);
    element("acSatisfied").textContent = D.criteria.satisfied;
    element("acTotal").textContent = D.criteria.total;
    element("openQ").textContent = openQs.length;
    element("waitRefs").textContent = waitRefs;
    element("nextCount").textContent = (D.next || []).length;
    element("lintErr").textContent = lintErr;
    element("lintWarn").textContent = lintWarn;
    function target(kind = "", filters = {}, overview = "", tab = "overview") {
      return locationHash(JSON.stringify({ types: Object.fromEntries(Object.keys(typeState).map((k) => [k, !kind || k === kind])), filters, overview, tab }));
    }
    function link(id, href) {
      const box = element(id).parentElement;
      const a = document.createElement("a");
      a.className = box.className;
      if (box.id)
        a.id = box.id;
      a.href = href;
      a.append(...box.childNodes);
      box.replaceWith(a);
    }
    link("acSatisfied", target("criterion", { criterion: "yes" }));
    link("acTotal", target("criterion"));
    link("openQ", target("question", { qstatus: "open" }));
    link("nextCount", target("need", {}, "next"));
    link("lintErr", target("", {}, "lint-error", "jams"));
    link("lintWarn", target("", {}, "lint-warn", "jams"));
    element("waitRefs").parentElement.classList.add("text-stat");
    const maxState = Math.max(1, ...Object.values(states));
    const stateColors = [token("--requirement"), token("--need"), token("--gate"), token("--decision"), token("--question"), token("--color-c97b4f"), token("--criterion"), token("--color-a8608a"), token("--color-7d6c5f"), token("--color-556b7a"), token("--color-3f8f6b")];
    let i = 0;
    const stateBar = element("stateBars");
    Object.entries(states).forEach(([st, cnt]) => {
      const w = Math.round(cnt / maxState * 100);
      const d = document.createElement("a");
      d.className = "bar";
      d.href = target("requirement", { stateSel: st });
      d.style.setProperty("--w", w + "%");
      d.style.setProperty("--bar", stateColors[i++ % stateColors.length]);
      d.title = st;
      d.innerHTML = esc(st) + " <em>" + cnt + "</em>";
      stateBar.appendChild(d);
    });
  }

  // src/survey/progress.ts
  function bars(elId, data, color) {
    const el = element(elId);
    const W = 400, H = 120, pad = 6;
    el.innerHTML = "";
    el.setAttribute("viewBox", "0 0 400 120");
    const max = Math.max(1, ...data.map((d) => d.value));
    data.forEach((d, i) => {
      const barW = (W - pad * 2) / data.length - 2;
      const h = d.value / max * (H - pad * 2);
      const r = document.createElementNS(NS, "rect");
      r.setAttribute("x", String(pad + i * (barW + 2)));
      r.setAttribute("y", String(H - pad - h));
      r.setAttribute("width", String(barW));
      r.setAttribute("height", String(h));
      r.setAttribute("fill", String(color));
      const txt = document.createElementNS(NS, "text");
      txt.setAttribute("x", String(pad + i * (barW + 2) + barW / 2));
      txt.setAttribute("y", String(H - pad - h - 3));
      txt.setAttribute("text-anchor", "middle");
      txt.setAttribute("font-size", token("--font-7px"));
      txt.textContent = d.label;
      el.appendChild(r);
      el.appendChild(txt);
    });
  }
  function initProgress() {
    const acData = D.criteria_timeline || [{ label: "today", value: D.criteria.satisfied || 0 }];
    bars("acChart", acData, token("--criterion"));
    element("acChartNote").textContent = D.criteria.satisfied + " / " + D.criteria.total + " acceptance criteria satisfied";
    if (D.arrival && D.arrival.available) {
      const qd = [{ label: "prev", value: Math.round((D.arrival.previous_per_day || 0) * 10) / 10 }, { label: "now", value: Math.round((D.arrival.current_per_day || 0) * 10) / 10 }];
      bars("qChart", qd, token("--gate"));
      element("arrivalNote").textContent = "new questions/day: " + D.arrival.current_per_day.toFixed(2) + " now, " + D.arrival.previous_per_day.toFixed(2) + " before; decay " + (D.arrival.decay_fraction == null ? "n/a" : (D.arrival.decay_fraction * 100).toFixed(0) + "%");
    } else {
      element("qChart").innerHTML = "";
      element("arrivalNote").textContent = "Question arrival is unavailable: no git history for this ledger.";
    }
    (function buildLegend() {
      const box = element("legendBox");
      let html = "<strong>Legend</strong>";
      Object.entries(KIND_COLORS).forEach(([k, c]) => {
        html += '<span class="legend-item"><span class="swatch ' + (k === "criterion" ? "shp-criterion" : "") + '" style="background:' + c + '"></span>' + k + "</span>";
      });
      const labels = [...new Set(EDGES.map((e) => e.label))];
      labels.forEach((l) => {
        html += '<div class="edge-row"><span class="eline" style="border-color:' + edgeColor(l) + '"></span>' + esc(l) + "</div>";
      });
      html += '<div class="edge-row"><span class="swatch shp-need" style="background:transparent;border:2px solid var(--color-ffb000)"></span>search hit</div>';
      html += '<div class="edge-row"><span class="swatch shp-need" style="background:var(--hl);border:1px solid var(--color-222)"></span>selected</div>';
      box.innerHTML = html;
    })();
  }

  // src/survey/index.ts
  function initSurvey() {
    initOverview();
    initBlockers();
    initProgress();
  }

  // src/main.ts
  initSvg();
  initFilters();
  initList();
  initNavigation();
  initDetail();
  initViewport();
  initSurvey();
  initLayout();
  buildGenealogy();
  makeDefs();
  restoreLocation();
  document.querySelectorAll("#tabs button").forEach((b) => b.addEventListener("click", () => gotoTab(b.dataset.tab)));
  window.addEventListener("hashchange", restoreLocation);
  ["click", "input", "change", "keydown"].forEach((event) => document.addEventListener(event, () => setTimeout(saveLocation, 0), true));
  window.addEventListener("resize", draw);
})();
