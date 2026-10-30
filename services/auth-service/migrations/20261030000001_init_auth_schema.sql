-- Migration: 20261030000001_init_auth_schema.sql
-- Description: Initialize core schema for auth-service (users, identities, refresh_tokens)

-- 1. Create Enums
CREATE TYPE user_status AS ENUM ('ACTIVE', 'BANNED', 'DELETED');
CREATE TYPE identity_provider AS ENUM ('EMAIL', 'GOOGLE');

-- 2. Create Users Table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    email VARCHAR(255) UNIQUE NOT NULL,
    status user_status NOT NULL DEFAULT 'ACTIVE',
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 3. Create Identities Table
CREATE TABLE identities (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider identity_provider NOT NULL,
    identifier VARCHAR(255) NOT NULL,
    credential VARCHAR(255),
    verified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_identities_provider_identifier UNIQUE (provider, identifier)
);

CREATE INDEX idx_identities_user_id ON identities(user_id);

-- 4. Create Refresh Tokens Table
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
