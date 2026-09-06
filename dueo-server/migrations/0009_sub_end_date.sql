-- Migration 0009: optional termination date for a subscription.
-- NULL = open-ended: the service keeps renewing cycle after cycle until the
-- user cancels it (the common case: Netflix, Spotify).
-- A date = the contract ends there; the scheduler stops rolling the cycle over
-- past it and the subscription becomes 'expired' (a VPS or domain paid for a
-- fixed term).
ALTER TABLE subscriptions ADD COLUMN end_date TEXT;
