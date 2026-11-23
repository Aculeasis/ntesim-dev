import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { readdirSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { defineConfig, type Plugin } from '../cache/web/node_modules/vite/dist/node/index.js';
import react from '../cache/web/node_modules/@vitejs/plugin-react/dist/index.js';
import { presetDirectory, syncStudioPresets, checkTeamCatalog } from '../tools/studio_presets.mjs';
const root = fileURLToPath(new URL('../', import.meta.url)),
  modules = path.join(root, 'cache/web/node_modules'),
  out = path.join(root, 'target/web');
const studioBuild = JSON.parse(
  execFileSync('python', [path.join(root, 'tools/studio_build.py')], {
    cwd: root,
    encoding: 'utf8',
  }),
);
function codecDevSources(): Plugin {
  const directory = path.join(modules, '@bokuweb/zstd-wasm/dist/web');
  return {
    name: 'ntesim-codec-dev-sources',
    apply: 'serve',
    enforce: 'pre',
    load(id) {
      const file = id.split('?')[0],
        relative = path.relative(directory, file);
      if (relative.startsWith('..') || path.isAbsolute(relative) || !file.endsWith('.js')) return;
      // The pinned npm package includes maps to lib/*.ts, but omits those sources.
      // Remove only its broken map references before Vite tries to load them.
      return {
        code: readFileSync(file, 'utf8').replace(/^\/\/# sourceMappingURL=.*$/gm, ''),
        map: null,
      };
    },
  };
}
function studioPresets(): Plugin {
  return {
    name: 'ntesim-studio-presets',
    buildStart() {
      syncStudioPresets();
      checkTeamCatalog();
      this.addWatchFile(presetDirectory);
    },
    configureServer(server) {
      server.watcher.add(presetDirectory);
      const sync = (file: string) => {
        const relative = path.relative(presetDirectory, file);
        if (relative.startsWith('..') || path.isAbsolute(relative)) return;
        try {
          syncStudioPresets();
          checkTeamCatalog();
        } catch (error) {
          server.ws.send({ type: 'error', err: { message: String(error), stack: '' } });
        }
      };
      server.watcher.on('add', sync).on('change', sync).on('unlink', sync);
    },
  };
}
function offline(): Plugin {
  return {
    name: 'ntesim-offline',
    closeBundle() {
      execFileSync('python', [path.join(root, 'tools/browser_distribution.py'), 'stage', out], {
        cwd: root,
        stdio: 'inherit',
        env: {
          ...process.env,
          NTESIM_BUILD_STARTED_AT: studioBuild.build_started_at,
          NTESIM_STUDIO_BUILD: JSON.stringify(studioBuild),
        },
      });
      const files = (dir: string): string[] =>
        readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
          entry.isDirectory()
            ? files(path.join(dir, entry.name))
            : [path.relative(out, path.join(dir, entry.name)).replaceAll('\\', '/')],
        );
      const assets = files(out)
        .filter((file) => file !== 'sw.js')
        .sort();
      const hash = createHash('sha256');
      for (const file of assets) {
        hash.update(file);
        hash.update(readFileSync(path.join(out, file)));
      }
      const version = hash.digest('hex').slice(0, 16);
      const source = readFileSync(path.join(root, 'web/offline-worker.template.js'), 'utf8')
        .replace('__CACHE_VERSION__', version)
        .replace(
          '__ASSET_MANIFEST__',
          JSON.stringify(['./', ...assets.map((file) => './' + file)]),
        );
      writeFileSync(path.join(out, 'sw.js'), source);
    },
  };
}
export default defineConfig({
  define: { __STUDIO_BUILD__: JSON.stringify(studioBuild) },
  base: './',
  cacheDir: path.join(root, 'cache/vite'),
  publicDir: 'public',
  plugins: [
    codecDevSources(),
    studioPresets(),
    react({
      exclude: [/[\\/]node_modules[\\/]/, /[\\/]\.?cache[\\/]/],
      babel: { compact: true },
    }),
    offline(),
  ],
  resolve: {
    alias: [
      {
        find: /^@report-codec-wasm(?=\?|$)/,
        replacement: path.join(modules, '@bokuweb/zstd-wasm/dist/web/zstd.wasm'),
      },
      {
        find: '@bokuweb/zstd-wasm',
        replacement: path.join(modules, '@bokuweb/zstd-wasm/dist/web/index.web.js'),
      },
      { find: 'html-to-image', replacement: path.join(modules, 'html-to-image/es/index.js') },
      { find: /^react-dom(\/.*)?$/, replacement: path.join(modules, 'react-dom') + '$1' },
      { find: /^react(\/.*)?$/, replacement: path.join(modules, 'react') + '$1' },
    ],
    dedupe: ['react', 'react-dom'],
  },
  optimizeDeps: { exclude: ['@bokuweb/zstd-wasm'] },
  build: { outDir: out, emptyOutDir: true },
  worker: { format: 'es' },
  server: { host: '127.0.0.1', port: 5173, strictPort: true, fs: { allow: [root] } },
});
