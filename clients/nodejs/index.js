/**
 * LoomDB Official Node.js & TypeScript Driver
 * Zero-dependency client supporting CLI subprocess and HTTP daemon connections.
 */

const http = require('http');
const { spawnSync } = require('child_process');
const path = require('path');
const fs = require('fs');

class LoomError extends Error {
  constructor(message, status = 'error', raw = null) {
    super(message);
    this.name = 'LoomError';
    this.status = status;
    this.raw = raw;
  }
}

class LoomResult {
  constructor(data) {
    this.status = data.status || 'ok';
    this.columns = data.columns || [];
    this.rows = data.rows || [];
    this.count = data.count !== undefined ? data.count : this.rows.length;
    this.affectedRows = data.affected_rows || 0;
    this.elapsedUs = data.elapsed_us || 0;
    this.message = data.message || '';
    this.raw = data;
  }

  first() {
    return this.rows.length > 0 ? this.rows[0] : null;
  }
}

class LoomDB {
  /**
   * @param {Object} [options]
   * @param {'auto'|'http'|'cli'} [options.mode='auto']
   * @param {string} [options.url='http://localhost:8484']
   * @param {string} [options.binaryPath]
   * @param {number} [options.timeout=5000]
   */
  constructor(options = {}) {
    this.mode = options.mode || 'auto';
    this.url = options.url || 'http://localhost:8484';
    this.binaryPath = options.binaryPath || this._resolveBinary();
    this.timeout = options.timeout || 5000;
  }

  _resolveBinary() {
    // 1. Look in LocalAppData (Windows)
    const localApp = process.env.LOCALAPPDATA;
    if (localApp) {
      const p = path.join(localApp, 'LoomDB', 'loom.exe');
      if (fs.existsSync(p)) return p;
    }

    // 2. Relative workspace binary
    const devBinary = path.resolve(__dirname, '..', '..', 'target', 'release', 'loom.exe');
    if (fs.existsSync(devBinary)) return devBinary;

    // 3. System default
    return process.platform === 'win32' ? 'loom.exe' : 'loom';
  }

  /**
   * Execute an SQL query against LoomDB.
   * @param {string} sql
   * @returns {Promise<LoomResult>}
   */
  async query(sql) {
    if (this.mode === 'http') {
      return this._queryHttp(sql);
    } else if (this.mode === 'cli') {
      return this._queryCli(sql);
    }

    // Auto mode: attempt HTTP, fallback to CLI
    try {
      return await this._queryHttp(sql);
    } catch (e) {
      return this._queryCli(sql);
    }
  }

  /**
   * Execute a DDL or DML statement.
   * @param {string} sql
   * @returns {Promise<LoomResult>}
   */
  async execute(sql) {
    return this.query(sql);
  }

  /**
   * Synchronous query execution via CLI subprocess.
   * @param {string} sql
   * @returns {LoomResult}
   */
  querySync(sql) {
    return this._queryCli(sql);
  }

  _queryHttp(sql) {
    return new Promise((resolve, reject) => {
      const parsedUrl = new URL('/query', this.url);
      const postData = JSON.stringify({ sql });

      const req = http.request(
        parsedUrl,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Content-Length': Buffer.byteLength(postData),
          },
          timeout: this.timeout,
        },
        (res) => {
          let body = '';
          res.setEncoding('utf8');
          res.on('data', (chunk) => { body += chunk; });
          res.on('end', () => {
            try {
              const data = JSON.parse(body);
              if (data.status === 'error') {
                reject(new LoomError(data.message || 'Database error', 'error', data));
              } else {
                resolve(new LoomResult(data));
              }
            } catch (err) {
              reject(new LoomError(`Invalid JSON from LoomDB daemon: ${body}`));
            }
          });
        }
      );

      req.on('error', (err) => reject(err));
      req.on('timeout', () => {
        req.destroy();
        reject(new LoomError('LoomDB HTTP request timed out'));
      });

      req.write(postData);
      req.end();
    });
  }

  _queryCli(sql) {
    const res = spawnSync(this.binaryPath, ['query', sql, '--format', 'json'], {
      encoding: 'utf8',
      timeout: this.timeout,
    });

    if (res.error) {
      throw new LoomError(`Failed to run Loom CLI at '${this.binaryPath}': ${res.error.message}`);
    }

    const output = (res.stdout || '').trim();
    if (!output) {
      const err = (res.stderr || '').trim();
      throw new LoomError(`Loom CLI returned empty output: ${err}`);
    }

    try {
      const data = JSON.parse(output);
      if (data.status === 'error') {
        throw new LoomError(data.message || 'Database error', 'error', data);
      }
      return new LoomResult(data);
    } catch (err) {
      if (err instanceof LoomError) throw err;
      throw new LoomError(`Failed to parse Loom output as JSON: ${output}`);
    }
  }

  /**
   * Fetch all tables currently stored in the database.
   * @returns {Promise<string[]>}
   */
  async getTables() {
    const res = await this.query('SELECT * FROM _catalog_tables');
    return res.rows.map((r) => r.name).filter(Boolean);
  }
}

module.exports = {
  LoomDB,
  LoomResult,
  LoomError,
};
