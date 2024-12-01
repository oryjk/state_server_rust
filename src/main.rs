#[macro_use]
extern crate rocket;
mod client;

use crate::client::routes::{process_status, AppState, Status};
use dotenv::dotenv;
use rocket::fairing::AdHoc;
use rocket::serde::Deserialize;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc;

// 初始化数据库连接池
async fn init_db_pool() -> MySqlPool {
    dotenv().ok(); // 加载 .env 文件
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = MySqlPoolOptions::new()
        .max_connections(20) // 设置连接池大小为 20
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    pool
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok(); // 加载 .env 文件
    // 从环境变量中获取端口号，默认为 8000
    let port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .expect("Invalid port number");

    let (sender, receiver) = mpsc::channel::<Status>(1000); // 队列大小为100
    let pool = init_db_pool().await;
    let pool_clone = pool.clone(); // 克隆连接池
    let app_state = Arc::new(AppState { pool, sender });
    tokio::spawn(process_status(pool_clone, receiver));
    rocket::build()
        .configure(rocket::Config {
            port, // 设置 Rocket 的端口
            ..Default::default()
        })
        .attach(AdHoc::on_ignite("MySQL Pool", |rocket| async {
            let pool = init_db_pool().await; // 初始化数据库连接池
            rocket.manage(pool) // 将连接池注入到 Rocket 的全局状态中
        }))
        .manage(app_state)
        .mount("/client", client::routes())
}


