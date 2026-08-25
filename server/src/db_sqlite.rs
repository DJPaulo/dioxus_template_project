use anyhow::Error;
use argon2::PasswordHash;
use secrecy::SecretString;
use shared::PasswordResetRequest;
//use rusqlite::{params, Connection};
use sqlx::{SqlitePool, query, query_as, query_scalar};
use shared::{Role, User};
use std::path::Path;
use std::fs;
use std::env;
//use crate::security;
use crate::security::{hash_password, verify_password};

const DB_FILENAME: &str = "users.db";
const EMBEDDED_DB_FULLPATH: &[u8] = include_bytes!("sql/sqlite/users.db");  // Embed the initial SQLite DB file into the binary


// Connect to the SQLite database, creating it if it doesn't exist
pub async fn connect_or_create_database() -> Result<SqlitePool, Error> {

    let email = &env::var("RUST_ADMIN_EMAIL")?;
    let password = &env::var("RUST_ADMIN_PASSWORD")?;
    let parent_dir = &env::var("RUST_DB_PATH")?;
    let full_path = format!("{}{}", parent_dir, DB_FILENAME);
    let db_path = Path::new(&full_path);

    let db_exists: bool = db_path.exists();
    if !db_exists {
        // Create a sqlite database if one doesn't already exist
        fs::write(&db_path, EMBEDDED_DB_FULLPATH)?;
        println!("  - Created new SQLite DB at {}", db_path.display());
    }

    let sqlite_path = db_path.to_string_lossy().to_string();

    let url = format!("sqlite://{}", sqlite_path);
    let pool = SqlitePool::connect(&url).await?;
    println!("  - Connected to SQLite DB at {}", db_path.display());

    if !db_exists {
        

        // Create an admin user for the newly created database. This initial admin user must be deleted once a proper one has been created.
        let hashed_password = hash_password(&SecretString::from(password.to_string())).await?;
        let result = query("INSERT INTO user (username, email, role, password_hash, is_active, change_password)
            VALUES ('Initial Admin User', ?1, 'Admin', ?2, 1, 1);")
            .bind(email)
            .bind(hashed_password)
            .execute(&pool)
            .await?;
        if result.rows_affected() == 1 {
            println!("  - Initial admin user created");
        }
        else
        {
            return Err(anyhow::anyhow!("Failed to create initial admin user"));
        }
    }

    Ok(pool)
}

// Authenticate a user by email and password
pub async fn authenticate_user(db: &sqlx::SqlitePool, email: &str, password: &SecretString) -> Result<Option<shared::User>, Error> {
    let row = query_as::<_, (i32, String, String, String, String, bool, bool)>(
        "SELECT id, username, email, role, password_hash, is_active, change_password FROM user WHERE email = ?1;"
    )
    .bind(email)
    .fetch_optional(db)
    .await?;

    if let Some(user_row) = row {

        let parsed_hash = match PasswordHash::new(&user_row.4) {
            Ok(hash) => hash,
            Err(_) => return Ok(None), // Invalid hash format
        };
        if !verify_password(password, &parsed_hash.to_string()).await? {
            return Ok(None);
        }
        
        let role = match user_row.3.as_str() {
            "Admin" => Role::Admin,
            _ => Role::User,
        };

        let user = User {
            id: user_row.0,
            username: user_row.1,
            email: user_row.2,
            role: role,
            is_active: user_row.5,
            change_password: user_row.6,
        };

        Ok(Some(user))
    } else {
        // Using a dummy hash to prevent timing attacks
        let dummy_hash: &str = "$argon2id$v=19$m=65536,t=2,p=1$gQB2vxiVLKZko9wUDSYWsQ$9RW64sFEYqcLU+HTKRk7J8q4Tjn2NoiuE8F/Hxb3F0M";

        let parsed_hash = match PasswordHash::new(dummy_hash) {
            Ok(hash) => hash,
            Err(_) => return Ok(None), // Invalid hash format
        };

        verify_password(password, &parsed_hash.to_string()).await?;

        Ok(None)
    }
}


// Create a new user session
pub async fn create_user_session(db: &SqlitePool, session_id: &str, user_id: i32) -> Result<usize, Error> {
    let result = query(
        "INSERT INTO user_session (session_id, user_id, expires_at) VALUES (?1, ?2, DATETIME(CURRENT_TIMESTAMP, '+1 hour'));",
    )
        .bind(session_id)
        .bind(user_id)
        .execute(db)
        .await?;
    Ok(result.rows_affected() as usize)
}


// Get a single user by session ID
pub async fn get_user_by_session(db: &SqlitePool, session_id: &str) -> Result<Option<User>, Error> {
    // Retrieve the user associated with the session
    let row = query_as::<_, (i32, String, String, String, bool, bool)>(
        "SELECT U.id, U.username, U.email, U.role, U.is_active, U.change_password
         FROM user_session US
         INNER JOIN user U
         ON U.id = US.user_id
         WHERE US.session_id = ?1
         AND US.expires_at > CURRENT_TIMESTAMP;"
    )
    .bind(session_id)
    .fetch_optional(db)
    .await?;

    if let Some(user_row) = row {

        let role = match user_row.3.as_str() {
            "Admin" => Role::Admin,
            _ => Role::User,
        };

        let user = User {
            id: user_row.0,
            username: user_row.1,
            email: user_row.2,
            role: role,
            is_active: user_row.4,
            change_password: user_row.5,
        };

        Ok(Some(user))
    }
    else {
        Ok(None)
    }
}


// Delete a session for a user by session ID, plus any expired sessions across all users (as a housekeeping measure)
pub async fn delete_user_session(db: &SqlitePool, session_id: &str) -> Result<usize, Error> {
    let result = query(
        "DELETE FROM user_session WHERE session_id = ?1 OR expires_at <= CURRENT_TIMESTAMP;",
    )
        .bind(session_id)
        .execute(db)
        .await?;
    Ok(result.rows_affected() as usize)
}


// Get a list of all users
pub async fn get_all_users(db: &SqlitePool) -> Result<Option<Vec<User>>, Error> {
    // Retrieve a list of all users
    let rows = query_as::<_, (i32, String, String, String, bool, bool)>(
        "SELECT id, username, email, role, is_active, change_password FROM user"
    )
    .fetch_all(db)
    .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let users: Vec<User> = rows
        .into_iter()
        .map(|(id, username, email, role_str, is_active, change_password)| {
            let role = match role_str.as_str() {
                "Admin" => Role::Admin,
                _ => Role::User,
            };

            User {
                id,
                username,
                email,
                role,
                is_active,
                change_password,
            }
    })
    .collect();

    Ok(Some(users))

}


// Save a user (insert or update)
pub async fn save_user(db: &SqlitePool, user: User) -> Result<usize, Error> {
    // A user ID of -1 indicates a new (or "unknown") user
    if user.id == -1 {
        // Insert new user
        let password_hash = hash_password(&SecretString::from("123")).await?;   // This should be replaced with a proper password handling mechanism in a real application                               
        let result = query(
            "INSERT INTO user (username, email, role, password_hash, is_active, change_password)
             VALUES (?1, ?2, ?3, ?4, ?5, 1);"
        )
        .bind(user.username)
        .bind(user.email)
        .bind(match user.role {
            Role::Admin => "Admin",
            Role::User => "User",
        })
        .bind(password_hash)
        .bind(user.is_active)
        .execute(db)
        .await?;
        Ok(result.rows_affected() as usize)
    } else {
        // Update existing user
        let result = query(
            "UPDATE user SET username = ?1, role = ?2, is_active = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?4;"
        )
        .bind(user.username)
        .bind(match user.role {
            Role::Admin => "Admin",
            Role::User => "User",
        })
        .bind(user.is_active)
        .bind(user.id)
        .execute(db)
        .await?;
        Ok(result.rows_affected() as usize)
    }

}


// Delete a single user by user ID
pub async fn delete_user(db: &SqlitePool, user_id: i32) -> Result<usize, Error> {
    let result = query(
        "DELETE FROM user WHERE id = ?1;"
    )
        .bind(user_id)
        .execute(db)
        .await?;

    Ok(result.rows_affected() as usize)
}


// Insert reset token
pub async fn insert_reset_token(db: &SqlitePool, user_email: &str, token: &str) -> sqlx::Result<bool> {

    let _result = query(
        "INSERT INTO user_password_token (user_id, token, expires_at) SELECT id, ?1, DATETIME(CURRENT_TIMESTAMP, '+15 minute') FROM user WHERE email = ?2;",
    )
    .bind(token)
    .bind(user_email)
    .execute(db)
    .await?;

    Ok(true)
}


// Get a single user by session ID
pub async fn get_user_by_token(db: &SqlitePool, token: &str) -> Result<Option<User>, Error> {
    // Retrieve the user associated with the session
    let row = query_as::<_, (i32, String, String, String, bool, bool)>(
        "SELECT U.id, U.username, U.email, U.role, U.is_active, U.change_password
         FROM user_password_token UPT
         INNER JOIN user U
         ON U.id = UPT.user_id
         WHERE UPT.token = ?1
         AND UPT.expires_at > CURRENT_TIMESTAMP
         AND UPT.used = 0;"
    )
    .bind(token)
    .fetch_optional(db)
    .await?;

    if let Some(user_row) = row {

        let role = match user_row.3.as_str() {
            "Admin" => Role::Admin,
            _ => Role::User,
        };

        let user = User {
            id: user_row.0,
            username: user_row.1,
            email: user_row.2,
            role: role,
            is_active: user_row.4,
            change_password: user_row.5,
        };

        Ok(Some(user))
    }
    else {
        Ok(None)
    }
}



// Change a user's password by reset request
pub async fn reset_password(db: &SqlitePool, req: &PasswordResetRequest) -> Result<bool, Error> {

    // If a token is passed, check the validity
    if !req.token.is_empty() {
        match get_user_by_token(db, &req.token).await? {
            Some(u) => {
                if u.id != req.user.id {    // If a token is valid, make sure it's for the specified user
                    return Ok(false)
                }
            },
            None => return Ok(false)
        }
    } else {
        return Ok(false)
    }
    
    let psw = &SecretString::from(req.new_password.clone());

    let password_hash = match hash_password(psw).await {
        Ok(hash) => hash,
        Err(_) => return Ok(false), // Invalid hash format
    };
    
    let mut tx = db.begin().await?;
    query(
        "UPDATE user SET password_hash = ?1, change_password = 0 WHERE id = ?2;"
    )
        .bind(password_hash)
        .bind(req.user.id)
        .execute(&mut *tx)
        .await?;

    query(
        "UPDATE user_password_token SET used = 1 WHERE user_id = ?1 AND token = ?2;"
    )
        .bind(req.user.id)
        .bind(&req.token)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(true)
}


// Change a user's password by user ID
pub async fn change_password(db: &SqlitePool, user_id: i32, old_password: &SecretString, new_password: &SecretString) -> Result<bool, Error> {
    println!("change 1");
    let row = query_scalar::<_, String>(
        "SELECT password_hash FROM user WHERE id = ?1;"
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?;
    println!("change 2");
    //let hashed_old_password = hash_password(old_password).await?;
    if let Some(password_hash) = row {
        let db_parsed_hash = match PasswordHash::new(&password_hash) {
            Ok(hash) => hash,
            Err(_) => return Ok(false), // Invalid hash format
        };
        if !verify_password(&old_password, &db_parsed_hash.to_string()).await? {
            return Ok(false);
        }
    }
    let hashed_new_password = match hash_password(new_password).await {
        Ok(hash) => hash,
        Err(_) => return Ok(false), // Invalid hash format
    };

    let _result = query(
        "UPDATE user SET password_hash = ?1, change_password = 0 WHERE id = ?2;"
    )
        .bind(hashed_new_password)
        .bind(user_id)
        .execute(db)
        .await?;

    Ok(true)
}

