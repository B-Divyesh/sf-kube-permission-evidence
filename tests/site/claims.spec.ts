import { expect, test } from '@playwright/test';
import { chmodSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

const cli = resolve('target/debug/kpe');
const examples = resolve('examples');

type RunResult = ReturnType<typeof spawnSync>;

function temporaryDirectory(label: string): string {
  return mkdtempSync(join(tmpdir(), `kpe-${label}-`));
}

function runCli(args: string[], cwd = temporaryDirectory('run'), env: NodeJS.ProcessEnv = process.env): RunResult {
  return runBinary(cli, args, cwd, env);
}

function runBinary(binary: string, args: string[], cwd = temporaryDirectory('run'), env: NodeJS.ProcessEnv = process.env): RunResult {
  return spawnSync(binary, args, { cwd, env, encoding: 'utf8' });
}

function installPackagedCli(): string {
  const packaged = spawnSync('cargo', ['package', '--locked', '--allow-dirty'], { cwd: resolve('.'), encoding: 'utf8' });
  if (packaged.status !== 0) throw new Error(`cargo package failed:\n${packaged.stdout}\n${packaged.stderr}`);

  const root = temporaryDirectory('consumer');
  const stagedPackage = resolve('target/package/kube-permission-evidence-0.1.0');
  const installed = spawnSync('cargo', ['install', '--locked', '--path', stagedPackage, '--root', root], { cwd: root, encoding: 'utf8' });
  if (installed.status !== 0) throw new Error(`cargo install failed:\n${installed.stdout}\n${installed.stderr}`);
  return join(root, 'bin', 'kpe');
}

function runDemo(): { directory: string; result: RunResult; report: Record<string, any> } {
  const parent = temporaryDirectory('demo');
  const directory = join(parent, 'sample');
  const result = runCli(['demo', '--output', directory], parent, { ...process.env, PATH: '' });
  const report = JSON.parse(readFileSync(join(directory, 'evidence.json'), 'utf8')) as Record<string, any>;
  return { directory, result, report };
}

function baseSnapshot(): Record<string, any> {
  return JSON.parse(readFileSync(join(examples, 'rbac-snapshot.json'), 'utf8')) as Record<string, any>;
}

function evaluateSnapshot(
  snapshot: Record<string, any>,
  checks: Record<string, unknown>[],
  subject = 'user:alice@example.com',
  groups: string[] = [],
): { result: RunResult; report: Record<string, any> } {
  const directory = temporaryDirectory('matrix');
  const snapshotPath = join(directory, 'snapshot.json');
  const matrixPath = join(directory, 'matrix.json');
  writeFileSync(snapshotPath, JSON.stringify(snapshot));
  writeFileSync(matrixPath, JSON.stringify({ checks }));
  const args = ['report', '--snapshot', snapshotPath, '--subject', subject];
  for (const group of groups) args.push('--as-group', group);
  args.push('--matrix', matrixPath, '--json');
  const result = runCli(args, directory);
  return { result, report: result.status === 0 ? JSON.parse(result.stdout) as Record<string, any> : {} };
}

function controlledCollection(): { calls: string[]; snapshot: string; status: number | null } {
  const directory = temporaryDirectory('collector');
  const kubectl = join(directory, 'kubectl');
  const log = join(directory, 'calls.log');
  const kubeconfig = join(directory, 'kubeconfig');
  const output = join(directory, 'snapshot.json');
  writeFileSync(kubeconfig, 'token: NEVER_INCLUDE_THIS_TOKEN\n');
  writeFileSync(kubectl, `#!/bin/sh
printf '%s\\n' "$*" >> "$KPE_CALL_LOG"
if [ "$1" = "config" ]; then printf 'sample-context\\n'; exit 0; fi
if [ "$1" = "version" ] && [ "$2" = "--client" ]; then printf '{"clientVersion":{"gitVersion":"v1.33.4"}}\\n'; exit 0; fi
if [ "$1" = "version" ]; then printf '{"serverVersion":{"gitVersion":"v1.33.4"}}\\n'; exit 0; fi
if [ "$1" = "get" ]; then printf '{"items":[]}\\n'; exit 0; fi
exit 1
`);
  chmodSync(kubectl, 0o755);
  const result = runCli(['snapshot', '--kubeconfig', kubeconfig, '--output', output], directory, {
    ...process.env,
    PATH: directory,
    KPE_CALL_LOG: log,
  });
  return {
    calls: readFileSync(log, 'utf8').trim().split('\n'),
    snapshot: readFileSync(output, 'utf8'),
    status: result.status,
  };
}

test('@claim:point-in-time-proof creates an auditor-readable packet with causal evidence', () => {
  const { result, report, directory } = runDemo();
  expect(result.status).toBe(0);
  expect(report.schemaVersion).toBe('kpe.evidence/v1');
  expect(report.generatedAt).toMatch(/^\d{4}-\d{2}-\d{2}T/);
  expect(report.subject).toMatchObject({ kind: 'User', name: 'alice@example.com' });
  expect(report.checks[0].grants[0]).toMatchObject({ bindingName: 'alice-secrets', roleName: 'secret-reader', ruleIndex: 0 });
  expect(readFileSync(join(directory, 'evidence.md'), 'utf8')).toContain('## Causal proof');
  const cast = readFileSync('site/public/kpe-demo.cast', 'utf8').trim().split('\n').slice(1)
    .map((line) => (JSON.parse(line) as [number, string, string])[2].replaceAll('\r', '')).join('');
  expect(cast).toContain(result.stdout.trim().split('\n').slice(0, 3).join('\n'));
});

test('@claim:collector-command-boundary invokes only the documented kubectl commands', () => {
  const { calls, status } = controlledCollection();
  expect(status).toBe(0);
  expect(calls).toEqual([
    'version --client -o json',
    expect.stringMatching(/^get roles --all-namespaces -o json --kubeconfig /),
    expect.stringMatching(/^get rolebindings --all-namespaces -o json --kubeconfig /),
    expect.stringMatching(/^get clusterroles -o json --kubeconfig /),
    expect.stringMatching(/^get clusterrolebindings -o json --kubeconfig /),
    expect.stringMatching(/^config current-context --kubeconfig /),
    expect.stringMatching(/^version -o json --kubeconfig /),
  ]);
});

test('@claim:bounded-matrix evaluates exactly the bundled four questions', () => {
  const { report } = runDemo();
  const matrix = JSON.parse(readFileSync(join(examples, 'matrix.json'), 'utf8')) as { checks: unknown[] };
  expect(report.summary.total).toBe(4);
  expect(report.checks).toHaveLength(matrix.checks.length);
  for (const [index, check] of matrix.checks.entries()) {
    expect(report.checks[index].request).toMatchObject(check as Record<string, unknown>);
  }
  const directory = temporaryDirectory('matrix-typo');
  const typo = join(directory, 'matrix.json');
  writeFileSync(typo, JSON.stringify({ checks: [{ verb: 'get', apiGruop: 'apps', resource: 'deployments' }] }));
  const rejected = runCli([
    'report', '--snapshot', join(examples, 'rbac-snapshot.json'), '--subject', 'user:alice@example.com',
    '--matrix', typo, '--json',
  ], directory);
  expect(rejected.status).toBe(1);
  expect(rejected.stderr).toContain('unknown field `apiGruop`');
});

test('@claim:all-grant-paths records every matching binding and rule', () => {
  const snapshot = baseSnapshot();
  snapshot.roleBindings.push({ ...snapshot.roleBindings[0], metadata: { name: 'second-secret-path', namespace: 'payments' } });
  const { report } = evaluateSnapshot(snapshot, [{ verb: 'get', resource: 'secrets', namespace: 'payments' }]);
  expect(report.checks[0].grants.map((grant: any) => grant.bindingName)).toEqual(['alice-secrets', 'second-secret-path']);
});

test('@claim:read-only-collection performs no mutating kubectl operation', () => {
  const { calls } = controlledCollection();
  expect(calls.some((call) => /(^| )(create|apply|patch|delete|replace|edit|exec)( |$)/.test(call))).toBe(false);
  expect(calls.filter((call) => call.startsWith('get '))).toHaveLength(4);
});

test('@claim:no-kubeconfig-token does not copy a kubeconfig token into a snapshot', () => {
  const { snapshot } = controlledCollection();
  expect(snapshot).not.toContain('NEVER_INCLUDE_THIS_TOKEN');
  expect(JSON.parse(snapshot)).toMatchObject({ context: 'sample-context', serverVersion: 'v1.33.4' });
});

test('@claim:single-binary-no-agent finishes the demo without installing a cluster component', () => {
  const { directory, result } = runDemo();
  expect(result.status).toBe(0);
  expect(readdirSync(directory).sort()).toEqual(['evidence.json', 'evidence.md', 'matrix.json', 'rbac-snapshot.json']);
  expect(runCli(['demo', '--output', directory], temporaryDirectory('repeat')).status).toBe(1);
  const automatic = runCli(['demo'], temporaryDirectory('automatic'), { ...process.env, PATH: '' });
  expect(automatic.status).toBe(0);
  const automaticPath = automatic.stdout.trim().split('\n').at(-1)?.replace('Sample files: ', '');
  expect(automaticPath).toBeTruthy();
  expect(readdirSync(automaticPath as string).sort()).toEqual(['evidence.json', 'evidence.md', 'matrix.json', 'rbac-snapshot.json']);
  rmSync(automaticPath as string, { recursive: true });
});

test('@claim:no-telemetry runs the CLI sample offline and keeps browser requests first-party', async ({ page }) => {
  const { result } = runDemo();
  expect(result.status).toBe(0);
  const origins = new Set<string>();
  page.on('request', (request) => origins.add(new URL(request.url()).origin));
  await page.goto('/demo/');
  await page.getByLabel('Permission question').selectOption('exec');
  expect([...origins]).toEqual(['http://127.0.0.1:4173']);
});

test('@claim:no-hosted-cluster uses the bundled sample without kubectl or external requests', async ({ page }) => {
  expect(runDemo().result.status).toBe(0);
  const external: string[] = [];
  page.on('request', (request) => {
    if (new URL(request.url()).origin !== 'http://127.0.0.1:4173') external.push(request.url());
  });
  await page.goto('/demo/');
  await page.getByLabel('Permission question').selectOption('health');
  expect(external).toEqual([]);
});

test('@claim:offline-report evaluates a saved snapshot when kubectl is unavailable', () => {
  const directory = temporaryDirectory('offline');
  const result = runCli([
    'report', '--snapshot', join(examples, 'rbac-snapshot.json'), '--subject', 'user:alice@example.com',
    '--as-group', 'platform-engineers', '--matrix', join(examples, 'matrix.json'), '--json',
  ], directory, { ...process.env, PATH: '' });
  expect(result.status).toBe(0);
  expect(JSON.parse(result.stdout).summary).toMatchObject({ total: 4, allowed: 2, denied: 2 });
});

test('@claim:markdown-json-output writes matching Markdown and JSON packet files', () => {
  const directory = temporaryDirectory('formats');
  const output = join(directory, 'audit');
  const result = runCli([
    'report', '--snapshot', join(examples, 'rbac-snapshot.json'), '--subject', 'user:alice@example.com',
    '--as-group', 'platform-engineers', '--matrix', join(examples, 'matrix.json'), '--output', output,
  ], directory);
  expect(result.status).toBe(0);
  expect(JSON.parse(readFileSync(`${output}.json`, 'utf8')).summary.total).toBe(4);
  expect(readFileSync(`${output}.md`, 'utf8')).toContain('**2 allowed**, **2 denied**');
});

test('@claim:json-stdout writes complete JSON to stdout without output files', () => {
  const directory = temporaryDirectory('stdout');
  const before = readdirSync(directory);
  const result = runCli([
    'report', '--snapshot', join(examples, 'rbac-snapshot.json'), '--subject', 'user:alice@example.com',
    '--as-group', 'platform-engineers', '--matrix', join(examples, 'matrix.json'), '--json',
  ], directory);
  expect(result.status).toBe(0);
  expect(JSON.parse(result.stdout).checks).toHaveLength(4);
  expect(readdirSync(directory)).toEqual(before);
});

test('@claim:exit-codes returns 0 for evaluation, 1 for input failure, and 3 for a matched policy', () => {
  const common = [
    'report', '--snapshot', join(examples, 'rbac-snapshot.json'), '--subject', 'user:alice@example.com',
    '--as-group', 'platform-engineers', '--matrix', join(examples, 'matrix.json'), '--json',
  ];
  expect(runCli(common).status).toBe(0);
  expect(runCli(['report', '--snapshot', 'missing.json', '--subject', 'user:a', '--matrix', join(examples, 'matrix.json'), '--json']).status).toBe(1);
  expect(runCli([...common, '--fail-on', 'denied']).status).toBe(3);
});

test('@claim:subject-forms accepts user, group, and service-account subjects', () => {
  const snapshot = baseSnapshot();
  snapshot.roleBindings[0].subjects.push({ kind: 'ServiceAccount', name: 'bot', namespace: 'payments' });
  const check = [{ verb: 'get', resource: 'secrets', namespace: 'payments' }];
  expect(evaluateSnapshot(snapshot, check, 'user:alice@example.com').result.status).toBe(0);
  expect(evaluateSnapshot(snapshot, [{ verb: 'list', apiGroup: 'apps', resource: 'deployments', namespace: 'payments' }], 'group:platform-engineers').report.summary.allowed).toBe(1);
  expect(evaluateSnapshot(snapshot, check, 'serviceaccount:payments:bot').report.summary.allowed).toBe(1);
});

test('@claim:service-account-groups derives standard Kubernetes service-account groups', () => {
  const snapshot = baseSnapshot();
  snapshot.roleBindings[0].subjects = [{ kind: 'Group', name: 'system:serviceaccounts:payments' }];
  const { report } = evaluateSnapshot(snapshot, [{ verb: 'get', resource: 'secrets', namespace: 'payments' }], 'serviceaccount:payments:bot');
  expect(report.summary.allowed).toBe(1);
  expect(report.evaluatedGroups).toEqual(expect.arrayContaining(['system:authenticated', 'system:serviceaccounts', 'system:serviceaccounts:payments']));
});

test('@claim:rolebinding-role explains a RoleBinding to Role grant', () => {
  const { report } = runDemo();
  expect(report.checks[0]).toMatchObject({ allowed: true, grants: [{ bindingKind: 'RoleBinding', roleKind: 'Role', roleName: 'secret-reader' }] });
});

test('@claim:rolebinding-clusterrole-namespace limits a RoleBinding to its namespace', () => {
  const snapshot = baseSnapshot();
  snapshot.roleBindings = [{
    kind: 'RoleBinding', metadata: { name: 'payments-view', namespace: 'payments' },
    subjects: [{ kind: 'User', name: 'alice@example.com' }],
    roleRef: { kind: 'ClusterRole', name: 'deployment-viewer', apiGroup: 'rbac.authorization.k8s.io' },
  }];
  snapshot.clusterRoleBindings = [];
  const { report } = evaluateSnapshot(snapshot, [
    { verb: 'list', apiGroup: 'apps', resource: 'deployments', namespace: 'payments' },
    { verb: 'list', apiGroup: 'apps', resource: 'deployments', namespace: 'staging' },
  ]);
  expect(report.checks.map((check: any) => check.allowed)).toEqual([true, false]);
});

test('@claim:clusterrolebinding-clusterwide grants a ClusterRole across namespaces', () => {
  const snapshot = baseSnapshot();
  const { report } = evaluateSnapshot(snapshot, [
    { verb: 'list', apiGroup: 'apps', resource: 'deployments', namespace: 'payments' },
    { verb: 'list', apiGroup: 'apps', resource: 'deployments', namespace: 'staging' },
  ], 'user:alice@example.com', ['platform-engineers']);
  expect(report.checks.map((check: any) => check.allowed)).toEqual([true, true]);
});

test('@claim:wildcard-matching handles exact and wildcard verbs, groups, and resources', () => {
  const snapshot = baseSnapshot();
  snapshot.clusterRoles = [{ kind: 'ClusterRole', metadata: { name: 'wildcard' }, rules: [{ apiGroups: ['*'], resources: ['*'], verbs: ['*'] }] }];
  snapshot.clusterRoleBindings = [{ kind: 'ClusterRoleBinding', metadata: { name: 'wildcard' }, subjects: [{ kind: 'User', name: 'alice@example.com' }], roleRef: { kind: 'ClusterRole', name: 'wildcard' } }];
  snapshot.roleBindings = [];
  const { report } = evaluateSnapshot(snapshot, [{ verb: 'patch', apiGroup: 'apps', resource: 'deployments', namespace: 'payments' }]);
  expect(report.summary.allowed).toBe(1);
  expect(report.checks[0].grants[0].rule).toMatchObject({ apiGroups: ['*'], resources: ['*'], verbs: ['*'] });
});

test('@claim:subresource-matching supports exact and wildcard subresources without matching the wrong target', () => {
  const snapshot = baseSnapshot();
  snapshot.clusterRoles = [{ kind: 'ClusterRole', metadata: { name: 'scale' }, rules: [{ apiGroups: ['apps'], resources: ['*/scale'], verbs: ['update'] }] }];
  snapshot.clusterRoleBindings = [{ kind: 'ClusterRoleBinding', metadata: { name: 'scale' }, subjects: [{ kind: 'User', name: 'alice@example.com' }], roleRef: { kind: 'ClusterRole', name: 'scale' } }];
  snapshot.roleBindings = [];
  const { report } = evaluateSnapshot(snapshot, [
    { verb: 'update', apiGroup: 'apps', resource: 'deployments', subresource: 'scale', namespace: 'payments' },
    { verb: 'update', apiGroup: 'apps', resource: 'deployments', subresource: 'status', namespace: 'payments' },
  ]);
  expect(report.checks.map((check: any) => check.allowed)).toEqual([true, false]);
});

test('@claim:resource-name-semantics follows create and named list rules', () => {
  const snapshot = baseSnapshot();
  snapshot.clusterRoles = [{ kind: 'ClusterRole', metadata: { name: 'named' }, rules: [{ apiGroups: [''], resources: ['pods'], verbs: ['create', 'get', 'list'], resourceNames: ['approved'] }] }];
  snapshot.clusterRoleBindings = [{ kind: 'ClusterRoleBinding', metadata: { name: 'named' }, subjects: [{ kind: 'User', name: 'alice@example.com' }], roleRef: { kind: 'ClusterRole', name: 'named' } }];
  snapshot.roleBindings = [];
  const { report } = evaluateSnapshot(snapshot, [
    { verb: 'create', resource: 'pods', namespace: 'payments', resourceName: 'approved' },
    { verb: 'list', resource: 'pods', namespace: 'payments', resourceName: 'approved' },
    { verb: 'list', resource: 'pods', namespace: 'payments', resourceName: 'approved', fieldSelector: 'metadata.name=approved' },
    { verb: 'get', resource: 'pods', namespace: 'payments', resourceName: 'approved' },
  ]);
  expect(report.checks.map((check: any) => check.allowed)).toEqual([false, false, true, true]);
});

test('@claim:non-resource-url matches exact and trailing-wildcard URL rules without accepting a RoleBinding grant', () => {
  const snapshot = baseSnapshot();
  snapshot.clusterRoles = [{ kind: 'ClusterRole', metadata: { name: 'health' }, rules: [{ nonResourceURLs: ['/healthz/*'], verbs: ['get'] }] }];
  snapshot.clusterRoleBindings = [{ kind: 'ClusterRoleBinding', metadata: { name: 'health' }, subjects: [{ kind: 'User', name: 'alice@example.com' }], roleRef: { kind: 'ClusterRole', name: 'health' } }];
  snapshot.roleBindings = [{
    kind: 'RoleBinding', metadata: { name: 'payments-health', namespace: 'payments' },
    subjects: [{ kind: 'User', name: 'alice@example.com' }],
    roleRef: { kind: 'ClusterRole', name: 'health' },
  }];
  const { report } = evaluateSnapshot(snapshot, [
    { verb: 'get', nonResourceURL: '/healthz/ready' },
    { verb: 'get', nonResourceURL: '/version' },
  ]);
  expect(report.checks.map((check: any) => check.allowed)).toEqual([true, false]);

  const packagedCli = installPackagedCli();
  const directory = temporaryDirectory('non-resource-package');
  const snapshotPath = join(directory, 'snapshot.json');
  const matrixPath = join(directory, 'matrix.json');
  writeFileSync(snapshotPath, JSON.stringify(snapshot));
  writeFileSync(matrixPath, JSON.stringify({ checks: [{ verb: 'get', nonResourceURL: '/healthz/ready' }] }));
  const allowed = runBinary(packagedCli, [
    'report', '--snapshot', snapshotPath, '--subject', 'user:alice@example.com', '--matrix', matrixPath, '--json',
  ], directory);
  expect(allowed.status).toBe(0);
  expect(JSON.parse(allowed.stdout).checks[0].allowed).toBe(true);

  snapshot.clusterRoleBindings = [];
  writeFileSync(snapshotPath, JSON.stringify(snapshot));
  const roleBindingOnly = runBinary(packagedCli, [
    'report', '--snapshot', snapshotPath, '--subject', 'user:alice@example.com', '--matrix', matrixPath, '--json',
  ], directory);
  expect(roleBindingOnly.status).toBe(0);
  expect(JSON.parse(roleBindingOnly.stdout).checks[0].allowed).toBe(false);

  writeFileSync(matrixPath, JSON.stringify({ checks: [{ verb: 'get', nonResourceURL: '/healthz/ready', namespace: 'payments' }] }));
  const invalid = runBinary(packagedCli, [
    'report', '--snapshot', snapshotPath, '--subject', 'user:alice@example.com', '--matrix', matrixPath, '--json',
  ], directory);
  expect(invalid.status).toBe(1);
  expect(invalid.stderr).toContain('nonResourceURL checks cannot set');
});

test('@claim:aggregation-uncertain marks grants from aggregated ClusterRoles uncertain', () => {
  const snapshot = baseSnapshot();
  snapshot.clusterRoles[0].aggregationRule = { clusterRoleSelectors: [{ matchLabels: { 'rbac.example/view': 'true' } }] };
  const { report } = evaluateSnapshot(snapshot, [{ verb: 'list', apiGroup: 'apps', resource: 'deployments', namespace: 'payments' }], 'user:alice@example.com', ['platform-engineers']);
  expect(report.checks[0]).toMatchObject({ allowed: true, uncertain: true });
  expect(report.checks[0].grants[0].uncertainty).toContain('aggregationRule');
});

test('@claim:server-version-recorded includes the collected Kubernetes version', () => {
  expect(runDemo().report.source.serverVersion).toBe('v1.33.4');
});

test('@claim:limitations-recorded names authorizer and identity-provider limits in the packet', () => {
  const limitations = runDemo().report.limitations.join(' ');
  expect(limitations).toContain('webhook, Node, and other authorizers');
  expect(limitations).toContain('External identity-provider group membership');
});

test('@claim:token-free-snapshot creates a reusable credential-free snapshot', () => {
  const { snapshot, status } = controlledCollection();
  expect(status).toBe(0);
  const parsed = JSON.parse(snapshot);
  expect(parsed.schemaVersion).toBe('kpe.snapshot/v1');
  expect(snapshot).not.toContain('token:');
  expect(parsed.roles).toEqual([]);
});

test('@claim:key-safety creates an owner-only key and refuses overwrite', () => {
  const directory = temporaryDirectory('key');
  const key = join(directory, 'audit.key');
  expect(runCli(['keygen', '--output', key], directory).status).toBe(0);
  expect(statSync(key).mode & 0o777).toBe(0o600);
  const first = readFileSync(key, 'utf8');
  expect(runCli(['keygen', '--output', key], directory).status).toBe(1);
  expect(readFileSync(key, 'utf8')).toBe(first);
});

test('@claim:signature-trust verifies content and rejects a different trusted signer', () => {
  const directory = temporaryDirectory('sign');
  const key = join(directory, 'audit.key');
  const pub = join(directory, 'audit.pub');
  const otherKey = join(directory, 'other.key');
  const otherPub = join(directory, 'other.pub');
  expect(runCli(['keygen', '--output', key, '--public-key-output', pub], directory).status).toBe(0);
  expect(runCli(['keygen', '--output', otherKey, '--public-key-output', otherPub], directory).status).toBe(0);
  const output = join(directory, 'evidence');
  expect(runCli([
    'report', '--snapshot', join(examples, 'rbac-snapshot.json'), '--subject', 'user:alice@example.com',
    '--as-group', 'platform-engineers', '--matrix', join(examples, 'matrix.json'), '--output', output, '--signing-key', key,
  ], directory).status).toBe(0);
  const selfCheck = runCli(['verify', `${output}.json`, '--json'], directory);
  expect(JSON.parse(selfCheck.stdout)).toMatchObject({ valid: true, trustedSigner: false });
  const valid = runCli(['verify', `${output}.json`, '--trusted-public-key', pub, '--json'], directory);
  expect(valid.status).toBe(0);
  expect(JSON.parse(valid.stdout)).toMatchObject({ valid: true, trustedSigner: true });
  expect(JSON.parse(valid.stdout).signerFingerprint).toMatch(/^SHA256:[0-9a-f]{64}$/);
  expect(runCli(['verify', `${output}.json`, '--trusted-public-key', otherPub], directory).status).toBe(1);
  const changed = JSON.parse(readFileSync(`${output}.json`, 'utf8'));
  changed.checks[0].displayVerdict = 'DENIED';
  const changedPath = join(directory, 'changed.json');
  writeFileSync(changedPath, JSON.stringify(changed));
  expect(runCli(['verify', changedPath, '--trusted-public-key', pub], directory).status).toBe(1);
});

test('@claim:least-privilege-collection succeeds with only RBAC reads and version queries', () => {
  const { calls, status } = controlledCollection();
  expect(status).toBe(0);
  expect(calls.every((call) => call.startsWith('get ') || call.startsWith('version ') || call.startsWith('config current-context'))).toBe(true);
});

test('@claim:isolated-browser-demo uses fictional data and preserves real storage', async ({ page }) => {
  await page.goto('/');
  await page.evaluate(() => localStorage.setItem('real:data', 'unchanged'));
  await page.goto('/demo/');
  await page.getByLabel('Permission question').selectOption('exec');
  await expect(page.getByText('0 matching grant paths')).toBeVisible();
  expect(await page.evaluate(() => localStorage.getItem('real:data'))).toBe('unchanged');
  expect(await page.evaluate(() => Object.keys(localStorage).filter((key) => key.startsWith('demo:')))).toEqual([]);
  expect(await page.evaluate(() => Object.keys(sessionStorage))).toEqual(['demo:kpe:selected-case']);
});

test('@claim:first-party-site loads the demo without analytics, trackers, CDN fonts, or third-party scripts', async ({ page }) => {
  const requests: string[] = [];
  page.on('request', (request) => requests.push(request.url()));
  await page.goto('/demo/');
  await page.getByLabel('Permission question').selectOption('deploy');
  expect(requests.every((url) => new URL(url).origin === 'http://127.0.0.1:4173')).toBe(true);
  expect(await page.locator('script[src^="http"], link[href^="http"]:not([rel="canonical"])').count()).toBe(0);
});

test('@claim:offline-site reloads the guide and sample while offline', async ({ browser }) => {
  const context = await browser.newContext();
  const page = await context.newPage();
  await page.goto('/');
  await page.evaluate(() => navigator.serviceWorker.ready);
  await page.waitForFunction(() => navigator.serviceWorker.controller !== null);
  await page.reload();
  await context.setOffline(true);
  await page.reload();
  await expect(page.getByText('You are offline.')).toBeVisible();
  await page.goto('/demo/');
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  await page.getByLabel('Permission question').selectOption('health');
  await expect(page.getByText('DENIED', { exact: true })).toBeVisible();
  await context.close();
});

test('@claim:license-storage stores a token locally, verifies once daily, and never caches its URL', async ({ page }) => {
  let calls = 0;
  await page.route('https://api.sociobot.in/api/v1/products/kube-permission-evidence/verify?license=existing-token', async (route) => {
    calls += 1;
    await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ valid: true, reason: 'ok', expires_at: null }) });
  });
  await page.goto('/?license=existing-token');
  await expect(page.getByText('Field Kit license verified on this device.')).toBeVisible();
  await page.reload();
  await expect(page.getByText('Field Kit license loaded from its saved verification.')).toBeVisible();
  expect(calls).toBe(1);
  expect(await page.evaluate(() => localStorage.getItem('sb_license:kube-permission-evidence'))).toBe('existing-token');
  const cached = await page.evaluate(async () => (await Promise.all((await caches.keys()).map(async (name) => (await (await caches.open(name)).keys()).map((request) => request.url)))).flat());
  expect(cached.some((url) => new URL(url).searchParams.has('license'))).toBe(false);
});

test('@claim:sales-paused exposes no checkout or payment action', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByText('Field Kit sales are paused.')).toBeVisible();
  await expect(page.locator('a[href*="/checkout"], button:has-text("Buy")')).toHaveCount(0);
  await expect(page.getByText('No payment is accepted on this site.')).toBeVisible();
});
