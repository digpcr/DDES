/** Edit all three component types and verify their TSV and archive round trips. */
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import {
  DdesArchive, initSync, readMain, writeMain, readAssays, writeAssays,
  readIntensity, writeIntensity,
} from '../pkg/ddes_typescript.js';

export function roundtrip(bytes: Uint8Array): Uint8Array {
  const archive = DdesArchive.fromBytes(bytes);
  try {
    const main = archive.main();
    if (main.records.length) {
      main.records[0].countsPositive = 18446744073709551615n;
      main.records[0].thresholdIfThreshold = null;
    }
    assert.deepEqual(readMain(writeMain(main)), main);
    const assays = archive.assays();
    if (assays.records.length) assays.records[0].probeConcentrationsNm = 'NA';
    assert.deepEqual(readAssays(writeAssays(assays)), assays);
    const intensities = archive.intensities();
    for (const file of intensities) {
      if (file.records.length) file.records[0].fiEndpoint = -12.75;
      assert.deepEqual(readIntensity(writeIntensity(file)), file);
    }
    // Snapshots are detached; explicitly copy edits back into the archive.
    archive.setMain(main);
    archive.setAssays(assays);
    archive.setIntensities(intensities);
    const output = archive.toBytes();
    const restored = DdesArchive.fromBytes(output);
    try {
      assert.deepEqual(restored.main(), main);
      assert.deepEqual(restored.assays(), assays);
      assert.deepEqual(restored.intensities(), intensities);
    } finally { restored.free(); }

    // The same plain objects can also construct an independent archive.
    const created = new DdesArchive(main, assays, intensities);
    try {
      assert.deepEqual(created.main(), main);
      assert.deepEqual(created.assays(), assays);
      assert.deepEqual(created.intensities(), intensities);
    } finally { created.free(); }
    return output;
  } finally { archive.free(); }
}

if (import.meta.main) {
  if (process.argv.length < 3 || process.argv.length > 4) {
    throw new Error('usage: node examples/roundtrip.ts <archive.ddes> [new-output.ddes]');
  }
  initSync({ module: readFileSync(new URL('../pkg/ddes_typescript_bg.wasm', import.meta.url)) });
  const bytes = roundtrip(readFileSync(process.argv[2]));
  if (process.argv[3]) {
    writeFileSync(process.argv[3], bytes, { flag: 'wx' });
    assert.deepEqual(readFileSync(process.argv[3]), Buffer.from(bytes));
  }
  console.log('Verified edited TSV and archive round trips');
}
