// ==============================================================================
// LoomDB Client Runtime & Interactive Motion Engine
// High-Craft Systems Engineering, Titanium Metal & Warm Amber Aesthetic
// ==============================================================================

document.addEventListener('DOMContentLoaded', () => {
  initSmoothScrollMappings();
  initScrollAnimations();
  initSpotlightEffect();
  initNavScrollTracking();
  initParadigmWeavingHover();
  initContainerLayoutMap();
  initBenchmarkVisualizer();
  initOSSelection();
  initTerminalWithQuota();
  initSDKTabs();
  initCopyButtons();
});

// 1. Smooth Scroll & Button Mapping Engine (Guarantees Zero Clipping Under Sticky Nav)
function initSmoothScrollMappings() {
  document.querySelectorAll('a[href^="#"]').forEach(link => {
    link.addEventListener('click', (e) => {
      const targetId = link.getAttribute('href');
      if (!targetId || targetId === '#') {
        e.preventDefault();
        window.scrollTo({ top: 0, behavior: 'smooth' });
        return;
      }

      const targetElement = document.querySelector(targetId);
      if (!targetElement) return;

      e.preventDefault();

      const nav = document.querySelector('nav.site-nav');
      const navHeight = nav ? nav.getBoundingClientRect().height : 75;

      // Targeted offset: For #downloads, frame the download card cleanly below the navbar
      let targetPos;
      if (targetId === '#downloads') {
        const deck = targetElement.querySelector('.download-deck-wide') || targetElement;
        const deckRect = deck.getBoundingClientRect();
        const absoluteTop = window.scrollY + deckRect.top;
        targetPos = Math.max(0, absoluteTop - navHeight - 36);
      } else {
        const rect = targetElement.getBoundingClientRect();
        const absoluteTop = window.scrollY + rect.top;
        targetPos = Math.max(0, absoluteTop - navHeight - 24);
      }

      window.scrollTo({
        top: targetPos,
        behavior: 'smooth'
      });

      if (window.history && window.history.pushState) {
        window.history.pushState(null, null, targetId);
      }
    });
  });
}

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
    threshold: 0.08,
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

// 2. Interactive Spotlight Cursor Effect on Cards
function initSpotlightEffect() {
  const cards = document.querySelectorAll('.spotlight-card');
  cards.forEach(card => {
    card.addEventListener('mousemove', (e) => {
      const rect = card.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      card.style.setProperty('--mouse-x', `${x}px`);
      card.style.setProperty('--mouse-y', `${y}px`);
    });
  });
}

// 3. Navigation Anchor Tracking
function initNavScrollTracking() {
  const sections = document.querySelectorAll('section[id], header.hero-section');
  const navLinks = document.querySelectorAll('.nav-item-link');

  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        const id = entry.target.getAttribute('id');
        navLinks.forEach(link => {
          link.classList.remove('active-nav');
          if (link.getAttribute('href') === `#${id}`) {
            link.classList.add('active-nav');
          }
        });
      }
    });
  }, { threshold: 0.3 });

  sections.forEach(s => observer.observe(s));
}

// 4. Interactive Paradigm Weaving Connector
function initParadigmWeavingHover() {
  const flowRows = document.querySelectorAll('.flow-row');
  const streamLines = document.querySelectorAll('.flow-stream-line');

  flowRows.forEach(row => {
    const strandIndex = row.getAttribute('data-strand');

    row.addEventListener('mouseenter', () => {
      flowRows.forEach(r => r.classList.remove('active-row'));
      row.classList.add('active-row');

      streamLines.forEach(l => l.classList.remove('active-stream'));
      const activeLine = document.getElementById(`stream-${strandIndex}`);
      if (activeLine) activeLine.classList.add('active-stream');
    });

    row.addEventListener('mouseleave', () => {
      row.classList.remove('active-row');
      streamLines.forEach(l => l.classList.remove('active-stream'));
    });
  });
}

// 5. Interactive Container Layout Map
function initContainerLayoutMap() {
  const blocks = document.querySelectorAll('.map-block');
  const descElem = document.getElementById('layoutMapDesc');

  blocks.forEach(block => {
    block.addEventListener('click', () => {
      blocks.forEach(b => b.classList.remove('active-block'));
      block.classList.add('active-block');

      const descText = block.getAttribute('data-desc');
      const titleText = block.querySelector('strong') ? block.querySelector('strong').textContent : '';
      if (descElem) {
        descElem.innerHTML = `<span>Active Layer: <strong>${escapeHtml(titleText)}</strong> &mdash; ${escapeHtml(descText)}</span>
<span style="font-size: 0.72rem; color: var(--accent-amber);">PAGED TO DISK</span>`;
      }
    });
  });
}

// 6. Interactive Benchmark Comparison Visualizer
const BENCHMARK_DATA = {
  ram: [
    { name: 'LoomDB', score: '< 0.48 MB', width: '4%', isLoom: true },
    { name: 'SQLite', score: '~ 4.2 MB', width: '14%', isLoom: false },
    { name: 'DuckDB', score: '~ 32 MB', width: '38%', isLoom: false },
    { name: 'MongoDB', score: '~ 350 MB', width: '95%', isLoom: false },
    { name: 'Pinecone', score: '~ 210 MB', width: '68%', isLoom: false },
  ],
  latency: [
    { name: 'LoomDB', score: '0.38 ms', width: '5%', isLoom: true },
    { name: 'SQLite', score: '1.85 ms', width: '20%', isLoom: false },
    { name: 'DuckDB', score: '11.4 ms', width: '44%', isLoom: false },
    { name: 'MongoDB', score: '165 ms', width: '92%', isLoom: false },
    { name: 'Pinecone', score: '220 ms', width: '98%', isLoom: false },
  ],
  size: [
    { name: 'LoomDB', score: '< 0.85 MB', width: '4%', isLoom: true },
    { name: 'SQLite', score: '~ 0.80 MB', width: '4%', isLoom: false },
    { name: 'DuckDB', score: '~ 18 MB', width: '42%', isLoom: false },
    { name: 'MongoDB', score: '~ 120 MB', width: '95%', isLoom: false },
    { name: 'Pinecone', score: '~ 45 MB', width: '58%', isLoom: false },
  ]
};

function renderBenchmarks(metricKey) {
  const container = document.getElementById('benchBarsContainer');
  const items = BENCHMARK_DATA[metricKey] || BENCHMARK_DATA.ram;
  if (!container) return;

  container.innerHTML = items.map(item => `
    <div class="bench-bar-row">
      <span class="bench-name ${item.isLoom ? 'loom-name' : ''}">${escapeHtml(item.name)}</span>
      <div class="bench-track">
        <div class="bench-fill ${item.isLoom ? 'loom-fill' : ''}" style="width: 0%;" data-target-width="${item.width}"></div>
      </div>
      <span class="bench-score ${item.isLoom ? 'loom-score' : ''}">${escapeHtml(item.score)}</span>
    </div>
  `).join('');

  // Trigger smooth bar width expansion
  requestAnimationFrame(() => {
    container.querySelectorAll('.bench-fill').forEach(fill => {
      const target = fill.getAttribute('data-target-width');
      fill.style.width = target;
    });
  });
}

function initBenchmarkVisualizer() {
  const tabs = document.querySelectorAll('.bench-tab-btn');
  tabs.forEach(tab => {
    tab.addEventListener('click', () => {
      tabs.forEach(t => t.classList.remove('active'));
      tab.classList.add('active');
      const metric = tab.getAttribute('data-metric');
      renderBenchmarks(metric);
    });
  });

  // Initial trigger when benchmark section is visible
  const benchBox = document.querySelector('.benchmark-interactive-box');
  if (benchBox) {
    const observer = new IntersectionObserver((entries) => {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          renderBenchmarks('ram');
          observer.unobserve(entry.target);
        }
      });
    }, { threshold: 0.2 });
    observer.observe(benchBox);
  }
}

// 7. Platform Detection & Interactive OS Card Switching
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
    mainBtn.href = `./downloads/${cfg.filename}?v=0.1.2`;
    mainBtn.setAttribute('download', cfg.filename);
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
      // If clicking the direct download button, let it trigger download naturally
      if (e.target.closest('a')) {
        return;
      }
      e.preventDefault();
      const osKey = card.getAttribute('data-os');
      if (osKey) setPlatform(osKey);
    });
  });
}

// 8. Terminal Emulator with Sandbox Quota Limit (Max 3 Queries)
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
  fork: {
    query: `FORK BRANCH 'experiment_a';
CHECKOUT 'experiment_a';`,
    output: `Snapshot OK: Created forkable branch 'experiment_a' (Copy-on-Write generation).
Switched to branch 'experiment_a'. Execution time: 0.32 ms
Page directory COW mapping: 100% shared base, zero duplicated bytes.`
  },
  budget: {
    query: `SELECT * FROM _sys_budget;`,
    output: `+----------------------+-------+---------+-----------------+--------+
| metric               | used  | cap     | utilization_pct | status |
+----------------------+-------+---------+-----------------+--------+
| max_ram_bytes        | 43008 | 524288  | 8.2%            | NORMAL |
| max_wal_bytes        | 0     | 4194304 | 0.0%            | NORMAL |
| max_page_cache_bytes | 65536 | 262144  | 25.0%           | NORMAL |
| max_io_per_query     | 0     | 1000    | 0.0%            | NORMAL |
+----------------------+-------+---------+-----------------+--------+
4 rows in set. Hard resource budget strictly enforced.`
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
<div style="color: var(--accent-amber); font-weight: 700; margin-bottom: 4px;">loom&gt; <span style="color: #fff; font-weight: normal;">${escapeHtml(query)}</span></div>
<div style="color: #94a3b8; line-height: 1.5;">${escapeHtml(outputText)}</div>
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

  setTimeout(() => {
    runTerminalQuery(input, `Query OK. (0.34 ms)
[LoomStore] 1 record processed.`);
  }, 50);
}

// 9. Developer SDK Code Tabs Switcher
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
    title: "app.py — Python Driver (Embedded & HTTP Daemon Modes)",
    code: `from loomdb import LoomDB

# Connect to LoomDB (auto-detects local CLI or running HTTP daemon)
db = LoomDB()

# 1. Execute DDL and DML
db.execute("CREATE TABLE products (id INT, name TEXT, price REAL)")
db.execute("INSERT INTO products VALUES (1, 'Neural Mesh Sensor', 49.99)")

# 2. Query with structured dictionary output
result = db.query("SELECT * FROM products")
print(f"Retrieved {result.count} rows in {result.elapsed_us}µs:")

for row in result:
    print(row["id"], row["name"], row["price"])`
  },
  ts: {
    title: "index.ts — TypeScript & Node.js Driver",
    code: `import { LoomDB } from 'loomdb';

// Connect in auto mode (HTTP micro-daemon or local CLI subshell)
const db = new LoomDB();

// Execute queries seamlessly
const result = await db.query('SELECT * FROM users');
console.log(\`Retrieved \${result.count} users in \${result.elapsedUs}µs:\`);
console.log(result.rows);

// Query all table names in active container
const tables = await db.getTables();
console.log('Tables:', tables);`
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

// 10. Copy Command to Clipboard
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
