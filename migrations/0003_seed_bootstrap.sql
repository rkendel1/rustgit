INSERT INTO users (user_id, email, name, auth_provider, created_at)
VALUES ('user-bootstrap', 'bootstrap@trythissoftware.com', 'Bootstrap User', 'github', NOW())
ON CONFLICT (user_id) DO NOTHING;

INSERT INTO organizations (
    org_id,
    name,
    slug,
    plan,
    max_workspaces,
    max_concurrent_executions,
    max_runtime_minutes,
    created_at
)
VALUES ('org-bootstrap', 'Bootstrap Org', 'bootstrap-org', 'free', 3, 5, 1000, NOW())
ON CONFLICT (org_id) DO NOTHING;

INSERT INTO memberships (user_id, org_id, role)
VALUES ('user-bootstrap', 'org-bootstrap', 'owner')
ON CONFLICT (user_id, org_id) DO NOTHING;

INSERT INTO agents (agent_id, capabilities, last_seen, status)
VALUES (
    'bootstrap-agent',
    '["schema-management", "history-queries"]'::jsonb,
    NOW(),
    'active'
)
ON CONFLICT (agent_id) DO NOTHING;
