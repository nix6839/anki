// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_i18n::I18n;
use rusqlite::{types::FromSqlError, Error};
use std::str::Utf8Error;

use super::AnkiError;

#[derive(Debug, PartialEq)]
pub struct DbError {
    pub info: String,
    pub kind: DbErrorKind,
}

#[derive(Debug, PartialEq)]
pub enum DbErrorKind {
    FileTooNew,
    FileTooOld,
    MissingEntity,
    Corrupt,
    Locked,
    Utf8,
    Other,
}

impl AnkiError {
    pub(crate) fn db_error(info: impl Into<String>, kind: DbErrorKind) -> Self {
        AnkiError::DbError(DbError {
            info: info.into(),
            kind,
        })
    }
}

impl From<Error> for AnkiError {
    fn from(err: Error) -> Self {
        if let Error::SqliteFailure(error, Some(reason)) = &err {
            if error.code == rusqlite::ErrorCode::DatabaseBusy {
                return AnkiError::DbError(DbError {
                    info: "".to_string(),
                    kind: DbErrorKind::Locked,
                });
            }
            if reason.contains("regex parse error") {
                return AnkiError::InvalidRegex(reason.to_owned());
            }
        }
        AnkiError::DbError(DbError {
            info: format!("{:?}", err),
            kind: DbErrorKind::Other,
        })
    }
}

impl From<FromSqlError> for AnkiError {
    fn from(err: FromSqlError) -> Self {
        if let FromSqlError::Other(ref err) = err {
            if let Some(_err) = err.downcast_ref::<Utf8Error>() {
                return AnkiError::DbError(DbError {
                    info: "".to_string(),
                    kind: DbErrorKind::Utf8,
                });
            }
        }
        AnkiError::DbError(DbError {
            info: format!("{:?}", err),
            kind: DbErrorKind::Other,
        })
    }
}

impl DbError {
    pub fn localized_description(&self, _tr: &I18n) -> String {
        match self.kind {
            DbErrorKind::Corrupt => self.info.clone(),
            // fixme: i18n
            DbErrorKind::Locked => "Anki already open, or media currently syncing.".into(),
            _ => format!("{:?}", self),
        }
    }
}
