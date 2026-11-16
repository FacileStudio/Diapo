use porte::directory::{DirectoryError, PersonDeleted, StateStore};
use sqlx::{PgConnection, PgPool};
use tronc::mcp::erase_pending;

/// The app's half of a GDPR erase: personal decks, pending agent actions, memberships, then the
/// account row.
#[derive(Clone)]
pub struct AppData {
    pool: PgPool,
}

impl AppData {
    /// An eraser over `pool`.
    pub fn new(pool: PgPool) -> Self {
        AppData { pool }
    }
}

fn failed(error: sqlx::Error) -> DirectoryError {
    DirectoryError::Failed(error.to_string())
}

async fn erase(tx: &mut PgConnection, users: &[i64], subject: &str) -> Result<(), DirectoryError> {
    sqlx::query("DELETE FROM decks WHERE space_id IS NULL AND owner_id = ANY($1)")
        .bind(users)
        .execute(&mut *tx)
        .await
        .map_err(failed)?;
    let names: Vec<String> = users.iter().map(i64::to_string).collect();
    erase_pending(&mut *tx, &names).await.map_err(|error| DirectoryError::Failed(error.to_string()))?;
    sqlx::query("DELETE FROM space_members WHERE subject = $1")
        .bind(subject)
        .execute(&mut *tx)
        .await
        .map_err(failed)?;
    sqlx::query("DELETE FROM porte_users WHERE id = ANY($1)").bind(users).execute(&mut *tx).await.map_err(failed)?;
    Ok(())
}

#[async_trait::async_trait]
impl PersonDeleted for AppData {
    async fn on_person_deleted(&self, provider: &str, subject: &str) -> Result<(), DirectoryError> {
        let mut tx = self.pool.begin().await.map_err(failed)?;
        let users: Vec<i64> =
            sqlx::query_scalar("SELECT user_id FROM porte_identities WHERE provider = $1 AND subject = $2")
                .bind(provider)
                .bind(subject)
                .fetch_all(&mut *tx)
                .await
                .map_err(failed)?;
        erase(&mut tx, &users, subject).await?;
        tx.commit().await.map_err(failed)
    }
}

/// The deletion cursor, in the one-row `deletion_state` table.
#[derive(Clone)]
pub struct DeletionState {
    pool: PgPool,
}

impl DeletionState {
    /// A state store over `pool`.
    pub fn new(pool: PgPool) -> Self {
        DeletionState { pool }
    }
}

#[async_trait::async_trait]
impl StateStore for DeletionState {
    async fn load(&self) -> Result<Vec<u8>, DirectoryError> {
        let state: Option<Vec<u8>> = sqlx::query_scalar("SELECT state FROM deletion_state WHERE id = 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(failed)?;
        Ok(state.unwrap_or_default())
    }

    async fn save(&self, state: &[u8]) -> Result<(), DirectoryError> {
        sqlx::query(
            "INSERT INTO deletion_state (id, state, updated_at) VALUES (1, $1, now())
             ON CONFLICT (id) DO UPDATE SET state = EXCLUDED.state, updated_at = now()",
        )
        .bind(state)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(failed)
    }
}
