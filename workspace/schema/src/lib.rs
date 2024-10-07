mod schema;
pub use schema::*;

pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("./migrations");
pub const MIGRATIONS_ASYNC: diesel_async_migrations::EmbeddedMigrations =
    diesel_async_migrations::embed_migrations!("./migrations");
