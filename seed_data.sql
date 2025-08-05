-- Insert some test users
INSERT INTO users (name, email) VALUES 
    ('Alice Dupont', 'alice@example.com'),
    ('Bob Martin', 'bob@example.com'),
    ('Claire Dubois', 'claire@example.com')
ON CONFLICT (email) DO NOTHING;
