const deviceGrid = document.getElementById('deviceGrid');
const loadingView = document.getElementById('loadingView');
const emptyView = document.getElementById('emptyView');
const errorView = document.getElementById('errorView');
const stateText = document.getElementById('stateText');

const fileInput = document.getElementById('fileInput');
const chooseFileBtn = document.getElementById('chooseFileBtn');
const dropZone = document.getElementById('dropZone');
const selectedFilesText = document.getElementById('selectedFilesText');
const receiverList = document.getElementById('receiverList');
const confirmSendBtn = document.getElementById('confirmSendBtn');
const confirmText = document.getElementById('confirmText');
const transferList = document.getElementById('transferList');

const incomingModal = document.getElementById('incomingModal');
const incomingMeta = document.getElementById('incomingMeta');
const simulateIncomingBtn = document.getElementById('simulateIncomingBtn');
const acceptBtn = document.getElementById('acceptBtn');
const declineBtn = document.getElementById('declineBtn');

const localFingerprint = document.getElementById('localFingerprint');
const trustStateText = document.getElementById('trustStateText');
const verifyPeerBtn = document.getElementById('verifyPeerBtn');
const revokePeerBtn = document.getElementById('revokePeerBtn');

const lanOnlyToggle = document.getElementById('lanOnlyToggle');
const relayToggle = document.getElementById('relayToggle');
const diagToggle = document.getElementById('diagToggle');
const updateChannelSelect = document.getElementById('updateChannelSelect');
const settingsSummary = document.getElementById('settingsSummary');

const reducedMotionToggle = document.getElementById('reducedMotionToggle');
const highContrastToggle = document.getElementById('highContrastToggle');
const largeTextToggle = document.getElementById('largeTextToggle');

const state = {
  mode: 'loading',
  devices: [],
  selectedFiles: [],
  selectedReceivers: new Set(),
  transfers: [],
  incomingRequest: null,
  trustState: 'unverified'
};

const sampleDevices = [
  { id: 'peer-a', name: 'Aarav iPhone', addr: '192.168.1.12', status: 'online' },
  { id: 'peer-b', name: 'Meera MacBook', addr: '192.168.1.34', status: 'busy' },
  { id: 'peer-c', name: 'Ravi Desktop', addr: '192.168.1.55', status: 'offline' }
];

const backendBase = (window.localStorage.getItem('p2pBackendBase') || 'http://127.0.0.1:8787').replace(/\/$/, '');

function setMode(mode) {
  state.mode = mode;
  loadingView.classList.toggle('hidden', mode !== 'loading');
  emptyView.classList.toggle('hidden', mode !== 'empty');
  errorView.classList.toggle('hidden', mode !== 'error');
  deviceGrid.classList.toggle('hidden', mode !== 'ready');

  if (mode === 'loading') stateText.textContent = 'Discovering nearby devices...';
  if (mode === 'ready') stateText.textContent = `${state.devices.length} device(s) found`;
  if (mode === 'empty') stateText.textContent = 'No nearby devices currently available';
  if (mode === 'error') stateText.textContent = 'Discovery unavailable (start backend_service on :8787)';
}

function renderDevices() {
  deviceGrid.innerHTML = '';
  receiverList.innerHTML = '';

  for (const d of state.devices) {
    const card = document.createElement('article');
    card.className = 'device-card';
    card.innerHTML = `<div class="device-avatar">${d.name[0]}</div><h3>${d.name}</h3><p>${d.addr}</p><span class="badge ${d.status}">${capitalize(d.status)}</span>`;
    deviceGrid.appendChild(card);

    const chip = document.createElement('label');
    chip.className = 'receiver-chip';
    chip.innerHTML = `<input type="checkbox" value="${d.id}" /> ${d.name}`;
    chip.querySelector('input').addEventListener('change', (e) => {
      if (e.target.checked) state.selectedReceivers.add(d.id);
      else state.selectedReceivers.delete(d.id);
      updateConfirmHint();
    });
    receiverList.appendChild(chip);
  }
}

async function runScan() {
  setMode('loading');
  try {
    const response = await fetch(`${backendBase}/api/v1/discovery/devices`, { method: 'GET' });
    if (!response.ok) throw new Error(`backend status ${response.status}`);

    const payload = await response.json();
    const devices = Array.isArray(payload.devices) ? payload.devices : [];
    state.devices = devices.map((d) => ({
      id: d.id,
      name: d.name,
      addr: d.addr,
      status: d.status
    }));

    renderDevices();
    if (!state.devices.length) setMode('empty');
    else setMode('ready');
  } catch (error) {
    console.error('Discovery scan failed:', error);
    state.devices = [];
    renderDevices();
    setMode('error');
  }
}

function handleFiles(files) {
  state.selectedFiles = [...files];
  selectedFilesText.textContent = state.selectedFiles.length
    ? `${state.selectedFiles.length} file(s): ${state.selectedFiles.map((f) => f.name).join(', ')}`
    : 'No files selected';
  updateConfirmHint();
}

function updateConfirmHint() {
  confirmText.textContent = `Ready check: files=${state.selectedFiles.length}, receivers=${state.selectedReceivers.size}`;
}

function confirmSend() {
  if (!state.selectedFiles.length) return (confirmText.textContent = 'Select at least one file before sending.');
  if (!state.selectedReceivers.size) return (confirmText.textContent = 'Select at least one receiver before sending.');

  const transfer = { id: Date.now(), name: state.selectedFiles[0].name, progress: 0, status: 'in-progress' };
  state.transfers.unshift(transfer);
  renderTransfers();
  tickTransfer(transfer.id);
}

function tickTransfer(id) {
  const interval = setInterval(() => {
    const t = state.transfers.find((x) => x.id === id);
    if (!t || t.status !== 'in-progress') return clearInterval(interval);
    t.progress = Math.min(100, t.progress + 20);
    if (t.progress === 100) t.status = 'completed';
    renderTransfers();
    if (t.progress === 100) clearInterval(interval);
  }, 500);
}

function renderTransfers() {
  transferList.innerHTML = '';
  if (!state.transfers.length) return (transferList.innerHTML = '<p class="muted">No active transfers yet.</p>');

  for (const t of state.transfers) {
    const row = document.createElement('div');
    row.className = 'transfer-row';
    row.innerHTML = `
      <div class="transfer-head"><strong>${t.name}</strong><span class="status ${t.status}">${t.status}</span></div>
      <div class="progress"><span style="width:${t.progress}%"></span></div>
      <div class="transfer-actions">
        <button class="btn tiny" data-act="pause">Pause</button>
        <button class="btn tiny" data-act="resume">Resume</button>
        <button class="btn tiny" data-act="cancel">Cancel</button>
      </div>`;

    row.querySelector('[data-act="pause"]').onclick = () => { if (t.status === 'in-progress') { t.status = 'paused'; renderTransfers(); } };
    row.querySelector('[data-act="resume"]').onclick = () => { if (t.status === 'paused') { t.status = 'in-progress'; renderTransfers(); tickTransfer(t.id); } };
    row.querySelector('[data-act="cancel"]').onclick = () => { t.status = 'failed'; renderTransfers(); };

    transferList.appendChild(row);
  }
}

function showIncomingRequest() {
  state.incomingRequest = { from: 'Aarav iPhone', fileName: 'holiday_photos.zip', size: '128 MB' };
  incomingMeta.textContent = `${state.incomingRequest.from} wants to send ${state.incomingRequest.fileName} (${state.incomingRequest.size})`;
  incomingModal.classList.remove('hidden');
}

function closeIncomingRequest(result) {
  if (state.incomingRequest) confirmText.textContent = `Incoming request ${result}: ${state.incomingRequest.fileName}`;
  state.incomingRequest = null;
  incomingModal.classList.add('hidden');
}

function updateTrustState(trusted) {
  state.trustState = trusted ? 'trusted' : 'unverified';
  trustStateText.textContent = trusted ? 'Trust state: Trusted peer' : 'Trust state: Unverified peer';
}

function updateSettingsSummary() {
  const mode = lanOnlyToggle.checked ? 'LAN-only' : 'Mixed-network';
  const relay = relayToggle.checked ? 'relay-on' : 'relay-off';
  const diag = diagToggle.checked ? 'diag-on' : 'diag-off';
  settingsSummary.textContent = `Mode: ${mode}, Channel: ${updateChannelSelect.value}, ${relay}, ${diag}`;
}

function updateAccessibilityClasses() {
  document.body.classList.toggle('reduced-motion', reducedMotionToggle.checked);
  document.body.classList.toggle('high-contrast', highContrastToggle.checked);
  document.body.classList.toggle('large-text', largeTextToggle.checked);
}

function capitalize(s) { return s.charAt(0).toUpperCase() + s.slice(1); }

for (const btn of document.querySelectorAll('[data-state]')) {
  btn.addEventListener('click', () => {
    const mode = btn.getAttribute('data-state');
    if (mode === 'ready') { state.devices = [...sampleDevices]; renderDevices(); }
    if (mode === 'empty') { state.devices = []; deviceGrid.innerHTML = ''; receiverList.innerHTML = ''; state.selectedReceivers.clear(); }
    setMode(mode);
  });
}

document.getElementById('retryBtn').onclick = runScan;
document.getElementById('scanBtn').onclick = runScan;
chooseFileBtn.onclick = () => fileInput.click();
fileInput.onchange = (e) => handleFiles(e.target.files || []);
confirmSendBtn.onclick = confirmSend;
simulateIncomingBtn.onclick = showIncomingRequest;
acceptBtn.onclick = () => closeIncomingRequest('accepted');
declineBtn.onclick = () => closeIncomingRequest('declined');
verifyPeerBtn.onclick = () => updateTrustState(true);
revokePeerBtn.onclick = () => updateTrustState(false);

lanOnlyToggle.onchange = updateSettingsSummary;
relayToggle.onchange = updateSettingsSummary;
diagToggle.onchange = updateSettingsSummary;
updateChannelSelect.onchange = updateSettingsSummary;
reducedMotionToggle.onchange = updateAccessibilityClasses;
highContrastToggle.onchange = updateAccessibilityClasses;
largeTextToggle.onchange = updateAccessibilityClasses;

dropZone.addEventListener('dragover', (e) => { e.preventDefault(); dropZone.classList.add('dragover'); });
dropZone.addEventListener('dragleave', () => dropZone.classList.remove('dragover'));
dropZone.addEventListener('drop', (e) => { e.preventDefault(); dropZone.classList.remove('dragover'); handleFiles(e.dataTransfer?.files || []); });

dropZone.addEventListener('keydown', (e) => { if (e.key === 'Enter' || e.key === ' ') fileInput.click(); });

localFingerprint.textContent = 'FA:13:7B:2C:90:AA:45:99';
updateSettingsSummary();
updateAccessibilityClasses();
runScan();
renderTransfers();
