use failure::Error;
use db::{SqliteBackend, DBMetadata};

pub trait DBBackend {
    type DBError;
    fn metadata(&self) -> Result<DBMetadata, Self::DBError>;
    fn close(self) -> Result<(), Self::DBError>;
}

impl DBBackend for SqliteBackend {
    type DBError = Error;

    fn metadata(&self) -> Result<DBMetadata, Error> {
        let (version, date_created) = self.connection.query_row(
            "SELECT version, date_created FROM metadata WHERE id = 1",
            &[],
            |row| {
                let version = row.get(0);
                let date_created = row.get(1);
                (version, date_created)
            }
        ).map_err(|e| format_err!("Error getting metadata from database: {}", e))?;
        Ok(
            DBMetadata {
                version: version,
                date_created: date_created,
            }
        )
    }

    fn close(self) -> Result<(), Error> {
        self.connection.close().map_err(|(_,e)| e.into())
    }
}

