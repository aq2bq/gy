import { esc } from '../components';
export function inline(source) {
    const s = String(source), out = [];
    for (let i = 0; i < s.length;) {
        if (s[i] === '\\' && s[i + 1] && /[^\w\s]/.test(s[i + 1])) {
            out.push(esc(s[i + 1]));
            i += 2;
            continue;
        }
        if (s[i] === '<' && /^<(?:\/?[A-Za-z]|!--)/.test(s.slice(i))) {
            let end = i + 1, quote = '';
            for (; end < s.length; end++) {
                if (quote) {
                    if (s[end] === quote)
                        quote = '';
                }
                else if (s[end] === '\"' || s[end] === "'")
                    quote = s[end];
                else if (s[end] === '>')
                    break;
            }
            if (end < s.length) {
                out.push(esc(s.slice(i, end + 1)));
                i = end + 1;
                continue;
            }
        }
        if (s[i] === '`') {
            const ticks = s.slice(i).match(/^`+/)[0], end = s.indexOf(ticks, i + ticks.length);
            if (end >= 0) {
                let code = s.slice(i + ticks.length, end).replace(/\n/g, ' ');
                if (/^ .* $/.test(code) && code.trim())
                    code = code.slice(1, -1);
                out.push('<code>' + esc(code) + '</code>');
                i = end + ticks.length;
                continue;
            }
        }
        if (s[i] === '[') {
            const middle = s.indexOf('](', i + 1);
            if (middle >= 0) {
                let end = middle + 2, depth = 1;
                for (; end < s.length; end++) {
                    if (s[end] === '\\') {
                        end++;
                        continue;
                    }
                    if (s[end] === '(')
                        depth++;
                    if (s[end] === ')' && --depth === 0)
                        break;
                }
                if (depth === 0) {
                    const destination = s.slice(middle + 2, end).match(/^(?:<([^>]*)>|(\S+?))(?:\s+"([^"]*)")?$/);
                    if (destination) {
                        const href = (destination[1] ?? destination[2]).replace(/\\([()])/g, '$1');
                        let safe = false;
                        try {
                            safe = ['http:', 'https:', 'mailto:', 'file:'].includes(new URL(href, location.href).protocol) && !/[\u0000-\u0020]/.test(href);
                        }
                        catch (_) { }
                        const label = inline(s.slice(i + 1, middle));
                        out.push(safe ? '<a href="' + esc(href) + '" rel="noopener noreferrer"' + (destination[3] ? ' title="' + esc(destination[3]) + '"' : '') + '>' + label + '</a>' : label + ' (' + esc(href) + ')');
                        i = end + 1;
                        continue;
                    }
                }
            }
        }
        let matched = false;
        for (const [mark, open, close] of [['***', '<strong><em>', '</em></strong>'], ['___', '<strong><em>', '</em></strong>'], ['**', '<strong>', '</strong>'], ['__', '<strong>', '</strong>'], ['~~', '<del>', '</del>'], ['*', '<em>', '</em>'], ['_', '<em>', '</em>']]) {
            if (!s.startsWith(mark, i) || /\s/.test(s[i + mark.length] || ' ') || (mark[0] === '_' && /\w/.test(s[i - 1] || '')))
                continue;
            let end = s.indexOf(mark, i + mark.length);
            while (end >= 0 && s[end - 1] === '\\')
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
    return out.join('');
}
export function tableCells(line) {
    const cells = [], text = line.trim();
    let cell = '', ticks = 0;
    for (let i = 0; i < text.length; i++) {
        if (text[i] === '\\' && text[i + 1] === '|') {
            cell += '\\|';
            i++;
            continue;
        }
        if (text[i] === '`') {
            const run = text.slice(i).match(/^`+/)[0];
            ticks = ticks === run.length ? 0 : (ticks || run.length);
            cell += run;
            i += run.length - 1;
            continue;
        }
        if (text[i] === '|' && !ticks) {
            cells.push(cell.trim());
            cell = '';
        }
        else
            cell += text[i];
    }
    cells.push(cell.trim());
    if (text.startsWith('|'))
        cells.shift();
    if (text.endsWith('|') && cells[cells.length - 1] === '')
        cells.pop();
    return cells;
}
export function renderBody(md) {
    const lines = String(md || '').replace(/\r\n?/g, '\n').split('\n');
    const fence = line => line.match(/^ {0,3}(`{3,}|~{3,})(.*)$/);
    const indentLine = line => line.replace(/^[ \t]*/, prefix => { let width = 0; for (const c of prefix)
        width += c === '\t' ? 4 - width % 4 : 1; return ' '.repeat(width); });
    const item = line => indentLine(line).match(/^( *)([-+*]|\d+[.)]) +(.*)$/);
    const heading = line => line.match(/^ {0,3}(#{1,6})\s+(.*)$/);
    function blocks(lines) {
        const html = [];
        let i = 0;
        const isTable = at => at + 1 < lines.length && lines[at].includes('|') &&
            tableCells(lines[at]).length > 0 && tableCells(lines[at + 1]).length === tableCells(lines[at]).length &&
            tableCells(lines[at + 1]).every(c => /^:?-+:?$/.test(c));
        while (i < lines.length) {
            const line = lines[i];
            if (!line.trim()) {
                i++;
                continue;
            }
            const f = fence(line);
            if (f) {
                const code = [], close = new RegExp('^ {0,3}' + f[1][0] + '{' + f[1].length + ',}\\s*$');
                i++;
                while (i < lines.length && !close.test(lines[i]))
                    code.push(lines[i++]);
                if (i < lines.length)
                    i++;
                const language = f[2].trim().split(/\s/)[0];
                html.push('<pre><code' + (language ? ' class="language-' + esc(language) + '"' : '') + '>' + esc(code.join('\n') + (code.length ? '\n' : '')) + '</code></pre>');
                continue;
            }
            if (/^ {0,3}<(?:\/?[A-Za-z]|!--)/.test(line)) {
                const literal = [];
                while (i < lines.length && lines[i].trim())
                    literal.push(lines[i++]);
                html.push('<p>' + esc(literal.join('\n')) + '</p>');
                continue;
            }
            const h = heading(line);
            if (h) {
                html.push('<h4>' + inline(h[2].replace(/\s+#+\s*$/, '')) + '</h4>');
                i++;
                continue;
            }
            if (isTable(i)) {
                const headers = tableCells(line), align = tableCells(lines[i + 1]).map(c => c.endsWith(':') ? (c.startsWith(':') ? 'center' : 'right') : 'left');
                const row = (cells, tag) => '<tr>' + headers.map((_, n) => '<' + tag + ' style="text-align:' + align[n] + '">' + inline(cells[n] || '') + '</' + tag + '>').join('') + '</tr>';
                html.push('<table><thead>' + row(headers, 'th') + '</thead><tbody>');
                i += 2;
                while (i < lines.length && lines[i].trim() && lines[i].includes('|'))
                    html.push(row(tableCells(lines[i++]), 'td'));
                html.push('</tbody></table>');
                continue;
            }
            const first = item(line);
            if (first) {
                const indent = first[1].length, ordered = /^\d/.test(first[2]), tag = ordered ? 'ol' : 'ul';
                html.push('<' + tag + (ordered ? ' start="' + parseInt(first[2], 10) + '"' : '') + '>');
                while (i < lines.length) {
                    const entry = item(lines[i]);
                    if (!entry || entry[1].length !== indent || /^\d/.test(entry[2]) !== ordered)
                        break;
                    const contentIndent = indentLine(lines[i]).length - entry[3].length, content = [entry[3]];
                    i++;
                    while (i < lines.length) {
                        const next = indentLine(lines[i]), spaces = next.match(/^ */)[0].length;
                        if (!next.trim()) {
                            content.push('');
                            i++;
                            continue;
                        }
                        if (spaces > indent) {
                            content.push(next.slice(Math.min(contentIndent, spaces)));
                            i++;
                            continue;
                        }
                        if (item(next) || heading(next) || fence(next) || content[content.length - 1] === '')
                            break;
                        content.push(next);
                        i++;
                    }
                    html.push('<li>' + blocks(content) + '</li>');
                }
                html.push('</' + tag + '>');
                continue;
            }
            if (/^ {0,3}>/.test(line)) {
                const quoted = [];
                while (i < lines.length && /^ {0,3}>/.test(lines[i]))
                    quoted.push(lines[i++].replace(/^ {0,3}> ?/, ''));
                html.push('<blockquote>' + blocks(quoted) + '</blockquote>');
                continue;
            }
            const paragraph = [line];
            i++;
            while (i < lines.length && lines[i].trim() && !fence(lines[i]) && !heading(lines[i]) && !item(lines[i]) && !/^ {0,3}>/.test(lines[i]) && !isTable(i))
                paragraph.push(lines[i++]);
            html.push('<p>' + inline(paragraph.join('\n')) + '</p>');
        }
        return html.join('\n');
    }
    return blocks(lines);
}
// Presentation only: the embedded core projection remains the source of state.
