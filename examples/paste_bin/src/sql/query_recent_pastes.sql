SELECT id,
    name,
    paste,
    date
FROM pastes
ORDER BY date DESC
LIMIT ?1