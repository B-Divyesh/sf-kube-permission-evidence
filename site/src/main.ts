type DemoCase = {
  verdict: 'ALLOWED' | 'DENIED' | 'ALLOWED · UNCERTAIN';
  tone: 'allowed' | 'denied' | 'uncertain';
  summary: string;
  trail: Array<{ type: string; name: string; note: string }>;
};

const cases: Record<string, DemoCase> = {
  secret: {
    verdict: 'ALLOWED', tone: 'allowed',
    summary: 'Two independent rules grant this request. Both belong in the packet.',
    trail: [
      { type: 'Subject', name: 'User / alice@example.com', note: 'Group asserted: platform-engineers' },
      { type: 'RoleBinding', name: 'payments / secret-auditors', note: 'Matches Group platform-engineers' },
      { type: 'ClusterRole', name: 'secret-reader · rule 0', note: 'verbs [get] · resources [secrets]' },
      { type: 'Effective access', name: 'get secrets / payments', note: '2 grant paths preserved' },
    ],
  },
  deploy: {
    verdict: 'DENIED', tone: 'denied',
    summary: 'No collected binding and rule matches the update verb in this namespace.',
    trail: [
      { type: 'Subject', name: 'User / alice@example.com', note: 'Direct and group subjects examined' },
      { type: 'Candidate rule', name: 'deployment-viewer · rule 1', note: 'verbs [get, list, watch] — update absent' },
      { type: 'Effective access', name: 'update deployments.apps / payments', note: '0 matching grant paths' },
    ],
  },
  aggregate: {
    verdict: 'ALLOWED · UNCERTAIN', tone: 'uncertain',
    summary: 'A resolved aggregation rule grants access, so the packet flags version-sensitive uncertainty.',
    trail: [
      { type: 'Subject', name: 'ServiceAccount / ops:reconciler', note: 'Derived group: system:serviceaccounts:ops' },
      { type: 'ClusterRoleBinding', name: 'workload-observers', note: 'Matches service-account group' },
      { type: 'Aggregated ClusterRole', name: 'workload-view · rule 4', note: 'Controller-resolved on Kubernetes v1.33.4' },
      { type: 'Effective access', name: 'get jobs.batch / all namespaces', note: '1 uncertain grant path' },
    ],
  },
};

const panel = document.querySelector<HTMLElement>('#proof-panel');
const select = document.querySelector<HTMLSelectElement>('#case-select');
const clear = document.querySelector<HTMLButtonElement>('#clear-proof');

function renderCase(key: string): void {
  if (!panel) return;
  const item = cases[key];
  if (!item) {
    panel.className = 'proof-panel empty';
    panel.innerHTML = '<p class="empty-mark" aria-hidden="true">⌁</p><h3>No specimen selected</h3><p>Choose a permission question above to inspect its evidence trail.</p>';
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

select?.addEventListener('change', () => renderCase(select.value));
clear?.addEventListener('click', () => {
  if (select) select.selectedIndex = -1;
  renderCase('');
});
renderCase(select?.value ?? 'secret');

const toast = document.querySelector<HTMLElement>('#toast');
document.querySelectorAll<HTMLButtonElement>('[data-copy]').forEach((button) => {
  button.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(button.dataset.copy ?? '');
      button.textContent = 'Copied';
      if (toast) toast.textContent = 'Install command copied.';
      window.setTimeout(() => { button.textContent = 'Copy'; if (toast) toast.textContent = ''; }, 1800);
    } catch {
      button.textContent = 'Select command';
      if (toast) toast.textContent = 'Clipboard access was unavailable. Select the command manually.';
    }
  });
});

const SLUG = 'kube-permission-evidence';
const API = 'https://api.sociobot.in/api/v1';
const LICENSE_KEY = `sb_license:${SLUG}`;
const VERDICT_KEY = `sb_license_verdict:${SLUG}`;
const DAY = 86_400_000;
const status = document.querySelector<HTMLElement>('#license-status');
const unlocked = document.querySelector<HTMLElement>('#unlocked-tools');
const form = document.querySelector<HTMLFormElement>('#license-form');
const tokenInput = document.querySelector<HTMLInputElement>('#license-token');

type CachedVerdict = { token: string; valid: boolean; checkedAt: number; reason: string };

function readVerdict(): CachedVerdict | null {
  try { return JSON.parse(localStorage.getItem(VERDICT_KEY) ?? 'null') as CachedVerdict | null; }
  catch { return null; }
}

function showLicense(valid: boolean, message: string): void {
  if (unlocked) unlocked.hidden = !valid;
  if (status) {
    status.className = `license-status ${valid ? 'valid' : ''}`;
    status.textContent = message;
  }
}

async function verifyLicense(token: string, force = false): Promise<void> {
  const storedVerdict = readVerdict();
  // A verdict is valid only for the exact token sent to the verification API.
  // Legacy verdicts without `token` and verdicts for a replaced token are ignored.
  const cached = storedVerdict?.token === token ? storedVerdict : null;
  if (cached?.valid) showLicense(true, 'Field kit unlocked from your saved license.');
  if (!force && cached && Date.now() - cached.checkedAt < DAY) {
    if (!cached.valid) showLicense(false, 'License no longer active. You can restore another token; new sales are paused.');
    return;
  }
  if (!navigator.onLine) {
    if (!cached?.valid) showLicense(false, 'Offline — license verification will resume when you reconnect.');
    return;
  }
  if (status) status.textContent = 'Checking saved license…';
  try {
    const response = await fetch(`${API}/products/${SLUG}/verify?license=${encodeURIComponent(token)}`, { headers: { accept: 'application/json' } });
    if (!response.ok) throw new Error(`verification returned ${response.status}`);
    const verdict = await response.json() as { valid: boolean; reason: string };
    localStorage.setItem(VERDICT_KEY, JSON.stringify({ token, valid: verdict.valid, reason: verdict.reason, checkedAt: Date.now() }));
    showLicense(verdict.valid, verdict.valid ? 'Field kit license verified on this device.' : 'License no longer active. You can restore another token; new sales are paused.');
  } catch {
    showLicense(Boolean(cached?.valid), cached?.valid ? 'Using your saved license while verification is unavailable.' : 'Could not reach license verification. The free CLI remains available; try again when online.');
  }
}

const query = new URLSearchParams(location.search);
const returnedLicense = query.get('license');
if (returnedLicense) {
  localStorage.setItem(LICENSE_KEY, returnedLicense);
  query.delete('license');
  history.replaceState({}, '', `${location.pathname}${query.size ? `?${query}` : ''}${location.hash}`);
}
const savedLicense = returnedLicense ?? localStorage.getItem(LICENSE_KEY);
if (savedLicense) void verifyLicense(savedLicense);

form?.addEventListener('submit', (event) => {
  event.preventDefault();
  const token = tokenInput?.value.trim();
  if (!token) return;
  localStorage.setItem(LICENSE_KEY, token);
  if (tokenInput) tokenInput.value = '';
  void verifyLicense(token, true);
});

const offlineNote = document.querySelector<HTMLElement>('#offline-note');
function updateConnection(): void { if (offlineNote) offlineNote.hidden = navigator.onLine; }
window.addEventListener('online', () => { updateConnection(); const token = localStorage.getItem(LICENSE_KEY); if (token) void verifyLicense(token); });
window.addEventListener('offline', updateConnection);
updateConnection();

if ('serviceWorker' in navigator && import.meta.env.PROD) {
  window.addEventListener('load', () => { void navigator.serviceWorker.register('/sw.js'); });
}
