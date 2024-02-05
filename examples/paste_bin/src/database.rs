use anyhow::Result;
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
use rusqlite::{params, Connection};
use uuid::Uuid;

pub struct Db {
    // Hold a reference to the database connection
    inner: Mutex<Option<Connection>>,
}

impl Db {
    pub fn new(connection: Connection) -> Self {
        Self {
            inner: Mutex::new(Some(connection)),
        }
    }

    // Check if we still have a connection
    // The connection is dropped when calling .cleanup()
    pub fn is_active(&self) -> bool {
        self.inner.lock().is_some()
    }

    // Take the connection out of the mutex
    fn take(&self) -> Connection {
        let val = self.inner.lock().take();
        val.expect("No value to take")
    }

    // Lock the mutex and map the value to a mutable reference to the connection
    fn lock(&self) -> MappedMutexGuard<'_, Connection> {
        MutexGuard::map(self.inner.lock(), |x: &mut Option<Connection>| {
            x.as_mut().expect("No value to take")
        })
    }
}

impl Db {
    // Setup the database
    // - Set the journal mode to WAL
    // - Set the synchronous mode to NORMAL
    // - Create the pastes table
    pub fn init(&self) -> Result<()> {
        let this = self.lock();
        this.pragma_update(None, "journal_mode", "WAL")?;
        this.pragma_update(None, "synchronous", "NORMAL")?;
        this.execute(include_str!("./sql/create_pastes.sql"), [])?;

        Ok(())
    }

    // Cleanup the database
    // - Set the journal mode to DELETE
    // - Run a checkpoint
    // - Run an optimize
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
    // Add a new paste to the database
    pub fn new_paste(&self, paste: &str, name: &str) -> Result<Uuid> {
        let uuid = Uuid::new_v4();

        self.lock().execute(
            include_str!("sql/insert_paste.sql"),
            params![uuid, name, paste],
        )?;
        Ok(uuid)
    }

    // Get a paste from the database
    pub fn get_paste(&self, uuid: Uuid) -> Result<Paste> {
        let paste = self
            .lock()
            .query_row(include_str!("sql/query_paste.sql"), [uuid], |row| {
                Ok(Paste {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    paste: row.get(2)?,
                    date: row.get(3)?,
                })
            })?;

        Ok(paste)
    }

    // Get a list of `count` recent pastes from the database
    pub fn recent_pastes(&self, count: u64) -> Result<Vec<Paste>> {
        let db = self.lock();
        let mut stmt = db.prepare(include_str!("sql/query_recent_pastes.sql"))?;
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
