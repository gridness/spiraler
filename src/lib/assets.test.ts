import { describe, expect, test } from 'bun:test';
import { parseAssetList, selectRange, filterAssets, assetStatus } from './assets';
import type { Asset, Job } from './types';
describe('production asset lists', () => {
  test('parses names, em-dash descriptions, CRLF, and empty lines', () => {
    expect(parseAssetList('Potion — red glass\r\n\r\nShield - round oak\nTree')).toEqual([
      { name: 'Potion', subject: 'red glass' },
      { name: 'Shield', subject: 'round oak' },
      { name: 'Tree', subject: 'Tree' },
    ]);
  });
  test('preserves quoted CSV commas, escaped quotes, and multiline descriptions', () => {
    expect(parseAssetList('name,subject\n"Mug, small","Blue ""clay""\nwith a handle"')).toEqual([
      { name: 'Mug, small', subject: 'Blue "clay"\nwith a handle' },
    ]);
  });
  test('handles TSV with a header', () =>
    expect(parseAssetList('name\tsubject\nHat\tgreen felt')).toEqual([
      { name: 'Hat', subject: 'green felt' },
    ]));
  test('does not split commas inside ordinary prose', () =>
    expect(parseAssetList('A red, round potion')[0].name).toBe('A red, round potion'));
  test('rejects malformed CSV and oversized queues', () => {
    expect(() => parseAssetList('name,subject\n"unfinished,blue')).toThrow();
    expect(() => parseAssetList(Array(2001).fill('tree').join('\n'))).toThrow();
  });
});
describe('collection selection', () => {
  test('shift selection works in both directions and recovers from a filtered anchor', () => {
    expect(selectRange(['a', 'b', 'c'], 'c', 'a')).toEqual(['a', 'b', 'c']);
    expect(selectRange(['a', 'b'], 'missing', 'b')).toEqual(['b']);
  });
  test('a failed variant stays visible as failed without discarding the original', () => {
    const a: Asset = {
      id: 'a',
      name: 'Potion',
      subject: 'Red glass',
      approved: false,
      preferredId: null,
      results: [],
    };
    const jobs = [{ assetId: 'a', status: 'failed' }] as Job[];
    expect(assetStatus(a, jobs)).toBe('failed');
    expect(filterAssets([a], jobs, 'red', 'failed')).toEqual([a]);
    expect(filterAssets([a], jobs, 'sword', 'failed')).toEqual([]);
  });
});
