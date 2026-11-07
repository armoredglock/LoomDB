# LoomDB Python Client (`loomdb`)

Official zero-dependency Python client for **LoomDB** — the world's lightest multi-modal AI-native database.

## Installation

```bash
pip install loomdb
```
*(Or install locally: `pip install -e clients/python`)*

## Quickstart

```python
from loomdb import LoomDB

# Connect in auto mode (HTTP daemon if running, otherwise embedded CLI)
db = LoomDB()

# 1. Create a table
db.execute("CREATE TABLE products (id INT, name TEXT, price REAL)")

# 2. Insert records
db.execute("INSERT INTO products VALUES (1, 'Neural Mesh Sensor', 49.99)")
db.execute("INSERT INTO products VALUES (2, 'Quantum Optical Loom', 249.50)")

# 3. Query records with structured dictionary output
result = db.query("SELECT * FROM products")
print(f"Count: {result.count}, Latency: {result.elapsed_us}us")

for row in result:
    print(row["id"], row["name"], row["price"])
```

## Modes of Operation

LoomDB supports two operational modes depending on your architecture:

1. **Auto / CLI Subshell Mode (Process-bound)**:
   Zero configuration. Works without Docker, background services, or cloud infrastructure. LoomDB runs direct zero-copy queries via the local binary and parses high-speed JSON stdout.

2. **HTTP REST Micro-Daemon Mode**:
   Connects to a running LoomDB daemon on port `8484`. Perfect for web applications, async workers, and microservices:
   ```python
   db = LoomDB(mode="http", http_url="http://localhost:8484")
   ```
