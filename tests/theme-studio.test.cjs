const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const studio = require('../apps/desktop/ui/theme-studio.js');

test('corrupt saved appearance cannot make the foreground invisible or overflow controls', () => {
  const appearance = studio.normalize({ transparencyLevel: 1000, cornerRadius: -20,
    backgroundBrightness: 'invalid', interfaceSize: 'huge' });
  assert.equal(appearance.transparencyLevel, 70);
  assert.equal(appearance.cornerRadius, 0);
  assert.equal(appearance.backgroundBrightness, studio.defaults.backgroundBrightness);
  assert.equal(appearance.interfaceSize, 'normal');
});

test('every theme ships a valid local full-size wallpaper and a separate small thumbnail', () => {
  assert.equal(studio.themes.length, 28);
  assert.equal(new Set(studio.themes.map(theme => theme[0])).size, 28);
  const dimensions = require('../apps/desktop/ui/assets/themes/dimensions.json');
  for (const [id] of studio.themes) {
    assert.ok(dimensions[id].width >= 1600 && dimensions[id].height >= 900, id);
    for (const thumb of [false, true]) {
      const bytes = fs.readFileSync(path.resolve(__dirname, '../apps/desktop/ui', studio.asset(id, thumb)));
      assert.ok(bytes.length > 1000, id);
      assert.equal(bytes.toString('ascii', 0, 4), 'RIFF');
      assert.equal(bytes.toString('ascii', 8, 12), 'WEBP');
    }
  }
});

test('maximum transparency retains a contrast backing, and light themes retain a stronger one', () => {
  const target = () => ({ dataset: {}, style: { values: {}, setProperty(key, value) { this.values[key] = value; } } });
  const dark = target(), light = target();
  studio.applyPresentation('cyberpunk', { transparencyLevel: 70 }, dark);
  studio.applyPresentation('linen', { transparencyLevel: 70 }, light);
  assert.ok(Number(dark.style.values['--glass-opacity']) >= .46);
  assert.ok(Number(light.style.values['--glass-opacity']) >= .74);
  assert.match(dark.style.values['--theme-background'], /cyberpunk\.webp/);
  assert.ok(!('opacity' in dark.style.values), 'text must not fade with the scenery');
});
