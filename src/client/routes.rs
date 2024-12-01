use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{get, post};
use sqlx::MySqlPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::time::sleep;

#[derive(Deserialize)]
pub struct Status {
    client_id: String,
    status: String,
}

pub struct AppState {
    pub pool: MySqlPool,
    pub sender: mpsc::Sender<Status>,
}


#[post("/status", format = "json", data = "<status>")]
pub async fn receive_status(
    status: Json<Status>,
    state: &rocket::State<Arc<AppState>>,
) -> &'static str {
    info!("Start status from client: {:?}", &status.0.client_id);
    state.sender.send(status.into_inner()).await.unwrap();
    // info!("End status from client: {:?}", status.0.client_id);
    "Status received"
}

pub async fn process_status(
    db_pool: MySqlPool,
    mut receiver: mpsc::Receiver<Status>,
) {
    let mut batch = Vec::new();
    let mut last_insert_time = tokio::time::Instant::now();

    loop {
        tokio::select! {
            // 等待新数据加入队列
            status = receiver.recv() => {
                if let Some(status) = status {
                    batch.push(status);
                }
            }

            // 每隔一定时间就插入一次数据库（例如每 100 毫秒）
            _ = sleep(Duration::from_millis(100)) => {
                if !batch.is_empty() {
                    // 批量插入数据库
                    let mut query = sqlx::query("INSERT INTO state_server.ss_client_state_log (client_id, status) VALUES ");
                    for status in &batch {
                        query = query.bind(&status.client_id).bind(&status.status);
                    }

                    query.execute(&db_pool).await.unwrap();
                    batch.clear(); // 清空批次数据
                    last_insert_time = tokio::time::Instant::now(); // 更新插入时间
                }
            }
        }

        // 如果等待的时间超过某个阈值（例如 500 毫秒），就强制插入数据库
        if last_insert_time.elapsed() > Duration::from_millis(500) && !batch.is_empty() {
            let mut placeholders = vec![];
            let mut params = vec![];
            let mut query_str = String::from("INSERT INTO ss_client_state_log (client_id, status) VALUES ");
            for status in &batch {
                placeholders.push(format!("(?, ?)"));
                params.push(status.client_id.clone());
                params.push(status.status.clone());

            }
            info!("query_str: {:?}", &query_str);
            query_str.push_str(&placeholders.join(", "));
            info!("query_str: {:?}", &query_str);
            // 将动态查询转换为可执行查询
            let mut query = sqlx::query(&query_str);
            for param in params {
                query = query.bind(param);
            }

            query.execute(&db_pool).await.unwrap();
            batch.clear();
            last_insert_time = tokio::time::Instant::now(); // 更新插入时间
        }
    }
}