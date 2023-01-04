use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio_postgres::{Client, Connection, Socket, NoTls, tls::NoTlsStream, Row};
use uuid::Uuid;

pub struct Database {
    address: String,
    client: Client,
    connection: Connection<Socket, NoTlsStream>
}

impl Database {
    pub async fn new(addr: &str) -> Result<Database, tokio_postgres::Error> {
        let client = tokio_postgres::connect(addr, NoTls).await?;

        Ok(Database {
            address: addr.to_string(),
            client: client.0,
            connection: client.1
        })
    }

    pub async fn postgres_client(&self) -> &tokio_postgres::Client {
        &self.client
    }

    pub async fn postgres_client_mut(&mut self) -> &mut tokio_postgres::Client {
        &mut self.client
    }

    pub async fn get_row_direct(&self, table: &str, row: u64) -> Option<Row> {
        self.client.execute(format!("SELECT * FROM {} WHERE row_number() == {}", table, row).as_str(), &[]);

        None
    }

    pub async fn get_all<T: DatabaseObject>(&self) -> Option<Vec<T>> {
        self.client.query("SELECT * FROM $1", &[ &T::table_name().to_owned() ]).await.ok()
        .map(|x| {
            x.into_iter()
            .map(|row| T::from_row(row))
            .flatten()
            .collect::<Vec<T>>()
        })
    }
}

#[async_trait]
pub trait DatabaseObject: Sized {
    async fn load(db: &Client, id: Uuid) -> Option<Self>;
    fn from_row(row: Row) -> Option<Self>;
    fn id(&self) -> Uuid;
    fn table_name() -> &'static str;
    async fn commit(&self, db: &mut Client) -> bool;
}

pub struct UserAltName {
    id: Uuid,
    pub user: Uuid,
    pub nickname: String,
    pub added: i64,
}

#[async_trait]
impl DatabaseObject for UserAltName {
    async fn load(db: &Client, id: Uuid) -> Option<Self> {
        if let Ok(row) = db.query_one("SELECT * FROM user_alt_name WHERE id = $1", &[ &id.to_string() ]).await {
            Self::from_row(row)
        } else {
            None
        }
    }

    fn table_name() -> &'static str {
        "user_alt_name"
    }

    fn from_row(row: Row) -> Option<Self> {
        let instance = UserAltName {
            id: Uuid::from_str(row.try_get(0).ok()?).ok()?,
            user: Uuid::from_str(row.try_get(1).ok()?).ok()?,
            nickname: row.try_get(2).ok()?,
            added: row.try_get(3).ok()?
        };

        Some(instance)
    }

    fn id(&self) -> Uuid {
        self.id
    }

    async fn commit(&self, db: &mut Client) -> bool {
        if let Err(_) = db.execute("UPDATE user_alt_name SET user = $2, nickname = $3, added = $4 WHERE id = $1", 
            &[ &self.id.to_string(), &self.user.to_string(), &self.nickname, &self.added ]).await {

            db.execute("INSERT INTO user_alt_name (id, user, nickname, added) VALUES $1 $2 $3 $4", 
                &[ &self.id.to_string(), &self.user.to_string(), &self.nickname.to_string(), &self.added ]).await.is_ok()
        } else {
            true
        }
    }
}

pub struct User {
    id: Uuid,
    username: String,
    password: String,
}

pub struct Device {
    id: Uuid,
    pub user: Uuid,
    pub name: String,
}

pub struct Message {
    id: Uuid,
    pub user: Uuid,
    pub message: String,
    pub date: DateTime<Utc>,
}
