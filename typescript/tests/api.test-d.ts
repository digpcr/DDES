import {
  DdesArchive, readMain, writeMain, readAssays, writeAssays, readIntensity, writeIntensity,
} from '../pkg/ddes_typescript.js';
import type {
  MainFile, MainRecord, AssaysFile, AssayRecord, IntensityFile, IntensityRecord,
} from '../pkg/ddes_typescript.js';

type Equals<A, B> =
  (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;
declare function assertType<T extends true>(): void;

assertType<Equals<ReturnType<typeof readIntensity>, IntensityFile>>();
assertType<Equals<Parameters<typeof writeIntensity>[0], IntensityFile>>();
assertType<Equals<ReturnType<typeof writeIntensity>, Uint8Array>>();
assertType<Equals<IntensityRecord['fiEndpoint'], number>>();
assertType<Equals<ReturnType<DdesArchive['intensity']>, IntensityFile>>();
assertType<Equals<ReturnType<typeof readMain>, MainFile>>();
assertType<Equals<Parameters<typeof writeMain>[0], MainFile>>();
assertType<Equals<ReturnType<typeof readAssays>, AssaysFile>>();
assertType<Equals<Parameters<typeof writeAssays>[0], AssaysFile>>();
assertType<Equals<MainRecord['countsPositive'], bigint | null>>();
assertType<Equals<MainRecord['countsNegative'], bigint | null>>();
assertType<Equals<MainRecord['concentrationReactionCpUl'], number | null>>();
assertType<Equals<MainRecord['thresholdIfThreshold'], number | null>>();
assertType<Equals<AssayRecord['probeConcentrationsNm'], string>>();
assertType<Equals<ConstructorParameters<typeof DdesArchive>, [MainFile, AssaysFile, IntensityFile[]]>>();
assertType<Equals<ReturnType<DdesArchive['main']>, MainFile>>();
assertType<Equals<ReturnType<DdesArchive['assays']>, AssaysFile>>();
assertType<Equals<ReturnType<DdesArchive['intensities']>, IntensityFile[]>>();
assertType<Equals<Parameters<DdesArchive['setMain']>, [MainFile]>>();
assertType<Equals<Parameters<DdesArchive['setAssays']>, [AssaysFile]>>();
assertType<Equals<Parameters<DdesArchive['setIntensities']>, [IntensityFile[]]>>();

declare const bytes: Uint8Array;
const file = readIntensity(bytes);
file.metadata.experimentId = 'editable';
file.records[0].fiEndpoint = -12.75;
writeIntensity(file);
// @ts-expect-error Endpoint measurements are numbers, not numeric tokens.
file.records[0].fiEndpoint = '12.75';
// @ts-expect-error Endpoint measurements cannot be missing.
file.records[0].fiEndpoint = null;
// @ts-expect-error TSV wire names are not the JS API names.
file.metadata.experiment_ID;
// @ts-expect-error Required metadata cannot be omitted.
writeIntensity({ records: [] });

const main = readMain(bytes);
main.records[0].countsPositive = 18446744073709551615n;
main.records[0].countsNegative = null;
main.records[0].concentrationReactionCpUl = 0;
main.records[0].thresholdIfThreshold = null;
writeMain(main);
// @ts-expect-error Counts require bigint, not potentially rounded JS numbers.
main.records[0].countsPositive = 42;
// @ts-expect-error Missing values must be explicit null.
main.records[0].thresholdIfThreshold = undefined;
// @ts-expect-error Properties remain required even if they allow null.
const row: MainRecord = { wellId: 'A1', sampleId: '001', assayId: 'a', targetId: 't' };
const assays = readAssays(bytes);
assays.records[0].probeConcentrationsNm = '125;NA;0';
writeAssays(assays);
// @ts-expect-error Probe lists are strings, not arrays or scalar measurements.
assays.records[0].probeConcentrationsNm = [125, null, 0];
const archive = new DdesArchive(main, assays, [file]);
archive.setMain(main);
archive.setAssays(assays);
archive.setIntensities([]);
// @ts-expect-error Component types cannot be mixed up.
archive.setMain(assays);
// @ts-expect-error Intensity collections require intensity files.
archive.setIntensities([main]);
// @ts-expect-error Counts are read-only.
archive.intensityCount = 7;
archive.free();
