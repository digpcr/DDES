import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import {
  DdesArchive, initSync, readMain, writeMain, readAssays, writeAssays,
  readIntensity, writeIntensity,
} from '../pkg/ddes_typescript.js';
import { clusterIntensities } from '../examples/clustering.ts';

// Exercise the web artifact itself, not a mock or a separate Node-target build.
initSync({ module: readFileSync(new URL('../pkg/ddes_typescript_bg.wasm', import.meta.url)) });
const core = fileURLToPath(new URL('../../ddes-rs/', import.meta.url));
const fixture = readFileSync(join(core, 'tests/fixtures/DDES_expID_run_202506301127_intensity_A1.tsv'));
const mainFixture = readFileSync(join(core, 'tests/fixtures/DDES_expID_run_202506301127_main.tsv'));
const assaysFixture = readFileSync(join(core, 'tests/fixtures/DDES_expID_run_202506301127_assays.tsv'));
const encode = (text) => new TextEncoder().encode(text);

test('reference data becomes editable camelCase objects and survives normalized TSV output', () => {
  const file = readIntensity(fixture);
  assert.deepEqual(file.metadata, {
    ddesVersion: '1.0', datetime: '202506301127', experimentId: 'expID',
    runId: '5160815c-092d-49ea-946c-cf7bcb49432a', ddesType: 'intensity',
    wellId: 'A1', assayId: 'HIV Triplex',
  });
  assert.equal(file.records.length, 9);
  assert.deepEqual(file.records[0], { partitionId: '9', channelId: 'CRIMSON', fiEndpoint: 34.65 });
  assert.deepEqual(file.records[8], { partitionId: '11', channelId: 'YELLOW', fiEndpoint: 30.09 });
  file.metadata.experimentId = 'edited-µ';
  file.records[0] = { partitionId: '001\t"µ"\n#partition', channelId: 'BLUE', fiEndpoint: -12.75 };
  const bytes = writeIntensity(file);
  assert.ok(bytes instanceof Uint8Array);
  assert.deepEqual(readIntensity(bytes), file);
  file.records = [];
  assert.deepEqual(readIntensity(writeIntensity(file)), file);
});

test('endpoint intensities use finite JS numbers and reject missing/nonfinite values', () => {
  const input = ['0', '-12.75', '1.2500', '1e2'];
  const expected = [0, -12.75, 1.25, 100];
  const header = fixture.toString('utf8').split('#data\n')[0];
  const bytes = encode(`${header}#data\npartition_ID\tchannel_ID\tFI_endpoint\n`
    + input.map((value, i) => `${i}\tGREEN\t${value}\n`).join(''));
  const file = readIntensity(bytes);
  assert.deepEqual(file.records.map((row) => row.fiEndpoint), expected);
  assert.deepEqual(readIntensity(writeIntensity(file)), file);
  for (const value of [Number.MIN_VALUE, Number.MAX_VALUE, -1.2345678901234567]) {
    file.records[0].fiEndpoint = value;
    assert.equal(readIntensity(writeIntensity(file)).records[0].fiEndpoint, value);
  }
  for (const token of ['', 'NA', 'NaN', 'Inf', '-Inf', '-', '1e400']) {
    const invalid = fixture.toString('utf8').replace('34.65', token);
    assert.throws(() => readIntensity(encode(invalid)), { name: 'Error' });
  }
  // Invalid JS values must be rejected, not silently converted into missing values.
  for (const value of [null, undefined, NaN, Infinity, -Infinity, '12.5']) {
    file.records[0].fiEndpoint = value;
    assert.throws(() => writeIntensity(file), { name: 'Error' });
  }
});

test('parser, ZIP, and malformed JS input failures throw Error objects', () => {
  assert.throws(() => readIntensity(encode('bad input')), { name: 'Error', message: /intensity\.tsv/ });
  assert.throws(() => DdesArchive.fromBytes(encode('not a ZIP')), { name: 'Error', message: /ZIP/ });
  assert.throws(() => writeIntensity({ records: [] }), { name: 'Error', message: /metadata/ });
  const file = readIntensity(fixture);
  file.records[0].fiEndpoint = '123';
  assert.throws(() => writeIntensity(file), { name: 'Error', message: /f64/ });
  file.records[0].fiEndpoint = 123;
  file.metadata.extraField = 'not silently lost';
  assert.throws(() => writeIntensity(file), { name: 'Error', message: /unknown field/ });
  delete file.metadata.extraField;
  file.metadata.ddesType = 'main';
  assert.throws(() => writeIntensity(file), { name: 'Error', message: /DDES_type/ });
});

test('native ZIP → WASM → native ZIP, with detached snapshots and explicit ownership', () => {
  const dir = mkdtempSync(join(tmpdir(), 'ddes-wasm-'));
  try {
    const nativePath = join(dir, 'native.ddes');
    execFileSync('cargo', ['run', '--locked', '--quiet', '--example', 'roundtrip', '--',
      'tests/fixtures', nativePath], { cwd: core });
    const archive = DdesArchive.fromBytes(readFileSync(nativePath));
    let snapshot, bytes;
    try {
      assert.equal(archive.intensityCount, 1);
      snapshot = archive.intensity(0);
      assert.deepEqual(snapshot, readIntensity(fixture));
      snapshot.records[0].fiEndpoint = 777;
      assert.equal(archive.intensity(0).records[0].fiEndpoint, 34.65);
      for (const index of [-1, 0.5, NaN, Infinity, 1, 2 ** 32, '0', null, undefined, 0n]) {
        assert.throws(() => archive.intensity(index), { name: 'Error', message: /index/ });
      }
      bytes = archive.toBytes();
    } finally {
      archive.free();
    }
    // Both outputs outlive the Rust handle; no dangling WASM memory views.
    assert.equal(readIntensity(writeIntensity(snapshot)).records[0].fiEndpoint, 777);
    const reopened = DdesArchive.fromBytes(bytes);
    try {
      assert.deepEqual(reopened.intensity(0), readIntensity(fixture));
    } finally {
      reopened.free();
    }
    const wasmPath = join(dir, 'wasm.ddes');
    writeFileSync(wasmPath, bytes);
    const result = execFileSync('cargo', ['run', '--locked', '--quiet', '--example', 'roundtrip', '--',
      wasmPath, join(dir, 'native-reloaded.ddes')], { cwd: core, encoding: 'utf8' });
    assert.match(result, /main records: 3, assay records: 1, intensity files: 1, intensity records: 9/);
    assert.match(result, /Verified semantic round trip/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('main values are exact bigints, numeric measurements, and explicit nulls', () => {
  const main = readMain(mainFixture);
  assert.equal(main.records.length, 3);
  const row = main.records[0];
  assert.equal(row.countsPositive, 4236n);
  assert.equal(row.countsNegative, 21173n);
  for (const count of [0n, 9007199254740993n, 18446744073709551615n, null]) {
    row.countsPositive = count;
    row.countsNegative = count;
    row.concentrationReactionCpUl = 0;
    row.thresholdIfThreshold = -12.75;
    assert.deepEqual(readMain(writeMain(main)), main);
  }
  row.concentrationReactionCpUl = null;
  row.thresholdIfThreshold = null;
  assert.deepEqual(readMain(writeMain(main)), main);
  for (const field of ['countsPositive', 'countsNegative']) {
    for (const value of [-1n, 18446744073709551616n, 0, 1.5, '42', true, undefined]) {
      row[field] = value;
      assert.throws(() => writeMain(main), Error);
    }
    delete row[field];
    assert.throws(() => writeMain(main), Error);
    row[field] = null;
  }
  for (const field of ['concentrationReactionCpUl', 'thresholdIfThreshold']) {
    for (const value of [NaN, Infinity, -Infinity, '12.5', 1n, true, undefined]) {
      row[field] = value;
      assert.throws(() => writeMain(main), Error);
    }
    delete row[field];
    assert.throws(() => writeMain(main), Error);
    row[field] = null;
  }
  row.concentrationReactionCpUl = -1;
  assert.throws(() => writeMain(main), Error);
});

test('all component snapshots support construction, nested edits, and list replacement', () => {
  for (const [read, write, input] of [
    [readMain, writeMain, mainFixture], [readAssays, writeAssays, assaysFixture],
    [readIntensity, writeIntensity, fixture],
  ]) {
    const original = read(input);
    const file = { metadata: { ...original.metadata }, records: [{ ...original.records[0] }] };
    file.metadata.experimentId = 'edited-µ';
    const id = Object.keys(file.records[0])[0];
    file.records[0][id] = '001\t"µ"\n#row';
    assert.deepEqual(read(write(file)), file);
    for (const target of [file, file.metadata, file.records[0]]) {
      target.extraField = undefined;
      assert.throws(() => write(file), { name: 'Error', message: /unknown field/ });
      delete target.extraField;
    }
    file.records.push({});
    assert.throws(() => write(file), Error);
    file.records = [];
    assert.deepEqual(read(write(file)), file);
    assert.throws(() => read(encode('bad input')), Error);
    assert.throws(() => write({ records: [] }), Error);
  }
});

test('assay concentrations retain whole-list and individual missing entries', () => {
  const assays = readAssays(assaysFixture);
  for (const value of ['NA', '125;NA;0', '1.2500;2e2']) {
    assays.records[0].probeConcentrationsNm = value;
    assert.deepEqual(readAssays(writeAssays(assays)), assays);
  }
  for (const type of ['assay', 'assays']) {
    assays.metadata.ddesType = type;
    assert.deepEqual(readAssays(writeAssays(assays)), assays);
  }
  for (const value of [null, undefined, 12, '-1', 'NaN', 'Infinity', '1;;2']) {
    assays.records[0].probeConcentrationsNm = value;
    assert.throws(() => writeAssays(assays), Error);
  }
});

test('archives copy components, apply edits atomically, and retain scientific values until write', () => {
  const main = readMain(mainFixture);
  const assays = readAssays(assaysFixture);
  const a = readIntensity(fixture);
  const b = structuredClone(a);
  b.metadata.wellId = 'B2';
  b.records[0].fiEndpoint = -8.25;
  const archive = new DdesArchive(main, assays, [b, a]);
  let saved, snapshots;
  try {
    main.records[0].countsPositive = 18446744073709551615n;
    assays.records[0].probeConcentrationsNm = 'NA';
    a.records[0].fiEndpoint = 777;
    assert.deepEqual(archive.main(), readMain(mainFixture));
    assert.deepEqual(archive.assays(), readAssays(assaysFixture));
    assert.equal(archive.intensity(1).records[0].fiEndpoint, 34.65);
    archive.setMain(main);
    archive.setAssays(assays);
    archive.setIntensities([b, a]);
    assert.deepEqual(archive.main(), main);
    assert.deepEqual(archive.assays(), assays);
    assert.deepEqual(archive.intensities(), [b, a]);
    saved = archive.toBytes();
    const restored = DdesArchive.fromBytes(saved);
    try {
      assert.deepEqual(restored.main(), main);
      assert.deepEqual(restored.assays(), assays);
      assert.deepEqual(restored.intensities(), [a, b]); // Reads sort by well ID.
    } finally { restored.free(); }
    main.records[0].countsPositive = 1n;
    assays.records[0].probeConcentrationsNm = '0';
    a.records[0].fiEndpoint = 19.5;
    assert.equal(archive.main().records[0].countsPositive, 18446744073709551615n);
    assert.equal(archive.assays().records[0].probeConcentrationsNm, 'NA');
    assert.equal(archive.intensity(1).records[0].fiEndpoint, 777);
    snapshots = [archive.main(), archive.assays(), archive.intensities()];
    b.records[0].fiEndpoint = 123;
    a.records.push({});
    assert.throws(() => archive.setIntensities([b, a]), Error);
    assert.deepEqual(archive.intensities(), snapshots[2]);
    for (const [file, setter, getter] of [
      [main, 'setMain', 'main'], [assays, 'setAssays', 'assays'],
    ]) {
      const before = archive[getter]();
      file.records.push({});
      assert.throws(() => archive[setter](file), Error);
      assert.deepEqual(archive[getter](), before);
    }
    const edited = archive.main();
    edited.records[0].thresholdIfThreshold = NaN;
    archive.setMain(edited);
    assert.ok(Number.isNaN(archive.main().records[0].thresholdIfThreshold));
    assert.throws(() => archive.toBytes(), Error);
    archive.setMain(snapshots[0]);
    const unsafe = readIntensity(fixture);
    unsafe.metadata.wellId = '../escape';
    for (const files of [[b, b], [unsafe]]) {
      archive.setIntensities(files);
      assert.throws(() => archive.toBytes(), Error);
    }
    archive.setIntensities([]);
    assert.equal(archive.intensityCount, 0);
    const empty = DdesArchive.fromBytes(archive.toBytes());
    try { assert.deepEqual(empty.intensities(), []); } finally { empty.free(); }
  } finally { archive.free(); }
  assert.equal(readMain(writeMain(snapshots[0])).records[0].countsPositive, 18446744073709551615n);
  const survivor = DdesArchive.fromBytes(saved);
  try { assert.equal(survivor.intensityCount, 2); } finally { survivor.free(); }
  assert.throws(() => new DdesArchive({}, {}, []), Error);
});

test('clustering aligns multichannel rows by exact ID without changing data', () => {
  const file = readIntensity(fixture);
  file.records = [
    ['04', 'BLUE', -2], ['001', 'BLUE', -2], ['03', 'BLUE', -1], ['010', 'BLUE', -1],
    ['010', 'GREEN', 103], ['03', 'GREEN', -20], ['001', 'GREEN', 100], ['04', 'GREEN', -15],
  ].map(([partitionId, channelId, fiEndpoint]) => ({ partitionId, channelId, fiEndpoint }));
  const before = structuredClone(file);
  const assignments = clusterIntensities(file, 2);
  assert.deepEqual(assignments.map(([id]) => id), ['001', '010', '03', '04']);
  const labels = new Map(assignments);
  assert.equal(labels.get('001'), labels.get('010'));
  assert.equal(labels.get('03'), labels.get('04'));
  assert.notEqual(labels.get('001'), labels.get('03'));
  assert.deepEqual(file, before);
  file.records.reverse();
  assert.deepEqual(clusterIntensities(file, 2), assignments);
  assert.deepEqual(new Set(clusterIntensities(file, 1).map(([, label]) => label)), new Set([0]));
  for (const count of [0, -1, 0.5, NaN, Infinity, 5]) {
    assert.throws(() => clusterIntensities(file, count), Error);
  }
  for (const records of [
    [...before.records, before.records[0]], before.records.slice(1), [],
    before.records.map((r) => ({ ...r, fiEndpoint: 0 })),
    before.records.map((r) => ({ ...r, fiEndpoint: NaN })),
  ]) {
    file.records = records;
    assert.throws(() => clusterIntensities(file, 2), Error);
  }
});

test('Node examples verify edited output, refuse overwrite, and emit cluster CSV', () => {
  const dir = mkdtempSync(join(tmpdir(), 'ddes-examples-'));
  const archive = new DdesArchive(readMain(mainFixture), readAssays(assaysFixture), [readIntensity(fixture)]);
  try {
    const input = join(dir, 'input.ddes');
    const output = join(dir, 'output.ddes');
    const inputBytes = Buffer.from(archive.toBytes());
    writeFileSync(input, inputBytes);
    const roundtripExample = fileURLToPath(new URL('../examples/roundtrip.ts', import.meta.url));
    const result = execFileSync(process.execPath, [roundtripExample, input, output], { encoding: 'utf8' });
    assert.match(result, /Verified edited TSV and archive round trips/);
    const savedBytes = readFileSync(output);
    const restored = DdesArchive.fromBytes(savedBytes);
    try {
      assert.equal(restored.main().records[0].countsPositive, 18446744073709551615n);
      assert.equal(restored.main().records[0].thresholdIfThreshold, null);
      assert.equal(restored.assays().records[0].probeConcentrationsNm, 'NA');
      assert.equal(restored.intensity(0).records[0].fiEndpoint, -12.75);
    } finally { restored.free(); }
    assert.throws(
      () => execFileSync(process.execPath, [roundtripExample, input, output], { stdio: 'pipe' }),
      (error) => /EEXIST/.test(error.stderr.toString()),
    );
    assert.deepEqual(readFileSync(output), savedBytes);
    const clusterExample = fileURLToPath(new URL('../examples/clustering.ts', import.meta.url));
    const csv = execFileSync(process.execPath, [clusterExample, input, '2'], { encoding: 'utf8' });
    const lines = csv.trim().split('\n');
    assert.equal(lines[0], 'well_id,assay_id,partition_id,cluster');
    const rows = lines.slice(1).map((line) => /^"A1","HIV Triplex","(\d+)","(\d+)"$/.exec(line));
    assert.ok(rows.every(Boolean));
    assert.deepEqual(rows.map((r) => r[1]), ['10', '11', '9']);
    assert.equal(rows[0][2], rows[1][2]);
    assert.notEqual(rows[0][2], rows[2][2]);
    assert.deepEqual(readFileSync(input), inputBytes);
  } finally {
    archive.free();
    rmSync(dir, { recursive: true, force: true });
  }
});
