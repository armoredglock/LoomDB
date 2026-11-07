// ==============================================================================
// LoomDB Client Runtime & Interactive Motion Engine
// ==============================================================================

document.addEventListener('DOMContentLoaded', () => {
  initScrollAnimations();
  initOSSelection();
  initTerminalWithQuota();
  initSDKTabs();
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

// 4. Developer SDK Code Tabs Switcher
const SDK_EXAMPLES = {
  rust: {
    title: "main.rs — Embedded In-Process Zero-Copy Access",
    code: `// Open or create a single-file portable container
let db = loom_core::LoomDB::open("./app.loom")?;

// Insert document with high-dimensional vector embedding
db.collection("memories").insert(serde_json::json!({
    "id": "mem_01",
    "topic": "High-Performance Rust Storage",
    "notes": "LoomDB achieves sub-512KB idle RAM overhead.",
    "embedding": [0.12, 0.45, -0.22, 0.81]
}))?;

// Hybrid single-pass vector distance query with relational filter
let hits = db.query(
    "SELECT id, topic, notes FROM memories 
     WHERE topic LIKE '%Rust%' 
     ORDER BY cosine_similarity(embedding, [0.12, 0.45, -0.22, 0.81]) DESC 
     LIMIT 1;"
)?;
println!("Found: {:?}", hits);`
  },
  python: {
    title: "rag_agent.py — Python Local AI Memory Engine",
    code: `import loomdb

# Connect to single-file local container (Zero Docker, Zero Cloud bills)
db = loomdb.open("./agent_memory.loom")

# Insert document chunks with embeddings
db.insert("knowledge", [
    {
        "id": "doc_991",
        "content": "LoomDB compiles as a native C-ABI library for Python.",
        "embedding": [0.041, -0.220, 0.880, 0.105],
        "category": "architecture"
    }
])

# Fused hybrid vector search: vector cosine similarity + metadata filter
results = db.vector_search(
    collection="knowledge",
    vector=[0.045, -0.231, 0.892, 0.112],
    filter="category = 'architecture'",
    top_k=3
)
for match in results:
    print(f"[{match['score']:.4f}] {match['content']}")`
  },
  ts: {
    title: "server.ts — TypeScript Client & Real-Time Live Queries",
    code: `import { LoomDB } from '@loomdb/client';

// Connect to local LoomDB background daemon
const db = await LoomDB.connect('loom://127.0.0.1:8484');

// Reactive Live Query with continuous push stream (No polling)
const liveOrders = db.live("SELECT * FROM telemetry WHERE temp > 85.0");

liveOrders.on('data', (point) => {
  console.warn("Critical temperature reading:", point.device_id, point.temp);
});

// Execute atomic transaction
await db.transaction(async (tx) => {
  await tx.execute("INSERT INTO telemetry (device_id, temp) VALUES ($1, $2)", ["sensor_04", 89.2]);
});`
  },
  cli: {
    title: "Terminal Command Prompt / PowerShell Usage",
    code: `C:\\> loom
[LOOM_DB] Interactive Shell v0.1.0
Mounted Container: ./data.loom

loom> CREATE TABLE users (id UUID PRIMARY KEY, name VARCHAR, email VARCHAR UNIQUE);
Query OK, 0 rows affected. Table 'users' initialized. (520.70µs)

loom> INSERT INTO users (id, name, email) VALUES ('u1', 'Alice', 'alice@loomdb.io');
Query OK, 1 row inserted. (278.40µs)

loom> SELECT * FROM users;
+----+-------+-----------------+
| id | name  | email           |
+----+-------+-----------------+
| u1 | Alice | alice@loomdb.io |
+----+-------+-----------------+
1 rows in set. (2.80µs)`
  }
};

function initSDKTabs() {
  const tabs = document.querySelectorAll('.sdk-tab');
  const titleElem = document.getElementById('sdkCodeTitle');
  const codeBlock = document.getElementById('sdkCodeBlock');

  tabs.forEach(tab => {
    tab.addEventListener('click', () => {
      tabs.forEach(t => t.classList.remove('active'));
      tab.classList.add('active');

      const sdkKey = tab.getAttribute('data-sdk');
      const example = SDK_EXAMPLES[sdkKey];
      if (example && titleElem && codeBlock) {
        titleElem.textContent = example.title;
        codeBlock.textContent = example.code;
      }
    });
  });
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
