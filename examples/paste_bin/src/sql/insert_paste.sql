INSERT INTO pastes (id, name, paste, date)
VALUES (?1, ?2, ?3, strftime('%s', 'now'))