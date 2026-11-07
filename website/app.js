// ==============================================================================
// LoomDB Client Runtime & Interactive Motion Engine
// ==============================================================================

document.addEventListener('DOMContentLoaded', () => {
  initScrollAnimations();
  initOSSelection();
  initTerminalWithQuota();
  initInstallerDialog();
  initCopyButtons();
});

// 1. Reading Progress Bar & Intersection Reveal Animations
function initScrollAnimations() {
  const progressBar = document.getElementById('scrollProgressBar');

  window.addEventListener('scroll', () => {
    const scrollTop = window.scrollY || document.documentElement.scrollTop;
    const scrollHeight = document.documentElement.scrollHeight - document.documentElement.clientHeight;
    if (scrollHeight > 0 && progressBar) {
      const progress = (scrollTop / scrollHeight) * 100;
      progressBar.style.width = `${progress}%`;
    }
  }, { passive: true });

  const revealElements = document.querySelectorAll('.reveal-on-scroll');
  const observerOptions = {
    threshold: 0.1,
    rootMargin: '0px 0px -40px 0px'
  };

  const revealObserver = new IntersectionObserver((entries, observer) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        entry.target.classList.add('is-revealed');
        observer.unobserve(entry.target);
      }
    });
  }, observerOptions);

  revealElements.forEach(el => revealObserver.observe(el));
}

// 2. Platform Detection & Interactive OS Card Switching
const PLATFORM_CONFIGS = {
  win: {
    name: 'Windows (64-bit)',
    filename: 'loom-setup.exe',
    cmd: 'irm https://loomdb.dev/install.ps1 | iex',
    cardId: 'os-win',
  },
  mac: {
    name: 'macOS (Apple Silicon & Intel)',
    filename: 'loom-setup-macos.sh',
    cmd: 'curl -fsSL https://loomdb.dev/install.sh | sh',
    cardId: 'os-mac-arm',
  },
  linux: {
    name: 'Linux (x86_64 & ARM64)',
    filename: 'loom-setup-linux.sh',
    cmd: 'curl -fsSL https://loomdb.dev/install.sh | sh',
    cardId: 'os-linux-x64',
  }
};

function setPlatform(key) {
  const cfg = PLATFORM_CONFIGS[key] || PLATFORM_CONFIGS.win;

  const osLabel = document.getElementById('detectedOSName');
  const mainBtn = document.getElementById('mainDownloadBtn');
  const cmdElem = document.getElementById('quickInstallCmd');

  if (osLabel) osLabel.textContent = cfg.name;
  if (mainBtn) {
    mainBtn.href = `./downloads/${cfg.filename}`;
    mainBtn.innerHTML = `<span>Download ${cfg.filename}</span>`;
  }
  if (cmdElem) cmdElem.textContent = cfg.cmd;

  // Highlight selected card
  document.querySelectorAll('.os-card-wide').forEach(c => c.classList.remove('active-match'));
  const activeCard = document.getElementById(cfg.cardId);
  if (activeCard) activeCard.classList.add('active-match');
}

function initOSSelection() {
  const ua = window.navigator.userAgent.toLowerCase();
  const platform = window.navigator.platform ? window.navigator.platform.toLowerCase() : '';

  let initialKey = 'win';
  if (ua.includes('mac') || platform.includes('mac')) {
    initialKey = 'mac';
  } else if (ua.includes('linux') || platform.includes('linux')) {
    initialKey = 'linux';
  }

  setPlatform(initialKey);

  // Attach click listeners to all OS cards
  document.querySelectorAll('.os-card-wide').forEach(card => {
    card.addEventListener('click', (e) => {
      e.preventDefault();
      const osKey = card.getAttribute('data-os');
      if (osKey) setPlatform(osKey);
    });
  });
}

// 3. Terminal Emulator with Sandbox Quota Limit (Max 3 Queries)
const MAX_SANDBOX_QUERIES = 3;
let sandboxQueryCount = 0;

const terminalPresets = {
  vector: {
    query: `SELECT id, topic, notes, 
       cosine_similarity(embedding, [0.12, 0.45, -0.22, 0.81]) AS score
FROM memories
WHERE topic LIKE '%Rust%'
ORDER BY score DESC
LIMIT 2;`,
    output: `+--------+------------------------------+--------------------------+--------+
| id     | topic                        | notes                    | score  |
+--------+------------------------------+--------------------------+--------+
| mem_01 | High-Performance Rust Storage| Micro profile <512KB RAM | 0.9842 |
| mem_02 | Fused HNSW SIMD Traversal    | Single-pass filter search| 0.9120 |
+--------+------------------------------+--------------------------+--------+
2 rows in set. Execution time: 0.38 ms (SIMD fused)`
  },
  document: {
    query: `SELECT id, profile->'author'->>'name' AS author, profile->'tags' AS tags
FROM articles
WHERE 'ai' IN profile->'tags'
LIMIT 2;`,
    output: `+--------+--------------------+-----------------------------+
| id     | author             | tags                        |
+--------+--------------------+-----------------------------+
| art_01 | Maya Lin           | ["ai", "storage", "crdt"]   |
| art_04 | Alex Thorne        | ["ai", "vector", "rust"]    |
+--------+--------------------+-----------------------------+
2 rows in set. Execution time: 0.42 ms`
  },
  timetravel: {
    query: `SELECT balance FROM user_accounts 
AS OF TIMESTAMP '2026-11-01T00:00:00Z'
WHERE account_id = 'acc_4921';`,
    output: `+-------------------+--------------+
| account_id        | balance      |
+-------------------+--------------+
| acc_4921          | $14,250.00   |
+-------------------+--------------+
Snapshot: v18492 (Zero-copy CoW MVCC branch). Execution time: 0.29 ms`
  },
  live: {
    query: `LIVE SELECT order_id, customer, total_usd 
FROM orders 
WHERE status = 'pending';`,
    output: `[STREAM_OPEN] Continuous Subscription Active (Stream ID: 0x4f1a)
[WAL CDC +0.02s] INSERT -> {"order_id": "ord_881", "customer": "Dana", "total_usd": 128.50}
[WAL CDC +1.14s] UPDATE -> {"order_id": "ord_881", "status": "processing"}
Watching WAL commits with zero polling overhead.`
  }
};

function updateSandboxBadge() {
  const badge = document.getElementById('sandboxQuotaBadge');
  if (badge) {
    badge.textContent = `Sandbox Usage: ${sandboxQueryCount} / ${MAX_SANDBOX_QUERIES} queries`;
  }

  if (sandboxQueryCount >= MAX_SANDBOX_QUERIES) {
    const gateCard = document.getElementById('terminalGateCard');
    const promptArea = document.getElementById('terminalPromptArea');
    if (gateCard) gateCard.classList.add('active');
    if (promptArea) promptArea.style.display = 'none';

    runTerminalQuery(
      "-- DEMO NOTICE",
      `[SANDBOX LIMIT REACHED: 3/3 QUERIES USED]
To unlock unlimited local queries, full WAL persistence, HNSW vector indexing, 
and local ACID storage, download and run LoomDB directly on your system.`
    );
  }
}

function initTerminalWithQuota() {
  const termInput = document.getElementById('termInput');
  const presetChips = document.querySelectorAll('.preset-chip');
  const resetBtn = document.getElementById('resetDemoBtn');

  presetChips.forEach(chip => {
    chip.addEventListener('click', () => {
      if (sandboxQueryCount >= MAX_SANDBOX_QUERIES) return;

      presetChips.forEach(c => c.classList.remove('active'));
      chip.classList.add('active');
      const presetKey = chip.getAttribute('data-preset');
      const preset = terminalPresets[presetKey];
      if (preset) {
        sandboxQueryCount++;
        runTerminalQuery(preset.query, preset.output);
        updateSandboxBadge();
      }
    });
  });

  if (termInput) {
    termInput.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        const val = termInput.value.trim();
        if (val) {
          termInput.value = '';
          if (sandboxQueryCount >= MAX_SANDBOX_QUERIES) {
            updateSandboxBadge();
            return;
          }
          sandboxQueryCount++;
          handleManualInput(val);
          updateSandboxBadge();
        }
      }
    });
  }

  if (resetBtn) {
    resetBtn.addEventListener('click', () => {
      sandboxQueryCount = 0;
      updateSandboxBadge();
      const gateCard = document.getElementById('terminalGateCard');
      const promptArea = document.getElementById('terminalPromptArea');
      if (gateCard) gateCard.classList.remove('active');
      if (promptArea) promptArea.style.display = 'block';
      if (termInput) termInput.focus();
    });
  }
}

function runTerminalQuery(query, outputText) {
  const terminalBody = document.getElementById('terminalBody');
  const logDiv = document.createElement('div');
  logDiv.className = 'term-log';

  logDiv.innerHTML = `
<div style="color: #1cb58e; font-weight: 700; margin-bottom: 4px;">loom&gt; <span style="color: #fff; font-weight: normal;">${escapeHtml(query)}</span></div>
<div style="color: #8b949e; line-height: 1.5;">${escapeHtml(outputText)}</div>
`;
  terminalBody.appendChild(logDiv);
  terminalBody.scrollTop = terminalBody.scrollHeight;
}

function handleManualInput(input) {
  if (input === '.help') {
    runTerminalQuery(input, `LoomDB Interactive Shell Commands:
  .tables     List tables & collections in container
  .describe   Show column definitions & types
  .stats      Show virtual pages, WAL ring, and RAM consumption
  .mode       Switch format: table, json, csv
  .exit       Exit shell`);
    return;
  } else if (input === '.tables') {
    runTerminalQuery(input, `+--------------+-------------+-------------+
| Table Name   | Columns     | Rows        |
+--------------+-------------+-------------+
| memories     | 4           | 2           |
| users        | 3           | 1           |
| telemetry    | 5           | 120         |
+--------------+-------------+-------------+`);
    return;
  }

  const start = performance.now();
  setTimeout(() => {
    runTerminalQuery(input, `Query OK. (0.34 ms)
[LoomStore] 1 record processed.`);
  }, 50);
}

// 4. Graphical Installer Dialog Box Navigation
let currentDialogStep = 0;
const TOTAL_DIALOG_STEPS = 6;

function setDialogStep(stepIndex) {
  currentDialogStep = (stepIndex + TOTAL_DIALOG_STEPS) % TOTAL_DIALOG_STEPS;

  // Update tabs
  document.querySelectorAll('.dialog-step-tab').forEach((tab, i) => {
    if (i === currentDialogStep) {
      tab.classList.add('active');
    } else {
      tab.classList.remove('active');
    }
  });

  // Update content stages
  document.querySelectorAll('.dialog-stage').forEach((stage, i) => {
    if (i === currentDialogStep) {
      stage.classList.add('active');
    } else {
      stage.classList.remove('active');
    }
  });

  const nextBtn = document.getElementById('dialogNextBtn');
  if (nextBtn) {
    nextBtn.textContent = currentDialogStep === TOTAL_DIALOG_STEPS - 1 ? 'Finish' : 'Next >';
  }
}

function initInstallerDialog() {
  document.querySelectorAll('.dialog-step-tab').forEach(tab => {
    tab.addEventListener('click', () => {
      const step = parseInt(tab.getAttribute('data-step'), 10);
      if (!isNaN(step)) setDialogStep(step);
    });
  });

  const nextBtn = document.getElementById('dialogNextBtn');
  const backBtn = document.getElementById('dialogBackBtn');

  if (nextBtn) {
    nextBtn.addEventListener('click', () => setDialogStep(currentDialogStep + 1));
  }
  if (backBtn) {
    backBtn.addEventListener('click', () => setDialogStep(currentDialogStep - 1));
  }
}

// 5. Copy Command to Clipboard
function initCopyButtons() {
  document.querySelectorAll('.btn-copy-cmd').forEach(btn => {
    btn.addEventListener('click', () => {
      const targetId = btn.getAttribute('data-target');
      const elem = document.getElementById(targetId);
      if (elem) {
        navigator.clipboard.writeText(elem.textContent.trim()).then(() => {
          showToast('[COPIED] Command copied to clipboard');
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
  }, 2200);
}

function escapeHtml(string) {
  return String(string)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}
