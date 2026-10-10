import { existsSync } from 'node:fs';
import path from 'node:path';
if (process.env.GITHUB_ACTIONS !== 'true' || !existsSync(path.join(process.env.RIGHT_GIT_ARTIFACT_ROOT || '', 'compiler-artifacts.json'))) throw new Error('CI candidate record required before installation');
