type DemoCase = {
  verdict: 'ALLOWED' | 'DENIED';
  tone: 'allowed' | 'denied';
  summary: string;
  trail: Array<{ type: string; name: string; note: string }>;
};

const cases: Record<string, DemoCase> = {
  secret: {
    verdict: 'ALLOWED',
    tone: 'allowed',
    summary: 'One RoleBinding and Role rule grant this request.',
    trail: [
      { type: 'Subject', name: 'User / alice@example.com', note: 'Direct user match' },
      { type: 'RoleBinding', name: 'payments / alice-secrets', note: 'References Role secret-reader' },
      { type: 'Role', name: 'secret-reader / rule 0', note: 'verbs [get] · resources [secrets]' },
      { type: 'Effective access', name: 'get secrets / payments', note: '1 matching grant path' },
    ],
  },
  deploy: {
    verdict: 'ALLOWED',
    tone: 'allowed',
    summary: 'One group binding and ClusterRole rule grant this request.',
    trail: [
      { type: 'Subject', name: 'User / alice@example.com', note: 'Asserted group: platform-engineers' },
      { type: 'ClusterRoleBinding', name: 'platform-viewers', note: 'Matches Group platform-engineers' },
      { type: 'ClusterRole', name: 'deployment-viewer / rule 0', note: 'verbs [get, list, watch]' },
      { type: 'Effective access', name: 'list deployments.apps / payments', note: '1 matching grant path' },
    ],
  },
  exec: {
    verdict: 'DENIED',
    tone: 'denied',
    summary: 'No collected binding and rule grants pod exec creation.',
    trail: [
      { type: 'Subject', name: 'User / alice@example.com', note: 'Direct and group subjects checked' },
      { type: 'Requested resource', name: 'pods/exec / api-worker', note: 'Verb: create · namespace: payments' },
      { type: 'Effective access', name: 'create pods/exec / payments', note: '0 matching grant paths' },
    ],
  },
  health: {
    verdict: 'DENIED',
    tone: 'denied',
    summary: 'No collected non-resource rule grants this request.',
    trail: [
      { type: 'Subject', name: 'User / alice@example.com', note: 'Direct and group subjects checked' },
      { type: 'Requested URL', name: '/healthz/ready', note: 'Verb: get' },
      { type: 'Effective access', name: 'get /healthz/ready', note: '0 matching grant paths' },
    ],
  },
};

const panel = document.querySelector<HTMLElement>('#proof-panel');
const select = document.querySelector<HTMLSelectElement>('#case-select');
const clear = document.querySelector<HTMLButtonElement>('#clear-proof');
const isDemo = document.body.dataset.demo === 'true';
const DEMO_STATE_KEY = 'demo:kpe:selected-case';

function renderCase(key: string): void {
  if (!panel) return;
  const item = cases[key];
  if (!item) {
    panel.className = 'proof-panel empty';
    panel.innerHTML = '<p class="empty-mark" aria-hidden="true">—</p><h3>No result selected</h3><p>Choose a permission question to inspect its evidence.</p>';
    return;
  }
  panel.className = `proof-panel ${item.tone}`;
  panel.innerHTML = `
    <div class="verdict-line"><span class="verdict ${item.tone}">${item.verdict}</span><span>${item.trail.length - 1} evidence steps</span></div>
    <p class="proof-summary">${item.summary}</p>
    <ol class="evidence-trail">
      ${item.trail.map((step, index) => `<li><span class="node">${String(index + 1).padStart(2, '0')}</span><div><span class="node-type">${step.type}</span><strong>${step.name}</strong><small>${step.note}</small></div></li>`).join('')}
    </ol>`;
}

if (select) {
  const stored = isDemo ? sessionStorage.getItem(DEMO_STATE_KEY) : null;
  if (stored && cases[stored]) select.value = stored;
  select.addEventListener('change', () => {
    if (isDemo) sessionStorage.setItem(DEMO_STATE_KEY, select.value);
    renderCase(select.value);
  });
  renderCase(select.value);
}

clear?.addEventListener('click', () => {
  if (select) select.selectedIndex = -1;
  if (isDemo) sessionStorage.setItem(DEMO_STATE_KEY, '');
  renderCase('');
});

const toast = document.querySelector<HTMLElement>('#toast');
const resetDemo = document.querySelector<HTMLButtonElement>('#reset-demo');
resetDemo?.addEventListener('click', () => {
  sessionStorage.removeItem(DEMO_STATE_KEY);
  if (select) {
    select.value = 'secret';
    renderCase(select.value);
    select.focus();
  }
  if (toast) toast.textContent = 'Demo reset to the first sample check.';
});
document.querySelector<HTMLAnchorElement>('#leave-demo')?.addEventListener('click', () => {
  sessionStorage.removeItem(DEMO_STATE_KEY);
});

document.querySelectorAll<HTMLButtonElement>('[data-copy]').forEach((button) => {
  button.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(button.dataset.copy ?? '');
      button.textContent = 'Copied';
      if (toast) toast.textContent = 'Command copied.';
      window.setTimeout(() => {
        button.textContent = 'Copy';
        if (toast) toast.textContent = '';
      }, 1800);
    } catch {
      button.textContent = 'Select command';
      if (toast) toast.textContent = 'Clipboard access was unavailable. Select the command manually.';
    }
  });
});

const recordingFallback = [
  [0.08, 'o', '$ kpe demo\r\n'],
  [0.24, 'o', 'Demo — bundled sample data; no cluster was contacted.\r\n'],
  [0.40, 'o', 'Subject: user:alice@example.com\r\n'],
  [0.56, 'o', 'Result: 4 checks — 2 allowed, 2 denied, 0 uncertain.\r\n'],
  [0.72, 'o', 'Sample files: /tmp/kpe-demo…\r\n'],
] as const;
const recordingOutput = document.querySelector<HTMLElement>('#demo-recording-output code');
document.querySelector<HTMLButtonElement>('#replay-demo')?.addEventListener('click', async (event) => {
  const button = event.currentTarget as HTMLButtonElement;
  if (!recordingOutput) return;
  button.disabled = true;
  recordingOutput.textContent = '';
  const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)').matches;
  let events: ReadonlyArray<readonly [number, string, string]> = recordingFallback;
  try {
    const response = await fetch('/kpe-demo.cast');
    if (response.ok) {
      events = (await response.text()).trim().split('\n').slice(1).map((line) => JSON.parse(line) as [number, string, string]);
    }
  } catch {
    // The built-in transcript keeps replay available during a first-load failure.
  }
  let previous = 0;
  for (const [time, stream, content] of events) {
    if (stream !== 'o') continue;
    if (!reducedMotion) await new Promise((resolve) => window.setTimeout(resolve, Math.max(0, time - previous) * 1000));
    recordingOutput.textContent += content.replaceAll('\r', '');
    previous = time;
  }
  button.disabled = false;
  button.focus();
});

const SLUG = 'kube-permission-evidence';
const API = 'https://api.sociobot.in/api/v1';
const LICENSE_KEY = `sb_license:${SLUG}`;
const VERDICT_KEY = `sb_license_verdict:${SLUG}`;
const DAY = 86_400_000;
const status = document.querySelector<HTMLElement>('#license-status');
const licensedPanel = document.querySelector<HTMLElement>('#unlocked-tools');
const form = document.querySelector<HTMLFormElement>('#license-form');
const tokenInput = document.querySelector<HTMLInputElement>('#license-token');

type CachedVerdict = { token: string; valid: boolean; checkedAt: number; reason: string };

function readVerdict(): CachedVerdict | null {
  try {
    return JSON.parse(localStorage.getItem(VERDICT_KEY) ?? 'null') as CachedVerdict | null;
  } catch {
    return null;
  }
}

function showLicense(valid: boolean, message: string): void {
  if (licensedPanel) licensedPanel.hidden = !valid;
  if (status) {
    status.className = `license-status ${valid ? 'valid' : ''}`;
    status.textContent = message;
  }
}

async function verifyLicense(token: string, force = false): Promise<void> {
  const storedVerdict = readVerdict();
  const cached = storedVerdict?.token === token ? storedVerdict : null;
  if (cached?.valid) showLicense(true, 'Field Kit license loaded from its saved verification.');
  if (!force && cached && Date.now() - cached.checkedAt < DAY) {
    if (!cached.valid) showLicense(false, 'License is not active. Paste another existing token or try later.');
    return;
  }
  if (!navigator.onLine) {
    if (!cached?.valid) showLicense(false, 'You are offline. License verification resumes when you reconnect.');
    return;
  }
  if (status) status.textContent = 'Checking the saved license…';
  try {
    const response = await fetch(`${API}/products/${SLUG}/verify?license=${encodeURIComponent(token)}`, { headers: { accept: 'application/json' } });
    if (!response.ok) throw new Error(`verification returned ${response.status}`);
    const verdict = await response.json() as { valid: boolean; reason: string };
    localStorage.setItem(VERDICT_KEY, JSON.stringify({ token, valid: verdict.valid, reason: verdict.reason, checkedAt: Date.now() }));
    showLicense(verdict.valid, verdict.valid ? 'Field Kit license verified on this device.' : 'License is not active. Paste another existing token or try later.');
  } catch {
    showLicense(Boolean(cached?.valid), cached?.valid ? 'Using the saved result while verification is unavailable.' : 'License verification is unavailable. The free CLI still works.');
  }
}

if (!isDemo && form) {
  const query = new URLSearchParams(location.search);
  const returnedLicense = query.get('license');
  if (returnedLicense) {
    localStorage.setItem(LICENSE_KEY, returnedLicense);
    query.delete('license');
    history.replaceState({}, '', `${location.pathname}${query.size ? `?${query}` : ''}${location.hash}`);
  }
  const savedLicense = returnedLicense ?? localStorage.getItem(LICENSE_KEY);
  if (savedLicense) void verifyLicense(savedLicense);

  form.addEventListener('submit', (event) => {
    event.preventDefault();
    const token = tokenInput?.value.trim();
    if (!token) return;
    localStorage.setItem(LICENSE_KEY, token);
    if (tokenInput) tokenInput.value = '';
    void verifyLicense(token, true);
  });
}

const offlineNote = document.querySelector<HTMLElement>('#offline-note');
function updateConnection(): void {
  if (offlineNote) offlineNote.hidden = navigator.onLine;
}
window.addEventListener('online', () => {
  updateConnection();
  if (!isDemo) {
    const token = localStorage.getItem(LICENSE_KEY);
    if (token) void verifyLicense(token);
  }
});
window.addEventListener('offline', updateConnection);
updateConnection();

if ('serviceWorker' in navigator && import.meta.env.PROD) {
  window.addEventListener('load', () => {
    void navigator.serviceWorker.register('/sw.js');
  });
}
