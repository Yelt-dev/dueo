-- Migration 0008: case-insensitive usernames + a missing index.

-- Enforce case-insensitive uniqueness so "Bob" and "bob" can't be two accounts,
-- and so login can look up case-insensitively (matching the lowercased rate-limit
-- key). The original column-level UNIQUE stays as a case-sensitive backstop.
CREATE UNIQUE INDEX idx_users_username_nocase ON users(username COLLATE NOCASE);

-- Back the ON DELETE CASCADE from subscriptions and the scheduler's per-sub rule
-- lookup; the existing UNIQUE leads with user_id, so there was no usable index
-- on subscription_id alone.
CREATE INDEX idx_reminder_rules_sub ON reminder_rules(subscription_id);
