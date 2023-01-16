create table messages
(
    id           integer primary key autoincrement,
    text         text not null,
    font_color   text,
    font_family  text
--     created_at datetime default current_timestamp,
--     expires_at datetime default (datetime(current_timestamp + 604800)) -- 7 days
)
