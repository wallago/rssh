use std::sync::{Arc, Mutex};

use rusqlite::{Connection, params};

use crate::{
    prelude::{Article, Category, Feed},
    utils::string_to_color,
};

pub fn get_db_conn() -> Option<Arc<Mutex<Connection>>> {
    let app_name = "rssh";
    let dir = std::path::PathBuf::from("/data/data/com.wallago.rssh/files");
    std::fs::create_dir_all(&dir)
        .unwrap_or_else(|e| panic!("failed to create data dir {dir:?}: {e}"));
    let conn = Connection::open(dir.join(format!("{app_name}.db")))
        .unwrap_or_else(|e| panic!("failed to get DB connection: {e}"));
    init_schema(&conn).expect("init schema");
    Some(Arc::new(Mutex::new(conn)))
}

pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY, 
            title TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS feeds (
            id INTEGER PRIMARY KEY, 
            title TEXT NOT NULL,
            category_id INTEGER NOT NULL, 
            error_count INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS entries (
            id INTEGER PRIMARY KEY, 
            feed_id INTEGER NOT NULL,
            title TEXT NOT NULL, 
            content TEXT, 
            url TEXT, 
            created_at TEXT, 
            status TEXT, 
            starred INTEGER NOT NULL
        );
    ",
    )
}

pub fn is_empty(conn: &Connection) -> rusqlite::Result<bool> {
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM entries", [], |r| r.get(0))?;
    Ok(n == 0)
}

pub fn write_all(
    conn: &mut Connection,
    cats: &[miniflux_api::models::Category],
    feeds: &[miniflux_api::models::Feed],
    entries: &[miniflux_api::models::Entry],
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    for c in cats {
        tx.execute(
            "INSERT OR REPLACE INTO categories (id,title) VALUES (?1,?2)",
            params![c.id, c.title],
        )?;
    }
    for f in feeds {
        tx.execute(
            "INSERT OR REPLACE INTO feeds (id,title,category_id,error_count) VALUES (?1,?2,?3,?4)",
            params![f.id, f.title, f.category.id, f.parsing_error_count],
        )?;
    }
    for e in entries {
        // only overwrite server-owned columns; ON CONFLICT preserves any future local-only column
        tx.execute(
            "INSERT INTO entries (id,feed_id,title,content,url,created_at,status,starred)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(id) DO UPDATE SET
                feed_id=excluded.feed_id, title=excluded.title, content=excluded.content, url=excluded.url,
                created_at=excluded.created_at, status=excluded.status, starred=excluded.starred",
            params![
                e.id,
                e.feed.id,
                e.title,
                e.content,
                e.url,
                e.created_at,
                e.status,
                e.starred
            ],
        )?;
    }
    tx.commit()
}

pub fn load_categories(conn: &Connection) -> rusqlite::Result<Vec<Category>> {
    let mut stmt = conn.prepare("SELECT id, title FROM categories ORDER BY title")?;
    let rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let label: String = row.get(1)?;
        Ok(Category {
            color: string_to_color(&label),
            id,
            label,
        })
    })?;
    rows.collect()
}

pub fn load_feeds(conn: &Connection) -> rusqlite::Result<Vec<Feed>> {
    let mut stmt = conn.prepare(
        "SELECT f.id, f.title, f.error_count, c.id, c.title
         FROM feeds f
         JOIN categories c ON c.id = f.category_id
         ORDER BY f.title",
    )?;
    let rows = stmt.query_map([], |row| {
        let cat_label: String = row.get(4)?;
        Ok(Feed {
            id: row.get::<_, i64>(0)?,
            label: row.get(1)?,
            error_count: row.get(2)?,
            category: Category {
                color: string_to_color(&cat_label),
                id: row.get::<_, i64>(3)?,
                label: cat_label,
            },
        })
    })?;
    rows.collect()
}

pub fn load_articles(conn: &Connection) -> rusqlite::Result<Vec<Article>> {
    let mut stmt = conn.prepare(
        "SELECT e.id, e.title, e.content, e.url, e.created_at, e.status, e.starred,
                f.id, f.title, f.error_count,
                c.id, c.title
         FROM entries e
         JOIN feeds f      ON f.id = e.feed_id
         JOIN categories c ON c.id = f.category_id
         ORDER BY e.created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let status: String = row.get(5)?;
        let cat_label: String = row.get(11)?;
        let content: String = row.get::<_, Option<String>>(2)?.unwrap_or_default();
        let url: String = row.get::<_, Option<String>>(3)?.unwrap_or_default();
        let snippet = make_snippet(&content);

        let category = Category {
            color: string_to_color(&cat_label),
            id: row.get::<_, i64>(10)?,
            label: cat_label,
        };
        let feed = Feed {
            id: row.get::<_, i64>(7)?,
            label: row.get(8)?,
            error_count: row.get(9)?,
            category,
        };
        Ok(Article {
            id: row.get::<_, i64>(0)?,
            feed,
            title: row.get(1)?,
            snippet,
            url,
            content,
            timestamp: {
                let raw = row.get::<_, Option<String>>(4)?.unwrap_or_default();
                let date = raw.get(0..10).unwrap_or(&raw);
                let time = raw.get(11..16).unwrap_or("");
                if time.is_empty() {
                    date.to_string()
                } else {
                    format!("{date} {time}")
                }
            },
            is_read: status == "read",
            is_starred: row.get::<_, i64>(6)? != 0,
        })
    })?;
    rows.collect()
}

fn make_snippet(html: &str) -> String {
    let text: String = html
        .split('<')
        .flat_map(|s| s.splitn(2, '>').nth(1))
        .collect::<Vec<_>>()
        .join(" ");
    let cleaned: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = cleaned.chars();
    let head: String = chars.by_ref().take(140).collect();
    if chars.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}
