//! Based `perplexity.rs` but using `#[enum_def(table_name = "xxx")]`.

use chrono::{DateTime, Utc};
use sea_query::{ColumnDef, Expr, PostgresQueryBuilder, Query, SimpleExpr, Table};
use sea_query_attr::enum_def;
use sea_query_binder::SqlxBinder;

use sqlx::{PgConnection, PgPool, Row};
use tokio;

#[derive(Debug, sqlx::FromRow)]
#[enum_def(table_name = "users")]
struct User {
    #[allow(unused)]
    id: i32,
    name: String,
    email: String,
    age: Option<i32>,
    #[allow(unused)]
    created_at: DateTime<Utc>,
}

// #[derive(Iden)]
// enum Users {
//     Table,
//     Id,
//     Name,
//     Age,
//     Email,
//     CreatedAt,
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a connection pool
    let pool = PgPool::connect("postgres://testuser:testpassword@localhost:9999/testdb").await?;
    let mut pool_conn = pool.try_acquire().unwrap();
    let db_conn = &mut *pool_conn;

    let create_table_sql = create_users_table(db_conn).await?;
    println!("Created users table: {create_table_sql}");

    // Create a new user
    let new_user = User {
        id: i32::MIN,
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        age: Some(30),
        created_at: Utc::now(),
    };

    // Insert the new user
    let inserted_id = insert_user(db_conn, &new_user).await?;
    println!("Inserted user with ID: {}", inserted_id);

    // Read the user back from the database
    let retrieved_user = get_user_by_id(db_conn, inserted_id).await?;
    println!("Retrieved user: {:?}", retrieved_user);

    Ok(())
}

async fn create_users_table(
    db_conn: &mut PgConnection,
) -> Result<String, Box<dyn std::error::Error>> {
    // Schema

    let sql = Table::create()
        .table(UserIden::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(UserIden::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(UserIden::Name).string().not_null())
        .col(
            ColumnDef::new(UserIden::Email)
                .string()
                .not_null()
                .unique_key(),
        )
        .col(ColumnDef::new(UserIden::Age).integer())
        .col(
            ColumnDef::new(UserIden::CreatedAt)
                .timestamp_with_time_zone()
                .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string()))
                .not_null(),
        )
        .build(PostgresQueryBuilder);

    let result = sqlx::query(&sql).execute(db_conn).await;
    println!("Create table Users: {result:?}");
    Ok(sql)
}

async fn insert_user(
    db_conn: &mut PgConnection,
    user: &User,
) -> Result<i32, Box<dyn std::error::Error>> {
    let (sql, values) = Query::insert()
        .into_table(UserIden::Table)
        .columns([UserIden::Name, UserIden::Email, UserIden::Age])
        .values_panic([
            user.name.clone().into(),
            user.email.clone().into(),
            user.age.into(),
        ])
        .returning_col(UserIden::Id)
        .build_sqlx(PostgresQueryBuilder);

    let id: i32 = sqlx::query_with(&sql, values)
        .fetch_one(db_conn)
        .await?
        .try_get("id")?;

    Ok(id)
}

async fn get_user_by_id(
    db_conn: &mut PgConnection,
    id: i32,
) -> Result<User, Box<dyn std::error::Error>> {
    let (sql, values) = Query::select()
        .columns([
            UserIden::Id,
            UserIden::Name,
            UserIden::Email,
            UserIden::Age,
            UserIden::CreatedAt,
        ])
        .from(UserIden::Table)
        .and_where(Expr::col(UserIden::Id).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    // let row = sqlx::query_with(&sql, values).fetch_one(pool).await?;

    // let user = User {
    //     id: row.try_get("id")?,
    //     name: row.try_get("name")?,
    //     email: row.try_get("email")?,
    //     age: row.try_get("age")?,
    // };

    let user = sqlx::query_as_with::<_, User, _>(&sql, values)
        .fetch_one(db_conn)
        .await?;

    Ok(user)
}
