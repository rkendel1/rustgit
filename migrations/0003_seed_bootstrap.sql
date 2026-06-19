INSERT INTO agents (agent_id, capabilities, last_seen, status)
VALUES (
    'bootstrap-agent',
    '["schema-management", "history-queries"]'::jsonb,
    NOW(),
    'active'
)
ON CONFLICT (agent_id) DO NOTHING;
