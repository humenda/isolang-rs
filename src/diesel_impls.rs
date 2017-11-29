use ::*;

use diesel::types::{FromSql, HasSqlType, IsNull, Text, ToSql, ToSqlOutput};
use diesel::backend::Backend;

use std::error::Error;
use std::io::Write;

expression_impls!(Text -> Language);
queryable_impls!(Text -> Language);

impl<DB: Backend<RawValue = [u8]>> FromSql<Text, DB> for Language {
    fn from_sql(bytes: Option<&DB::RawValue>) -> Result<Self, Box<Error + Send + Sync>> {
        let str_: String = FromSql::<Text, DB>::from_sql(bytes)?;
        Language::from_639_3(&str_)
            .ok_or("Unknown ISO 639-3 code".into())
    }
}

impl<DB> ToSql<Text, DB> for Language
where
    String: ToSql<Text, DB>,
    DB: Backend,
    DB: HasSqlType<Text>,
{
    fn to_sql<W: Write>(
        &self,
        out: &mut ToSqlOutput<W, DB>,
    ) -> Result<IsNull, Box<Error + Send + Sync>> {
        String::from(self.to_639_3()).to_sql(out)
    }
}