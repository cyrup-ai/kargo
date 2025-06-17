# Struct `Table`

A wrapper type to assert that you ment to use a string as a table name.
To prevent some possible errors, by defauit [`IntoResource`] does not allow `:` in table names
as this might be an indication that the user might have intended to use a record id instead.
If you wrap your table name string in this tupe the [`IntoResource`] trait will accept any
table names.

## Fields

Field information will be available in a future version.

