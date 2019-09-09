CREATE TABLE users (
    -- management, TODO consider propic
    id uuid NOT NULL UNIQUE PRIMARY KEY,
    user_name TEXT NOT NULL UNIQUE,
    created_at timestamp with time zone NOT NULL DEFAULT (now() at time zone 'utc'),
    created_by uuid REFERENCES users(id),
    updated_at timestamp with time zone NOT NULL DEFAULT (now() at time zone 'utc'),
    updated_by uuid REFERENCES users(id),
    -- contact info
    first_name TEXT,
    last_name TEXT,
    email VARCHAR(320) -- 64 char for first part, 1 for `@`, 255 for domain, TODO uniqueness?
);

-- TODO consider images & history
CREATE TABLE posts (
    -- management
    id uuid NOT NULL UNIQUE PRIMARY KEY,
    created_at timestamp with time zone NOT NULL DEFAULT (now() at time zone 'utc'),
    created_by uuid REFERENCES users(id) NOT NULL,
    updated_at timestamp with time zone NOT NULL DEFAULT (now() at time zone 'utc'),
    updated_by uuid REFERENCES users(id) NOT NULL,
    published_at timestamp with time zone,
    published_by uuid REFERENCES users(id),
    archived_at timestamp with time zone,
    archived_by uuid REFERENCES users(id),
    deleted_at timestamp with time zone,
    deleted_by uuid REFERENCES users(id),
    -- basic info
    title TEXT NOT NULL UNIQUE,
    body TEXT NOT NULL
);

CREATE TABLE tags (
    -- management
    id uuid NOT NULL UNIQUE PRIMARY KEY,
    created_at timestamp with time zone NOT NULL DEFAULT (now() at time zone 'utc'),
    created_by uuid REFERENCES users(id) NOT NULL,
    -- basic info
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL
);

CREATE TABLE post_tag_junctions (
    -- junction
    post_id uuid NOT NULL REFERENCES posts(id),
    tag_id uuid NOT NULL REFERENCES tags(id),
    -- managerial
    created_at timestamp with time zone NOT NULL DEFAULT (now() at time zone 'utc'),
    created_by uuid REFERENCES users(id) NOT NULL,
    -- enforce no dupes
    CONSTRAINT post_tag_junction_pk PRIMARY KEY (post_id, tag_id)
);

CREATE TABLE passwords (
    -- management
    id uuid NOT NULL UNIQUE PRIMARY KEY,
    created_at timestamp with time zone NOT NULL DEFAULT (now() at time zone 'utc'),
    created_by uuid REFERENCES users(id) NOT NULL,
    updated_at timestamp with time zone NOT NULL DEFAULT (now() at time zone 'utc'),
    updated_by uuid REFERENCES users(id) NOT NULL,
    user_id uuid REFERENCES users(id) NOT NULL,
    -- basic info
    hash TEXT NOT NULL
);
CREATE TABLE google_sso (
    -- management
    id uuid NOT NULL UNIQUE PRIMARY KEY,
    created_at timestamp with time zone NOT NULL DEFAULT (now() at time zone 'utc'),
    created_by uuid REFERENCES users(id) NOT NULL,
    updated_at timestamp with time zone NOT NULL DEFAULT (now() at time zone 'utc'),
    updated_by uuid REFERENCES users(id) NOT NULL,
    user_id uuid REFERENCES users(id) NOT NULL
    -- TODO figure out what's needed
);

