# LoomDB Node.js & TypeScript Client (`loomdb`)

Official zero-dependency driver for **LoomDB** — the ultra-lightweight multi-modal AI database.

## Installation

```bash
npm install loomdb
```

## Quickstart (JavaScript & TypeScript)

```javascript
const { LoomDB } = require('loomdb');

async function main() {
  const db = new LoomDB();

  // Create table
  await db.execute('CREATE TABLE telemetry (id INT, sensor TEXT, reading REAL)');

  // Insert records
  await db.execute("INSERT INTO telemetry VALUES (1, 'gyro-01', 98.4)");
  await db.execute("INSERT INTO telemetry VALUES (2, 'temp-04', 32.1)");

  // Query structured records
  const result = await db.query('SELECT * FROM telemetry');
  console.log(`Retrieved ${result.count} rows in ${result.elapsedUs}µs:`);
  console.log(result.rows);
}

main().catch(console.error);
```

## Synchronous Execution
For scripts or rapid CLI tasks:

```javascript
const { LoomDB } = require('loomdb');
const db = new LoomDB();

const res = db.querySync('SELECT * FROM users');
console.log(res.rows);
```
