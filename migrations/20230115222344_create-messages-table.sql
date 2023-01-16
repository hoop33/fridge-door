create table messages
(
    id          integer primary key autoincrement,
    text        text not null,
    font_color  text,
    font_family text,
    created_at  datetime default current_timestamp not null,
    expires_at  datetime default (datetime('now', '+7 days')) not null
)
