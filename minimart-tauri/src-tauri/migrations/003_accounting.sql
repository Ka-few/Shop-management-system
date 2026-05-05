-- Accounting module schema and default chart of accounts.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS accounts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  code TEXT UNIQUE NOT NULL,
  name TEXT NOT NULL,
  type TEXT NOT NULL CHECK (type IN ('Asset', 'Liability', 'Equity', 'Revenue', 'Expense')),
  parent_id INTEGER,
  is_active INTEGER NOT NULL DEFAULT 1,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (parent_id) REFERENCES accounts(id)
);

CREATE TABLE IF NOT EXISTS journal_entries (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  date TEXT NOT NULL,
  reference TEXT,
  description TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'posted' CHECK (status IN ('posted', 'reversed')),
  source_type TEXT DEFAULT 'manual',
  source_id INTEGER,
  reversed_entry_id INTEGER,
  is_system_generated INTEGER NOT NULL DEFAULT 0,
  created_by INTEGER,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (reversed_entry_id) REFERENCES journal_entries(id),
  FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS journal_lines (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  journal_entry_id INTEGER NOT NULL,
  account_id INTEGER NOT NULL,
  debit REAL NOT NULL DEFAULT 0 CHECK (debit >= 0),
  credit REAL NOT NULL DEFAULT 0 CHECK (credit >= 0),
  memo TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (journal_entry_id) REFERENCES journal_entries(id) ON DELETE CASCADE,
  FOREIGN KEY (account_id) REFERENCES accounts(id)
);

CREATE INDEX IF NOT EXISTS idx_accounts_code ON accounts(code);
CREATE INDEX IF NOT EXISTS idx_accounts_type ON accounts(type);
CREATE INDEX IF NOT EXISTS idx_accounts_parent ON accounts(parent_id);
CREATE INDEX IF NOT EXISTS idx_journal_entries_date ON journal_entries(date);
CREATE INDEX IF NOT EXISTS idx_journal_entries_reference ON journal_entries(reference);
CREATE INDEX IF NOT EXISTS idx_journal_entries_source ON journal_entries(source_type, source_id);
CREATE INDEX IF NOT EXISTS idx_journal_lines_entry ON journal_lines(journal_entry_id);
CREATE INDEX IF NOT EXISTS idx_journal_lines_account ON journal_lines(account_id);

CREATE TRIGGER IF NOT EXISTS update_accounts_timestamp
  AFTER UPDATE ON accounts
  FOR EACH ROW
  BEGIN
    UPDATE accounts SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
  END;

CREATE TRIGGER IF NOT EXISTS update_journal_entries_timestamp
  AFTER UPDATE ON journal_entries
  FOR EACH ROW
  BEGIN
    UPDATE journal_entries SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
  END;

INSERT OR IGNORE INTO accounts (code, name, type, parent_id) VALUES
('1000', 'Assets', 'Asset', NULL),
('1100', 'Cash', 'Asset', (SELECT id FROM accounts WHERE code = '1000')),
('1110', 'M-Pesa', 'Asset', (SELECT id FROM accounts WHERE code = '1000')),
('1120', 'Card Clearing', 'Asset', (SELECT id FROM accounts WHERE code = '1000')),
('1200', 'Inventory', 'Asset', (SELECT id FROM accounts WHERE code = '1000')),
('2000', 'Liabilities', 'Liability', NULL),
('2100', 'VAT Payable', 'Liability', (SELECT id FROM accounts WHERE code = '2000')),
('3000', 'Equity', 'Equity', NULL),
('3100', 'Owner Equity', 'Equity', (SELECT id FROM accounts WHERE code = '3000')),
('4000', 'Revenue', 'Revenue', NULL),
('4100', 'Sales Revenue', 'Revenue', (SELECT id FROM accounts WHERE code = '4000')),
('5000', 'Expenses', 'Expense', NULL),
('5100', 'Cost of Goods Sold', 'Expense', (SELECT id FROM accounts WHERE code = '5000')),
('5200', 'Operating Expenses', 'Expense', (SELECT id FROM accounts WHERE code = '5000'));
