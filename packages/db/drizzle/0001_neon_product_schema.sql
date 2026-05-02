CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TYPE release_channel AS ENUM ('dev', 'beta', 'stable');
CREATE TYPE release_platform AS ENUM ('windows-x64');
CREATE TYPE benchmark_phase AS ENUM ('before', 'after', 'single');
CREATE TYPE audit_outcome AS ENUM ('allowed', 'denied', 'failed');
CREATE TYPE license_status AS ENUM ('pending', 'active', 'expired', 'revoked');

CREATE TABLE auth_users (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  external_subject varchar(256),
  email_hash varchar(128),
  display_name varchar(160),
  metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
  created_at timestamptz DEFAULT now() NOT NULL,
  updated_at timestamptz DEFAULT now() NOT NULL
);

CREATE TABLE license_entitlements (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  user_id uuid REFERENCES auth_users(id) ON DELETE SET NULL,
  license_key_hash varchar(128) NOT NULL,
  status license_status DEFAULT 'pending' NOT NULL,
  plan varchar(64) DEFAULT 'future' NOT NULL,
  issued_at timestamptz,
  expires_at timestamptz,
  metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
  created_at timestamptz DEFAULT now() NOT NULL,
  updated_at timestamptz DEFAULT now() NOT NULL
);

CREATE TABLE devices (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  user_id uuid REFERENCES auth_users(id) ON DELETE SET NULL,
  stable_device_hash varchar(128) NOT NULL,
  install_id varchar(128) NOT NULL,
  display_name varchar(160),
  os_name varchar(64) DEFAULT 'windows' NOT NULL,
  os_build varchar(64),
  cpu_summary varchar(240),
  gpu_summary varchar(240),
  app_version varchar(64),
  last_seen_at timestamptz,
  metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
  created_at timestamptz DEFAULT now() NOT NULL,
  updated_at timestamptz DEFAULT now() NOT NULL
);

CREATE TABLE app_releases (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  version varchar(64) NOT NULL,
  channel release_channel NOT NULL,
  platform release_platform DEFAULT 'windows-x64' NOT NULL,
  artifact_url text,
  artifact_sha256 varchar(64),
  signature text,
  release_notes_url text,
  minimum_app_version varchar(64),
  rollout_percent integer DEFAULT 100 NOT NULL,
  is_critical boolean DEFAULT false NOT NULL,
  published_at timestamptz,
  metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
  created_at timestamptz DEFAULT now() NOT NULL,
  CONSTRAINT ck_app_releases_rollout_percent CHECK (rollout_percent BETWEEN 0 AND 100),
  CONSTRAINT ck_app_releases_artifact_sha256 CHECK (
    artifact_sha256 IS NULL OR artifact_sha256 ~ '^[a-f0-9]{64}$'
  )
);

CREATE TABLE tweak_catalog_versions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  version varchar(96) NOT NULL,
  channel release_channel NOT NULL,
  schema_version varchar(32) NOT NULL,
  minimum_app_version varchar(64),
  payload_sha256 varchar(64) NOT NULL,
  signature text NOT NULL,
  payload_url text,
  published_at timestamptz,
  revoked_at timestamptz,
  created_at timestamptz DEFAULT now() NOT NULL,
  CONSTRAINT ck_tweak_catalog_payload_sha256 CHECK (payload_sha256 ~ '^[a-f0-9]{64}$')
);

CREATE TABLE tweak_catalog_entries (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  catalog_version_id uuid NOT NULL REFERENCES tweak_catalog_versions(id) ON DELETE CASCADE,
  tweak_id varchar(160) NOT NULL,
  category varchar(80) NOT NULL,
  mode varchar(48) NOT NULL,
  risk varchar(48) NOT NULL,
  title varchar(160) NOT NULL,
  summary text NOT NULL,
  payload jsonb DEFAULT '{}'::jsonb NOT NULL,
  sort_order integer DEFAULT 0 NOT NULL,
  created_at timestamptz DEFAULT now() NOT NULL
);

CREATE TABLE benchmark_sessions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  external_session_id varchar(128) NOT NULL,
  device_id uuid REFERENCES devices(id) ON DELETE SET NULL,
  user_id uuid REFERENCES auth_users(id) ON DELETE SET NULL,
  game varchar(64) NOT NULL,
  session_label varchar(96),
  active_power_plan varchar(96) NOT NULL,
  active_optimizer_profile varchar(96) NOT NULL,
  windows_build varchar(64) NOT NULL,
  driver_version varchar(64) NOT NULL,
  consented_at timestamptz,
  created_at timestamptz NOT NULL,
  uploaded_at timestamptz DEFAULT now() NOT NULL,
  metadata jsonb DEFAULT '{}'::jsonb NOT NULL
);

CREATE TABLE benchmark_captures (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  session_id uuid NOT NULL REFERENCES benchmark_sessions(id) ON DELETE CASCADE,
  external_capture_id varchar(128) NOT NULL,
  phase benchmark_phase NOT NULL,
  captured_at timestamptz NOT NULL,
  measurement_source varchar(96) NOT NULL,
  average_fps numeric(10, 3) NOT NULL,
  one_percent_low_fps numeric(10, 3) NOT NULL,
  zero_point_one_percent_low_fps numeric(10, 3) NOT NULL,
  frametime_p50_ms numeric(10, 3) NOT NULL,
  frametime_p95_ms numeric(10, 3) NOT NULL,
  frametime_p99_ms numeric(10, 3) NOT NULL,
  dropped_frames integer DEFAULT 0 NOT NULL,
  delayed_frames integer DEFAULT 0 NOT NULL,
  generated_frames_detected boolean DEFAULT false NOT NULL,
  latency_proxy boolean DEFAULT false NOT NULL,
  metrics jsonb DEFAULT '{}'::jsonb NOT NULL,
  CONSTRAINT ck_benchmark_captures_nonnegative CHECK (
    average_fps >= 0
    AND one_percent_low_fps >= 0
    AND zero_point_one_percent_low_fps >= 0
    AND frametime_p50_ms >= 0
    AND frametime_p95_ms >= 0
    AND frametime_p99_ms >= 0
    AND dropped_frames >= 0
    AND delayed_frames >= 0
  )
);

CREATE TABLE audit_events (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  device_id uuid REFERENCES devices(id) ON DELETE SET NULL,
  user_id uuid REFERENCES auth_users(id) ON DELETE SET NULL,
  request_id varchar(128),
  actor_type varchar(48) DEFAULT 'system' NOT NULL,
  action varchar(160) NOT NULL,
  outcome audit_outcome NOT NULL,
  procedure varchar(160),
  entity_type varchar(96),
  entity_id varchar(160),
  remote_address_hash varchar(128),
  user_agent_hash varchar(128),
  metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
  created_at timestamptz DEFAULT now() NOT NULL
);

CREATE TABLE feature_flags (
  key varchar(128) PRIMARY KEY NOT NULL,
  description text DEFAULT '' NOT NULL,
  enabled boolean DEFAULT false NOT NULL,
  default_variant varchar(96),
  rollout_percent integer DEFAULT 0 NOT NULL,
  constraints jsonb DEFAULT '{}'::jsonb NOT NULL,
  created_at timestamptz DEFAULT now() NOT NULL,
  updated_at timestamptz DEFAULT now() NOT NULL,
  CONSTRAINT ck_feature_flags_rollout_percent CHECK (rollout_percent BETWEEN 0 AND 100)
);

CREATE TABLE feature_flag_overrides (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  flag_key varchar(128) NOT NULL REFERENCES feature_flags(key) ON DELETE CASCADE,
  channel release_channel,
  device_id uuid REFERENCES devices(id) ON DELETE CASCADE,
  user_id uuid REFERENCES auth_users(id) ON DELETE CASCADE,
  enabled boolean NOT NULL,
  variant varchar(96),
  reason text,
  starts_at timestamptz,
  expires_at timestamptz,
  created_at timestamptz DEFAULT now() NOT NULL,
  CONSTRAINT ck_feature_flag_overrides_target CHECK (
    channel IS NOT NULL OR device_id IS NOT NULL OR user_id IS NOT NULL
  )
);

CREATE UNIQUE INDEX ux_auth_users_external_subject ON auth_users(external_subject);
CREATE UNIQUE INDEX ux_auth_users_email_hash ON auth_users(email_hash);
CREATE UNIQUE INDEX ux_license_entitlements_key_hash ON license_entitlements(license_key_hash);
CREATE INDEX ix_license_entitlements_user_id ON license_entitlements(user_id);
CREATE UNIQUE INDEX ux_devices_stable_install ON devices(stable_device_hash, install_id);
CREATE INDEX ix_devices_user_id ON devices(user_id);
CREATE INDEX ix_devices_last_seen_at ON devices(last_seen_at);
CREATE UNIQUE INDEX ux_app_releases_version_channel_platform ON app_releases(version, channel, platform);
CREATE INDEX ix_app_releases_channel_published ON app_releases(channel, published_at DESC);
CREATE UNIQUE INDEX ux_tweak_catalog_versions_version_channel ON tweak_catalog_versions(version, channel);
CREATE INDEX ix_tweak_catalog_versions_channel_published ON tweak_catalog_versions(channel, published_at DESC);
CREATE UNIQUE INDEX ux_tweak_catalog_entries_catalog_tweak ON tweak_catalog_entries(catalog_version_id, tweak_id);
CREATE INDEX ix_tweak_catalog_entries_catalog_sort ON tweak_catalog_entries(catalog_version_id, sort_order);
CREATE UNIQUE INDEX ux_benchmark_sessions_external_id ON benchmark_sessions(external_session_id);
CREATE INDEX ix_benchmark_sessions_device_id ON benchmark_sessions(device_id);
CREATE INDEX ix_benchmark_sessions_user_id ON benchmark_sessions(user_id);
CREATE INDEX ix_benchmark_sessions_created_at ON benchmark_sessions(created_at DESC);
CREATE UNIQUE INDEX ux_benchmark_captures_session_capture ON benchmark_captures(session_id, external_capture_id);
CREATE INDEX ix_benchmark_captures_session_id ON benchmark_captures(session_id);
CREATE INDEX ix_benchmark_captures_captured_at ON benchmark_captures(captured_at DESC);
CREATE INDEX ix_audit_events_device_id ON audit_events(device_id);
CREATE INDEX ix_audit_events_user_id ON audit_events(user_id);
CREATE INDEX ix_audit_events_request_id ON audit_events(request_id);
CREATE INDEX ix_audit_events_created_at ON audit_events(created_at DESC);
CREATE INDEX ix_feature_flag_overrides_flag_key ON feature_flag_overrides(flag_key);
CREATE INDEX ix_feature_flag_overrides_channel ON feature_flag_overrides(channel);
CREATE INDEX ix_feature_flag_overrides_device_id ON feature_flag_overrides(device_id);
CREATE INDEX ix_feature_flag_overrides_user_id ON feature_flag_overrides(user_id);
CREATE UNIQUE INDEX ux_feature_flag_overrides_channel_target
  ON feature_flag_overrides(flag_key, channel)
  WHERE channel IS NOT NULL AND device_id IS NULL AND user_id IS NULL;
CREATE UNIQUE INDEX ux_feature_flag_overrides_device_target
  ON feature_flag_overrides(flag_key, device_id)
  WHERE device_id IS NOT NULL AND user_id IS NULL;
CREATE UNIQUE INDEX ux_feature_flag_overrides_user_target
  ON feature_flag_overrides(flag_key, user_id)
  WHERE user_id IS NOT NULL;

COMMENT ON TABLE devices IS 'Anonymized device records; raw hardware identifiers must not be stored.';
COMMENT ON TABLE benchmark_sessions IS 'Consented cloud benchmark session metadata linked to users/devices when auth arrives.';
COMMENT ON TABLE audit_events IS 'Least-privilege cloud audit trail without raw IP addresses or user agents.';
COMMENT ON TABLE feature_flags IS 'Remote feature flags for dev, beta, and stable rollout decisions.';
