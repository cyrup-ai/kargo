# Struct `Notification`

A live query notification
Live queries return a stream of notifications. The notification contains an `action` that triggered the change in the database record and `data` itself.
For deletions the data is the record before it was deleted. For everything else, it's the newly created record or updated record depending on whether
the action is create or update.

## Fields

Field information will be available in a future version.

