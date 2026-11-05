import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const output = resolve('dist/npm');
mkdirSync(output, { recursive: true });
const packed = JSON.parse(execFileSync('npm', ['pack', './typescript/pkg', '--json', '--pack-destination', output], { encoding: 'utf8' }));
const directory = mkdtempSync(join(tmpdir(), 'ddes-npm-'));
try {
  execFileSync('npm', ['install', '--prefix', directory, '--ignore-scripts', '--no-audit', '--no-fund', join(output, packed[0].filename)], { stdio: 'inherit' });
  const pkg = join(directory, 'node_modules/ddes-typescript');
  const { initSync, readMain, writeMain, DdesArchive } = await import(pathToFileURL(join(pkg, 'ddes_typescript.js')));
  initSync({ module: readFileSync(join(pkg, 'ddes_typescript_bg.wasm')) });
  const consumer = join(directory, 'consumer.mts');
  writeFileSync(consumer, `import { readMain } from 'ddes-typescript';
    const count: bigint | null = readMain(new Uint8Array()).records[0].countsPositive;`);
  execFileSync(process.execPath, [resolve('typescript/node_modules/typescript/bin/tsc'),
    '--strict', '--noEmit', '--target', 'esnext', '--module', 'nodenext', '--lib', 'esnext,dom', consumer], { stdio: 'inherit' });
  const main = readMain(readFileSync('ddes-rs/tests/contract/DDES_contract_run_202601011200_main.tsv'));
  assert.equal(main.records[0].countsPositive, 9007199254740993n);
  assert.equal(main.records[1].countsPositive, 18446744073709551615n);
  assert.deepEqual(readMain(writeMain(main)), main);
  assert.equal(typeof DdesArchive.fromBytes, 'function');
  console.log('Installed and verified npm artifact:', packed[0].filename);
} finally {
  rmSync(directory, { recursive: true, force: true });
}
