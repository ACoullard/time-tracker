import { Resvg } from '@resvg/resvg-js';
import { readFileSync, writeFileSync, readdirSync } from 'fs';
import { fileURLToPath } from 'url';
import { resolve, basename } from 'path';

const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const svgDir = resolve(root, 'src-tauri/icons/svg');
const outDir = resolve(root, 'src-tauri/icons');

const svgFiles = readdirSync(svgDir).filter(f => f.endsWith('.svg'));

for (const file of svgFiles) {
  const name = basename(file, '.svg');
  const svgData = readFileSync(resolve(svgDir, file), 'utf-8');
  const resvg = new Resvg(svgData, { fitTo: { mode: 'width', value: 128 } });
  const pngData = resvg.render().asPng();
  writeFileSync(resolve(outDir, `${name}.png`), pngData);
  console.log(`wrote src-tauri/icons/${name}.png (${pngData.length} bytes)`);
}
