use actix_web::{web, App, HttpResponse, HttpServer, Result};
use std::path::PathBuf;
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
};

async fn list_files(path: web::Path<(String,)>) -> Result<HttpResponse> {
    let base_dir = ".";
    let dir_path = if path.0.is_empty() {
        PathBuf::from(base_dir)
    } else {
        PathBuf::from(format!("{}/{}", base_dir, path.0.trim_start_matches("./")))
    };

    // 要排除的文件扩展名
    let excluded_extensions = vec![
        ".exe",
        ".lock",
        ".toml",
        ".dll",
        ".msi",
        ".md",
        ".png",
        ".jpg",
        ".jpeg",
        ".gif",
        ".webp",
        ".csv",
        ".xlsx",
        ".xls",
        ".docx",
        ".doc",
        ".pptx",
        ".ppt",
        ".pdf",
        ".xml",
        ".bat",
        ".vbs",
        ".git",
        ".gitignore",
    ];

    match fs::read_dir(&dir_path).await {
        Ok(mut entries) => {
            let mut file_list = String::new();
            file_list.push_str("<html><body><h1>File List</h1><ul>");

            while let Ok(Some(entry)) = entries.next_entry().await {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                let file_type = entry.file_type().await.unwrap();

                // Check if the file has an excluded extension
                if excluded_extensions
                    .iter()
                    .any(|ext| file_name_str.to_lowercase().ends_with(ext))
                {
                    continue;
                }

                if file_type.is_dir() {
                    file_list.push_str(&format!(
                        "<li><a href=\"/{}/\">{}</a></li>",
                        format!("{}{}", path.0.replace(r"//", "/"), file_name_str),
                        file_name_str
                    ));
                } else {
                    file_list.push_str(&format!(
                        "<li><a href=\"/{}\">{}</a></li>",
                        format!("{}{}", path.0.replace("//", "/"), file_name_str),
                        file_name_str
                    ));
                }
            }
            file_list.push_str("</ul></body></html>");
            Ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(file_list))
        }
        Err(_) => read_file(path).await,
    }
}

async fn read_file(path: web::Path<(String,)>) -> Result<HttpResponse> {
    let filename = &path.0;
    let filepath = PathBuf::from(format!("{}", filename));

    let mut file = match File::open(&filepath).await {
        Ok(file) => file,
        Err(_) => return Ok(HttpResponse::NotFound().body("File not found")),
    };

    let mut contents = Vec::new();
    if let Err(_) = file.read_to_end(&mut contents).await {
        return Ok(HttpResponse::InternalServerError().body("Could not read file"));
    }

    match String::from_utf8(contents) {
        Ok(contents) => Ok(HttpResponse::Ok()
            .content_type("text/plain; charset=utf-8")
            .body(contents)),
        Err(_) => Ok(HttpResponse::InternalServerError().body("File content is not valid UTF-8")),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 获取本机的私有IP地址
    let local_ip = match local_ip_address::local_ip() {
        Ok(ip) => ip,
        Err(e) => {
            eprintln!("Failed to get local IP address: {}", e);
            return Ok(());
        }
    };
    // 绑定的端口
    let port = 10999;
    println!(
        "Server is running at: http://{}:{} or http://127.0.0.1:{}",
        local_ip.to_string(),
        port,
        port
    );
    HttpServer::new(|| {
        App::new()
            .route("/?", web::get().to(list_files))
            .route("/{tail:.*}", web::get().to(list_files))
    })
    .bind(format!("0.0.0.0:{}", port))? // 监听所有 IPv4 地址
    .run()
    .await
}
