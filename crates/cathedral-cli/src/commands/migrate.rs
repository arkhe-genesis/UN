use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct MigrateArgs {
    #[command(subcommand)]
    pub command: MigrateCommand,
}

#[derive(Subcommand)]
pub enum MigrateCommand {
    Sqlite {
        #[arg(long, default_value = "cathedral.db")]
        database: String,
    },
    Postgres {
        #[arg(long)]
        url: String,
    }
}

pub async fn execute(args: MigrateArgs) -> Result<(), String> {
    match args.command {
        MigrateCommand::Sqlite { database } => {
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .connect(&database)
                .await
                .map_err(|e| e.to_string())?;

            sqlx::migrate!("../../migrations")
                .run(&pool)
                .await
                .map_err(|e| e.to_string())?;

            println!("✅ Migrações SQLite aplicadas no banco {}", database);
            Ok(())
        },
        MigrateCommand::Postgres { url } => {
            let pool = sqlx::postgres::PgPoolOptions::new()
                .connect(&url)
                .await
                .map_err(|e| e.to_string())?;

            sqlx::migrate!("../../migrations")
                .run(&pool)
                .await
                .map_err(|e| e.to_string())?;

            println!("✅ Migrações Postgres aplicadas no banco {}", url);
            Ok(())
        }
    }
}
