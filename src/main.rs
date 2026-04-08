use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    serve,
};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, sync::RwLock};

#[tokio::main]
async fn main() {
    let r = fs::read("config.json").expect("fail to read config");
    let cfg: BaseCfg = serde_json::from_slice(r.as_slice()).expect("fail to serialize cfg");
    let mut map = HashMap::new();
    for i in cfg.mounts {
        println!("{:?}", i);
        map.insert(i.virt.clone(), i);
    }

    let router = Router::new()
        .route("/{virt}/{*reflect}", get(route_file))
        .route("/{virt}", get(route_file))
        .with_state(Arc::new(RwLock::new(map)));
    let listener = TcpListener::bind(cfg.host)
        .await
        .expect("fail to bind server");
    serve(listener, router)
        .await
        .expect("fail to serve service");
}

async fn route_file(
    State(state): State<Arc<RwLock<HashMap<String, MountPoint>>>>,
    Path(qpath): Path<QPath>,
) -> Result<impl IntoResponse, StatusCode> {
    // 1. 获取映射基础路径
    let (real_base, reflect_path) = {
        let map = state.read().await;
        let r = map.get(&qpath.virt).ok_or(StatusCode::NOT_FOUND)?;
        (r.real.clone(), qpath.reflect.clone())
    }; // 尽早释放读锁

    let full_path = PathBuf::from(real_base).join(reflect_path.unwrap_or_default());
    println!("{:?}", full_path);
    // 2. 判断路径类型
    if full_path.is_file() {
        let file = tokio::fs::File::open(&full_path)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;

        // 【核心】根据路径猜测 MIME 类型，猜不到则默认为 application/octet-stream
        let mime = mime_guess::from_path(&full_path).first_or_octet_stream();

        let body = axum::body::Body::from_stream(tokio_util::io::ReaderStream::new(file));

        // 返回时带上 Content-Type 响应头
        return Ok(([(axum::http::header::CONTENT_TYPE, mime.as_ref())], body).into_response());
    } else if full_path.is_dir() {
        // 读取目录并返回字符串列表
        let mut entries = tokio::fs::read_dir(full_path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let mut files = Vec::new();

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        {
            files.push(entry.file_name().to_string_lossy().into_owned());
        }

        Ok(Json(files).into_response())
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct MountPoint {
    virt: String,
    real: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct BaseCfg {
    host: String,
    mounts: Vec<MountPoint>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
struct QPath {
    virt: String,
    reflect: Option<String>,
}
