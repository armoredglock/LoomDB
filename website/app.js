// ==============================================================================
// LoomDB Client Runtime & Interactive Engine
// ==============================================================================

document.addEventListener('DOMContentLoaded', () => {
  initOSDetection();
  initTerminal();
  initCopyButtons();
});

// 1. OS & Architecture Detection
function initOSDetection() {
  const ua = window.navigator.userAgent.toLowerCase();
  const platform = window.navigator.platform ? window.navigator.platform.toLowerCase() : '';

  let detectedOS = 'Windows (x64)';
  let filename = 'loom-v0.1.0-windows-x64.zip';
  let installCmd = 'irm https://loomdb.dev/install.ps1 | iex';
  let activeCardId = 'os-win';

  if (ua.includes('mac') || platform.includes('mac')) {
    // Check for Apple Silicon or Intel
    const isArm = ua.includes('arm') || (navigator.userAgentData && navigator.userAgentData.architecture === 'arm');
    if (isArm) {
      detectedOS = 'macOS (Apple Silicon M1/M2/M3/M4)';
      filename = 'loom-v0.1.0-darwin-arm64.tar.gz';
      activeCardId = 'os-mac-arm';
    } else {
      detectedOS = 'macOS (Intel x64)';
      filename = 'loom-v0.1.0-darwin-x64.tar.gz';
      activeCardId = 'os-mac-intel';
    }
    installCmd = 'curl -fsSL https://loomdb.dev/install.sh | sh';
  } else if (ua.includes('linux') || platform.includes('linux')) {
    const isArm = ua.includes('aarch64') || ua.includes('arm');
    if (isArm) {
      detectedOS = 'Linux (ARM64)';
      filename = 'loom-v0.1.0-linux-arm64.tar.gz';
      activeCardId = 'os-linux-arm';
    } else {
      detectedOS = 'Linux (x86_64)';
      filename = 'loom-v0.1.0-linux-x64.tar.gz';
      activeCardId = 'os-linux-x64';
    }
    installCmd = 'curl -fsSL https://loomdb.dev/install.sh | sh';
  }

  // Update UI Elements
  const osLabel = document.getElementById('detectedOSName');
  const mainBtn = document.getElementById('mainDownloadBtn');
  const cmdElem = document.getElementById('quickInstallCmd');

  if (osLabel) osLabel.textContent = detectedOS;
  if (mainBtn) {
    mainBtn.href = `./downloads/${filename}`;
    mainBtn.innerHTML = `<span>⬇ Download LoomDB for ${detectedOS.split(' ')[0]}</span> <span style="opacity: 0.7; font-size: 0.75rem;">(v0.1.0)</span>`;
  }
  if (cmdElem) cmdElem.textContent = installCmd;

  // Highlight active matching card
  const activeCard = document.getElementById(activeCardId);
  if (activeCard) {
    document.querySelectorAll('.os-card').forEach(c => c.classList.remove('active-match'));
    activeCard.classList.add('active-match');
  }
}

// 2. Interactive Terminal Emulator
const terminalPresets = {
  vector: {
    query: `SELECT id, topic, notes, 
       cosine_similarity(embedding, [0.12, 0.45, -0.22, 0.81]) AS score
FROM memories
WHERE topic LIKE '%Rust%'
ORDER BY score DESC
LIMIT 2;`,
    output: `┌────────┬──────────────────────────────┬──────────────────────────┬────────┐
│ id     │ topic                        │ notes                    │ score  │
├────────┼──────────────────────────────┼──────────────────────────┼────────┤
│ mem_01 │ High-Performance Rust Storage│ Micro profile <512KB RAM │ 0.9842 │
│ mem_02 │ Fused HNSW SIMD Traversal    │ Single-pass filter search│ 0.9120 │
└────────┴──────────────────────────────┴──────────────────────────┴────────┘
2 rows in set. Execution time: 0.38 ms (SIMD fused)`
  },
  document: {
    query: `SELECT id, profile->'author'->>'name' AS author, profile->'tags' AS tags
FROM articles
WHERE 'ai' IN profile->'tags'
LIMIT 2;`,
    output: `┌────────┬────────────────────┬─────────────────────────────┐
│ id     │ author             │ tags                        │
├────────┼────────────────────┼─────────────────────────────┤
│ art_01 │ Maya Lin           │ ["ai", "storage", "crdt"]   │
│ art_04 │ Alex Thorne        │ ["ai", "vector", "rust"]    │
└────────┴────────────────────┴─────────────────────────────┘
2 rows in set. Execution time: 0.42 ms`
  },
  timetravel: {
    query: `SELECT balance FROM user_accounts 
AS OF TIMESTAMP '2026-11-01T00:00:00Z'
WHERE account_id = 'acc_4921';`,
    output: `┌───────────────────┬──────────────┐
│ account_id        │ balance      │
├───────────────────┼──────────────┤
│ acc_4921          │ $14,250.00   │
└───────────────────┴──────────────┘
Snapshot: v18492 (Zero-copy CoW MVCC branch). Execution time: 0.29 ms`
  },
  live: {
    query: `LIVE SELECT order_id, customer, total_usd 
FROM orders 
WHERE status = 'pending';`,
    output: `● Continuous Subscription Active (Stream ID: 0x4f1a)
[WAL CDC +0.02s] INSERT -> {"order_id": "ord_881", "customer": "Dana", "total_usd": 128.50}
[WAL CDC +1.14s] UPDATE -> {"order_id": "ord_881", "status": "processing"}
Watching WAL commits with zero polling overhead...`
  }
};

function initTerminal() {
  const terminalBody = document.getElementById('terminalBody');
  const termInput = document.getElementById('termInput');
  const presetChips = document.querySelectorAll('.preset-chip');

  presetChips.forEach(chip => {
    chip.addEventListener('click', () => {
      presetChips.forEach(c => c.classList.remove('active'));
      chip.classList.add('active');
      const presetKey = chip.getAttribute('data-preset');
      const preset = terminalPresets[presetKey];
      if (preset) {
        runTerminalQuery(preset.query, preset.output);
      }
    });
  });

  if (termInput) {
    termInput.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        const val = termInput.value.trim();
        if (val) {
          handleManualInput(val);
          termInput.value = '';
        }
      }
    });
  }
}

function runTerminalQuery(query, outputText) {
  const terminalBody = document.getElementById('terminalBody');
  const logDiv = document.createElement('div');
  logDiv.className = 'term-log';

  logDiv.innerHTML = `
<div style="color: #1cd8a4; font-weight: 700; margin-bottom: 4px;">loom&gt; <span style="color: #fff; font-weight: normal;">${escapeHtml(query)}</span></div>
<div style="color: #8b949e; line-height: 1.5;">${escapeHtml(outputText)}</div>
`;
  terminalBody.appendChild(logDiv);
  terminalBody.scrollTop = terminalBody.scrollHeight;
}

function handleManualInput(input) {
  if (input === '.help') {
    runTerminalQuery(input, `LoomDB Dot-Commands:
  .tables     List tables & collections in single .loom file
  .stats      Show virtual pages, WAL ring, and RAM consumption
  .studio     Export and view Web Studio UI
  .exit       Exit terminal`);
    return;
  } else if (input === '.tables') {
    runTerminalQuery(input, `┌──────────────┬─────────────┬─────────────┐
│ Name         │ Type        │ Engine      │
├──────────────┼─────────────┼─────────────┤
│ memories     │ Multi-Modal │ HNSW + JSON │
│ users        │ Relational  │ B+Tree      │
│ telemetry    │ Time-Series │ Gorilla XOR │
└──────────────┴─────────────┴─────────────┘`);
    return;
  } else if (input === '.stats') {
    runTerminalQuery(input, `Container: ./data.loom (40,960 bytes)
Allocated Virtual Pages: 10 pages (4KB/page)
Active In-Container WAL: Ring buffer (1 MB)
RAM Overhead: 482 KB (Micro Profile)`);
    return;
  }

  // Generic query simulation
  const start = performance.now();
  setTimeout(() => {
    const elapsed = (performance.now() - start).toFixed(2);
    runTerminalQuery(input, `Query OK. (0.34 ms)
[LoomStore] 1 record processed.`);
  }, 80);
}

// 3. Copy to Clipboard & Toast
function initCopyButtons() {
  document.querySelectorAll('.btn-copy-cmd').forEach(btn => {
    btn.addEventListener('click', () => {
      const targetId = btn.getAttribute('data-target');
      const elem = document.getElementById(targetId);
      if (elem) {
        navigator.clipboard.writeText(elem.textContent.trim()).then(() => {
          showToast('Copied installation command to clipboard!');
        });
      }
    });
  });
}

function showToast(msg) {
  let toast = document.querySelector('.toast');
  if (!toast) {
    toast = document.createElement('div');
    toast.className = 'toast';
    document.body.appendChild(toast);
  }
  toast.textContent = msg;
  toast.classList.add('show');
  setTimeout(() => {
    toast.classList.remove('show');
  }, 2500);
}

function escapeHtml(string) {
  return String(string)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}
