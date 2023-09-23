use std::borrow::Cow;

use anyhow::Result;
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
use rusqlite::{params, Connection};
use uuid::Uuid;

pub struct Db {
    inner: Mutex<Option<Connection>>,
}

impl Db {
    pub fn new(connection: Connection) -> Self {
        Self {
            inner: Mutex::new(Some(connection)),
        }
    }

    pub fn is_active(&self) -> bool {
        self.inner.lock().is_some()
    }

    fn take(&self) -> Connection {
        let val = self.inner.lock().take();
        val.expect("No value to take")
    }

    fn lock(&self) -> MappedMutexGuard<'_, Connection> {
        MutexGuard::map(self.inner.lock(), |x: &mut Option<Connection>| {
            x.as_mut().expect("No value to take")
        })
    }
}

impl Db {
    pub fn init(&self) -> Result<()> {
        let mut this = self.lock();
        this.pragma_update(None, "journal_mode", "WAL")?;
        this.pragma_update(None, "synchronous", "NORMAL")?;

        let trans = this.transaction()?;
        for i in [include_str!("./sql/create_pastes.sql")] {
            trans.execute(i, [])?;
        }
        trans.commit()?;

        Ok(())
    }

    pub fn cleanup(&self) -> Result<()> {
        let this = self.take();
        this.pragma_update(None, "wal_checkpoint", "TRUNCATE")?;
        this.pragma_update(None, "optimize", "")?;
        this.pragma_update(None, "wal_checkpoint", "TRUNCATE")?;
        drop(this);

        Ok(())
    }
}

pub struct Paste {
    pub id: Uuid,
    pub name: String,
    pub paste: String,
    pub date: u64,
}

impl Db {
    pub fn new_paste(&self, paste: &str, name: &str) -> Result<Uuid> {
        let uuid = Uuid::new_v4();

        self.lock().execute(
            "INSERT INTO pastes (id, name, paste, date) VALUES (?1, ?2, ?3, strftime('%s', 'now'))",
            params![uuid, name, paste],
        )?;
        Ok(uuid)
    }

    pub fn get_paste(&self, uuid: Uuid) -> Result<Paste> {
        let paste = self.lock().query_row(
            "SELECT id, name, paste, date FROM pastes WHERE id = ?1",
            [uuid],
            |row| {
                Ok(Paste {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    paste: row.get(2)?,
                    date: row.get(3)?,
                })
            },
        )?;

        Ok(paste)
    }

    pub fn recent_pastes(&self, count: u64) -> Result<Vec<Paste>> {
        let db = self.lock();
        let mut stmt =
            db.prepare("SELECT id, name, paste, date FROM pastes ORDER BY date DESC LIMIT ?1")?;
        let pastes = stmt
            .query_map([count], |row| {
                Ok(Paste {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    paste: row.get(2)?,
                    date: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(pastes)
    }
}
