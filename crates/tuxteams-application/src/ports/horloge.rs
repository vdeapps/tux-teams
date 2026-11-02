//! Port de l'horloge : heure courante et attente, remplaçables dans les tests.

use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait Horloge: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
    async fn sleep(&self, duree: Duration);
}
