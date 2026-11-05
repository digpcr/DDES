/** Per-well multichannel k-means. Labels are arbitrary, not positive/negative calls.
 * Uses all channels without scaling and does not modify the DDES data.
 */
import { readFileSync } from 'node:fs';
import { kmeans } from 'ml-kmeans';
import { DdesArchive, initSync } from '../pkg/ddes_typescript.js';
import type { IntensityFile } from '../pkg/ddes_typescript.js';

export function clusterIntensities(file: IntensityFile, count: number): [string, number][] {
  if (!Number.isInteger(count) || count < 1) {
    throw new Error('Cluster count must be a positive integer');
  }
  // Align channels by partition ID, not row position or numeric identifier value.
  const partitions = new Map<string, Map<string, number>>();
  const channels = new Set<string>();
  for (const row of file.records) {
    if (!Number.isFinite(row.fiEndpoint)) throw new Error('Intensity must be finite');
    let measurements = partitions.get(row.partitionId);
    if (!measurements) {
      measurements = new Map();
      partitions.set(row.partitionId, measurements);
    }
    if (measurements.has(row.channelId)) {
      throw new Error(`Duplicate measurement: ${row.partitionId}, ${row.channelId}`);
    }
    measurements.set(row.channelId, row.fiEndpoint);
    channels.add(row.channelId);
  }
  const channelIds = [...channels].sort();
  const partitionIds = [...partitions.keys()].sort();
  const data = partitionIds.map((id) => {
    const measurements = partitions.get(id)!;
    if (measurements.size !== channelIds.length) {
      throw new Error(`Partition ${id} is missing channel measurements`);
    }
    return channelIds.map((channel) => measurements.get(channel)!);
  });
  if (new Set(data.map((row) => JSON.stringify(row))).size < count) {
    throw new Error('Need at least as many distinct intensity vectors as clusters');
  }
  const result = kmeans(data, count, { seed: 42, initialization: 'kmeans++' });
  if (!result.converged) throw new Error('Clustering did not converge');
  return partitionIds.map((id, index) => [id, result.clusters[index]]);
}

if (import.meta.main) {
  if (process.argv.length < 3 || process.argv.length > 4) {
    throw new Error('usage: node examples/clustering.ts <archive.ddes> [clusters-per-well=2]');
  }
  const count = process.argv[3] === undefined ? 2 : Number(process.argv[3]);
  initSync({ module: readFileSync(new URL('../pkg/ddes_typescript_bg.wasm', import.meta.url)) });
  const archive = DdesArchive.fromBytes(readFileSync(process.argv[2]));
  try {
    if (!archive.intensityCount) throw new Error('Archive has no intensity files');
    console.log('well_id,assay_id,partition_id,cluster');
    for (const file of archive.intensities()) {
      for (const [partition, label] of clusterIntensities(file, count)) {
        const cells = [file.metadata.wellId, file.metadata.assayId, partition, String(label)];
        console.log(cells.map((cell) => `"${cell.replaceAll('"', '""')}"`).join(','));
      }
    }
  } finally { archive.free(); }
}
