import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';

const rootDir = fs.existsSync(path.resolve('src-tauri'))
  ? process.cwd()
  : path.resolve('..');
const tauriDir = path.join(rootDir, 'src-tauri');
const universalDir = path.join(tauriDir, 'target', 'universal-apple-darwin', 'release');
const targetAgm = path.join(universalDir, 'agm');

if (fs.existsSync(universalDir) && !fs.existsSync(targetAgm)) {
  console.log('[before-bundle] Universal Darwin target detected. Preparing universal agm binary...');
  const x64Agm = path.join(tauriDir, 'target', 'x86_64-apple-darwin', 'release', 'agm');
  const armAgm = path.join(tauriDir, 'target', 'aarch64-apple-darwin', 'release', 'agm');

  if (!fs.existsSync(x64Agm)) {
    try {
      console.log('[before-bundle] Compiling x86_64 agm...');
      execSync('cargo build --target x86_64-apple-darwin --release --bin agm', { cwd: tauriDir, stdio: 'inherit' });
    } catch (err) {
      console.warn('[before-bundle] Warning: Could not compile x86_64 agm:', err.message);
    }
  }

  if (!fs.existsSync(armAgm)) {
    try {
      console.log('[before-bundle] Compiling aarch64 agm...');
      execSync('cargo build --target aarch64-apple-darwin --release --bin agm', { cwd: tauriDir, stdio: 'inherit' });
    } catch (err) {
      console.warn('[before-bundle] Warning: Could not compile aarch64 agm:', err.message);
    }
  }

  if (fs.existsSync(x64Agm) && fs.existsSync(armAgm)) {
    try {
      console.log('[before-bundle] Merging x86_64 and aarch64 agm binaries via lipo...');
      execSync(`lipo -create -output "${targetAgm}" "${x64Agm}" "${armAgm}"`, { stdio: 'inherit' });
      fs.chmodSync(targetAgm, 0o755);
      console.log('[before-bundle] Successfully generated universal binary:', targetAgm);
    } catch (err) {
      console.warn('[before-bundle] lipo failed, attempting single architecture fallback:', err.message);
      const fallbackSource = fs.existsSync(armAgm) ? armAgm : x64Agm;
      fs.copyFileSync(fallbackSource, targetAgm);
      fs.chmodSync(targetAgm, 0o755);
    }
  } else if (fs.existsSync(armAgm)) {
    console.log('[before-bundle] Using aarch64 agm as fallback universal binary...');
    fs.copyFileSync(armAgm, targetAgm);
    fs.chmodSync(targetAgm, 0o755);
  } else if (fs.existsSync(x64Agm)) {
    console.log('[before-bundle] Using x86_64 agm as fallback universal binary...');
    fs.copyFileSync(x64Agm, targetAgm);
    fs.chmodSync(targetAgm, 0o755);
  } else {
    try {
      console.log('[before-bundle] Compiling native agm binary as fallback...');
      execSync('cargo build --release --bin agm', { cwd: tauriDir, stdio: 'inherit' });
      const nativeAgm = path.join(tauriDir, 'target', 'release', 'agm');
      if (fs.existsSync(nativeAgm)) {
        fs.copyFileSync(nativeAgm, targetAgm);
        fs.chmodSync(targetAgm, 0o755);
      }
    } catch (err) {
      console.warn('[before-bundle] Could not create fallback agm:', err.message);
    }
  }
}
