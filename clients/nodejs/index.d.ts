export interface LoomRow {
  [key: string]: any;
}

export interface LoomResultRaw {
  status: 'ok' | 'error';
  columns?: string[];
  rows?: LoomRow[];
  count?: number;
  affected_rows?: number;
  elapsed_us?: number;
  message?: string;
}

export declare class LoomResult {
  status: string;
  columns: string[];
  rows: LoomRow[];
  count: number;
  affectedRows: number;
  elapsedUs: number;
  message: string;
  raw: LoomResultRaw;

  first(): LoomRow | null;
}

export interface LoomOptions {
  mode?: 'auto' | 'http' | 'cli';
  url?: string;
  binaryPath?: string;
  timeout?: number;
}

export declare class LoomError extends Error {
  status: string;
  raw?: any;
}

export declare class LoomDB {
  constructor(options?: LoomOptions);

  query(sql: string): Promise<LoomResult>;
  execute(sql: string): Promise<LoomResult>;
  querySync(sql: string): LoomResult;
  getTables(): Promise<string[]>;
}
