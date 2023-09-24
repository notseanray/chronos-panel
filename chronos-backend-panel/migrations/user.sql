CREATE TABLE users(
    id TEXT PRIMARY KEY ASC NOT NULL,
    discord_id TEXT NOT NULL,
    username TEXT NOT NULL,
    discriminator TEXT NOT NULL,
    avatar TEXT,
    verified INTEGER NOT NULL,
    email TEXT,
    accent_color INTEGER
);
