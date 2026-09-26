import type { Asset, AssetInput, Filter, Job } from './types';
export const preferred = (asset: Asset) =>
  asset.results.find((r) => r.id === asset.preferredId) ?? asset.results.at(-1);
export const assetStatus = (asset: Asset, jobs: Job[]) => {
  const active = jobs.find(
    (j) => j.assetId === asset.id && ['queued', 'generating'].includes(j.status),
  );
  if (active) return active.status;
  const latest = [...jobs].reverse().find((j) => j.assetId === asset.id);
  if (latest?.status === 'failed') return 'failed';
  if (preferred(asset)) return 'ready';
  return latest?.status === 'cancelled' ? 'cancelled' : 'draft';
};
export function filterAssets(
  assets: Asset[],
  jobs: Job[],
  search: string,
  filter: Filter,
): Asset[] {
  const needle = search.trim().toLocaleLowerCase();
  return assets.filter(
    (a) =>
      (!needle || `${a.name} ${a.subject}`.toLocaleLowerCase().includes(needle)) &&
      (filter === 'all' || (filter === 'approved' ? a.approved : assetStatus(a, jobs) === filter)),
  );
}
// RFC 4180 quoting, including newlines in quoted fields. Plain lists use name — subject.
export function parseAssetList(text: string): AssetInput[] {
  if (text.length > 1024 * 1024) throw new Error('Keep lists under 1 MB.');
  const firstLine = text.split(/\r?\n/)[0] ?? '';
  const delimiter = firstLine.includes('\t')
    ? '\t'
    : /^(name|title),/i.test(firstLine) || firstLine.startsWith('"')
      ? ','
      : null;
  let rows: string[][];
  if (delimiter) {
    rows = [];
    let row: string[] = [];
    let field = '';
    let quoted = false;
    for (let i = 0; i < text.length; i++) {
      const c = text[i];
      if (c === '"') {
        if (quoted && text[i + 1] === '"') {
          field += '"';
          i++;
        } else quoted = !quoted;
      } else if (!quoted && c === delimiter) {
        row.push(field);
        field = '';
      } else if (!quoted && c === '\n') {
        row.push(field.replace(/\r$/, ''));
        rows.push(row);
        row = [];
        field = '';
      } else field += c;
    }
    if (quoted) throw new Error('A quoted field is missing its closing quote.');
    row.push(field.replace(/\r$/, ''));
    rows.push(row);
    if (/^(name|title)$/i.test(rows[0]?.[0]?.trim() ?? '')) rows.shift();
  } else
    rows = text.split(/\r?\n/).map((line) => {
      const split = line.search(/\s[—–]\s|\s-\s/);
      return split < 0 ? [line] : [line.slice(0, split), line.slice(split + 3)];
    });
  const items = rows
    .filter((r) => r.some((v) => v.trim()))
    .map(([name, subject]) => ({ name: name.trim(), subject: subject?.trim() || name.trim() }));
  if (items.length > 2000) throw new Error('Use up to 2,000 assets per project.');
  if (items.some((i) => !i.name || i.name.length > 160 || i.subject.length > 8000))
    throw new Error(
      'Each asset needs a name under 160 characters and a description under 8,000 characters.',
    );
  return items;
}
export function selectRange(ids: string[], anchor: string, target: string): string[] {
  const start = ids.indexOf(anchor);
  const end = ids.indexOf(target);
  return start < 0 || end < 0
    ? [target]
    : ids.slice(Math.min(start, end), Math.max(start, end) + 1);
}
