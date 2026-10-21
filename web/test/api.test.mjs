// Tests the built package as a JavaScript caller sees it.

import assert from 'node:assert/strict';
import { describe, test } from 'node:test';

import {
  Duration,
  Epoch,
  TimeScale,
  TimeSeries,
  Unit,
  Weekday,
  constants,
  parseTimeScale,
  version,
} from '../pkg-inline/hifitime-inline.js';

describe('hifitime', () => {
  test('needs no init call', () => {
    assert.match(version(), /^\d+\.\d+\.\d+/);
  });

  test('parses and formats an epoch', () => {
    const epoch = new Epoch('2024-02-29T12:34:56.789 UTC');
    assert.equal(epoch.toString(), '2024-02-29T12:34:56.789000000 UTC');
    assert.equal(epoch.toGregorianString(TimeScale.GPST), '2024-02-29T12:35:14.789000000 GPST');
    assert.equal(epoch.timeScale, TimeScale.UTC);
    assert.equal(epoch.year, 2024);
    assert.equal(epoch.weekdayUtc(), Weekday.Thursday);
  });

  test('keeps every nanosecond as a bigint', () => {
    const step = Duration.fromParts(3, 12n);
    assert.equal(step.totalNanoseconds(), 9467280000000000012n);
    assert.equal(typeof step.toSeconds(), 'number');

    const epoch = new Epoch('2024-02-29T12:34:56.123456789 UTC');
    assert.equal(epoch.toUnixNanoseconds(), 1709210096123456789n);
    assert.ok(Epoch.fromUnixNanoseconds(epoch.toUnixNanoseconds()).equals(epoch));
    assert.equal(constants().nanosecondsPerDay, 86400000000000n);
  });

  test('does exact arithmetic', () => {
    const launch = new Epoch('2024-02-29T12:34:56.789 UTC');
    const cutoff = launch.add(Duration.fromHours(1.5));
    assert.equal(cutoff.durationSince(launch).toString(), '1 h 30 min');
    assert.equal(launch.addUnit(90, Unit.Minute).compare(cutoff), 0);
  });

  test('serializes with JSON.stringify', () => {
    const value = { at: new Epoch('2024-01-01T00:00:00 UTC'), span: new Duration('1 d') };
    assert.equal(JSON.stringify(value), '{"at":"2024-01-01T00:00:00 UTC","span":"1 day"}');
  });

  test('sorts with compare', () => {
    const epochs = ['2024-03-01T00:00:00 UTC', '2023-01-01T00:00:00 UTC', '2024-01-01T00:00:00 UTC'].map(
      (text) => new Epoch(text),
    );
    epochs.sort((a, b) => a.compare(b));
    assert.deepEqual(
      epochs.map((epoch) => epoch.year),
      [2023, 2024, 2024],
    );
  });

  test('throws a normal Error', () => {
    assert.throws(() => Epoch.fromGregorianUtc(2023, 2, 29, 0, 0, 0, 0), (error) => {
      assert.ok(error instanceof Error);
      assert.equal(error.message, 'InvalidGregorianDate');
      return true;
    });
    assert.throws(() => new Epoch('not an epoch'), Error);
    assert.throws(() => parseTimeScale('XYZ'), Error);
  });

  test('converts to and from a Date', () => {
    const date = new Date('2024-02-29T12:34:56.789Z');
    const epoch = Epoch.fromDate(date);
    assert.equal(epoch.toIsoformat(), '2024-02-29T12:34:56.789000');
    assert.equal(epoch.toUnixNanoseconds(), 1709210096789000000n);
    assert.equal(epoch.toDate().getTime(), date.getTime());
  });

  test('walks a time series', () => {
    const start = new Epoch('2024-01-01T00:00:00 UTC');
    const end = new Epoch('2024-01-01T02:00:00 UTC');
    const series = TimeSeries.inclusive(start, end, Duration.fromMinutes(30));
    assert.equal(series.count(), 5);
    assert.equal(series.toStringArray().at(-1), '2024-01-01T02:00:00 UTC');
    let seen = 0;
    for (let epoch = series.next(); epoch !== undefined; epoch = series.next()) {
      seen += 1;
      epoch.free();
    }
    assert.equal(seen, 5);
  });

  test('releases memory with free', () => {
    const epoch = Epoch.now();
    epoch.free();
    assert.throws(() => epoch.toString());
  });
});
