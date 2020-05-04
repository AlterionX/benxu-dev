ALTER TABLE capabilities RENAME TO permissions;
ALTER TABLE permissions RENAME COLUMN capability TO permission;

