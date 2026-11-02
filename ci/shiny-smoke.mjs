import assert from 'node:assert/strict';
import { spawn, execFileSync } from 'node:child_process';
import { createServer } from 'node:net';
import { mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { chromium } from 'playwright';

// Use an ephemeral port so local previews can stay running.
const socket = createServer();
await new Promise(resolve => socket.listen(0, '127.0.0.1', resolve));
const port = socket.address().port;
await new Promise(resolve => socket.close(resolve));
const directory = mkdtempSync(join(tmpdir(), 'ddes-shiny-'));
const source = join(directory, 'input.ddes');
execFileSync('cargo', ['run', '--locked', '--quiet', '--manifest-path', 'ddes-rs/Cargo.toml',
  '--example', 'roundtrip', '--', 'ddes-rs/tests/contract', source], { stdio: 'inherit' });
const server = spawn('Rscript', ['-e', `shiny::runApp("r/examples/shiny", host="127.0.0.1", port=${port}, launch.browser=FALSE)`], { stdio: 'inherit' });
const exited = new Promise(resolve => server.once('exit', resolve));
let browser;
try {
  const url = `http://127.0.0.1:${port}`;
  let ready = false;
  for (let attempt = 0; attempt < 120; attempt++) {
    if (server.exitCode !== null) throw new Error('Shiny exited before becoming ready');
    try { ready = (await fetch(url)).ok; } catch { /* Not listening yet. */ }
    if (ready) break;
    await new Promise(resolve => setTimeout(resolve, 500));
  }
  assert.ok(ready, 'Shiny did not start within 60 seconds');
  browser = await chromium.launch(process.env.CHROME_CHANNEL ? { channel: process.env.CHROME_CHANNEL } : {});
  const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.goto(url);
  await page.waitForFunction(() => document.querySelector('#status')?.innerText === 'Load a dataset to begin.');
  await page.locator('#ddes_upload').setInputFiles(source);
  await page.waitForFunction(() => document.querySelector('#ddes_upload_progress')?.innerText.includes('Upload complete'));
  await page.locator('#start').click();
  await page.waitForFunction(() => document.querySelector('#status')?.innerText.includes('Loaded 3 results'));
  await page.locator('#experiment_id').fill('browser-edited');
  await page.locator('#save_run').click();
  await page.waitForFunction(() => document.querySelector('#status')?.innerText === 'Run information saved.');
  await page.locator('a[data-value="samples"]').click();
  const cell = page.locator('#samples .ht_master tbody tr').first().locator('td').nth(1);
  await cell.dblclick();
  const editor = page.locator('#samples textarea.handsontableInput:visible');
  await editor.focus();
  await editor.press('ControlOrMeta+A');
  await editor.press('Backspace');
  await editor.pressSequentially('0001-edited');
  await editor.press('Enter');
  assert.equal(await cell.innerText(), '0001-edited');
  await page.locator('#save_samples').click();
  await page.waitForFunction(() => document.querySelector('#status')?.innerText === 'Sample IDs saved.');
  await page.locator('a[data-value="intensity"]').click();
  await page.waitForFunction(() => document.querySelectorAll('#intensity_records tbody tr').length === 4);
  assert.equal(await page.locator('#intensity_well').inputValue(), 'A1');
  mkdirSync('.amp/in/artifacts/ci', { recursive: true });
  await page.screenshot({ path: '.amp/in/artifacts/ci/shiny.png', fullPage: true });
  await page.locator('a[data-value="download"]').click();
  await page.locator('#prepare_download').click();
  const downloadPromise = page.waitForEvent('download');
  await page.locator('#download').click();
  const download = await downloadPromise;
  const output = join(directory, 'download.ddes');
  await download.saveAs(output);
  execFileSync('Rscript', ['-e', `library(ddes); x <- read_archive(${JSON.stringify(output)});
    expected <- read_archive("ddes-rs/tests/contract");
    expected$main$metadata$experiment_id <- "browser-edited";
    expected$assays$metadata$experiment_id <- "browser-edited";
    expected$intensities[[1]]$metadata$experiment_id <- "browser-edited";
    expected$main$records$sample_id[expected$main$records$well_id == "A1"] <- "0001-edited";
    stopifnot(identical(x, expected))`], { stdio: 'inherit' });
  const other = await browser.newPage();
  await other.goto(url);
  await other.waitForFunction(() => document.querySelector('#status')?.innerText === 'Load a dataset to begin.');
  assert.deepEqual(errors, []);
  console.log('Shiny edits, full download contents, intensity preview, and session isolation passed.');
} finally {
  await browser?.close();
  server.kill('SIGTERM');
  await exited;
  rmSync(directory, { recursive: true, force: true });
}
