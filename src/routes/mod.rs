
pub mod admin;
pub mod editor;
pub mod files;
pub mod generate;
pub mod page;

use crate::AppState;

use axum::{
    body::Body,
    http::{header::LOCATION, HeaderValue, Request, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};

use tower_http::services::ServeDir;

/// 修正 Zola Blog 靜態目錄的重新導向網址。
///
/// 例如：
/// ServeDir 原本回傳 Location: /tags/
/// 經過這個 middleware 後，改成 Location: /blog/tags/
///
/// 只處理 Blog Router 回傳的重新導向，
/// 不影響 CMS 其他路由及 /public。
/* http ok but not https
async fn fix_blog_redirect(
    request: Request<Body>,
    next: Next,
) -> Response {
    // 先讓 ServeDir 正常處理請求
    let mut response = next.run(request).await;

    // 只處理 3xx 重新導向回應
    if response.status().is_redirection() {
        // 取得 ServeDir 回傳的 Location
        let location = response
            .headers()
            .get(LOCATION)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);

        if let Some(location) = location {
            // 只處理以 / 開頭的站內絕對路徑
            // 並避免重複加入 /blog
            if location.starts_with("/")
                && location != "/blog"
                && !location.starts_with("/blog/")
            {
                let corrected_location =
                    format!("/blog{}", location);

                if let Ok(value) =
                    HeaderValue::from_str(&corrected_location)
                {
                    response
                        .headers_mut()
                        .insert(LOCATION, value);
                }
            }
        }
    }

    response
}
*/

/// 修正 Zola Blog 靜態目錄的重新導向網址。
///
/// ServeDir 在 nest("/blog", ...) 下收到的是已剝除前綴的路徑，
/// 因此目錄尾斜線重新導向會產生 Location: /tags/（或帶 scheme 的絕對 URL）。
/// 此 middleware 會把路徑前綴補回 /blog，同時相容：
/// - 路徑形式：/tags/  →  /blog/tags/
/// - 絕對 URL：https://localhost:5000/tags/  →  https://localhost:5000/blog/tags/
///
/// 這解決了「HTTP 正常、HTTPS 卻跳到 /tags」的差異（HTTPS/HTTP2 較常產生完整 URI）。
async fn fix_blog_redirect(
    request: Request<Body>,
    next: Next,
) -> Response {
    // 先讓 ServeDir 正常處理請求
    let mut response = next.run(request).await;

    // 只處理 3xx 重新導向
    if !response.status().is_redirection() {
        return response;
    }

    // 取得 Location
    let Some(location) = response
        .headers()
        .get(LOCATION)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
    else {
        return response;
    };

    // 嘗試解析為完整 URI（可能含 scheme + authority）
    let corrected = if let Ok(uri) = location.parse::<Uri>() {
        let path = uri.path();
        // 只處理站內絕對路徑，且尚未含 /blog 前綴
        if path.starts_with('/')
            && !path.starts_with("//")
            && path != "/blog"
            && !path.starts_with("/blog/")
        {
            let new_path = format!("/blog{}", path);
            // 重建 URI：保留原本的 scheme / authority / query / fragment
            let mut parts = uri.into_parts();
            let path_and_query = match parts.path_and_query {
                Some(pq) => {
                    if let Some(q) = pq.query() {
                        format!("{}?{}", new_path, q)
                    } else {
                        new_path
                    }
                }
                None => new_path,
            };
            parts.path_and_query = path_and_query.parse().ok();
            Uri::from_parts(parts)
                .ok()
                .map(|u| u.to_string())
        } else {
            None
        }
    } else {
        // 解析失敗時退回字串處理（純路徑 + query/fragment）
        let suffix_pos = location.find(|c| c == '?' || c == '#');
        let (path, suffix) = match suffix_pos {
            Some(pos) => (&location[..pos], &location[pos..]),
            None => (location.as_str(), ""),
        };

        if path.starts_with('/')
            && !path.starts_with("//")
            && path != "/blog"
            && !path.starts_with("/blog/")
        {
            Some(format!("/blog{}{}", path, suffix))
        } else {
            None
        }
    };

    if let Some(corrected_location) = corrected {
        if let Ok(value) = HeaderValue::from_str(&corrected_location) {
            response.headers_mut().insert(LOCATION, value);
        }
    }

    response
}
/// 統一處理 Blog 根目錄網址。
///
/// /blog/  ->  /blog
///
/// 使用 308 Permanent Redirect，
/// 保留 HTTP 方法及請求內容。
///
/// 不影響 /blog 下的文章、標籤、分類及其他檔案。
async fn normalize_blog_root(
    request: Request<Body>,
    next: Next,
) -> Response {
    if request.uri().path() == "/blog/" {
        return Redirect::permanent("/blog").into_response();
    }

    next.run(request).await
}

/// 建立 RustCMS 主 Router。
pub fn create_router(state: AppState) -> Router {
    let static_path = state.base_dir.join("static");
    let images_path = state.base_dir.join("images");
    let downloads_path = state.base_dir.join("downloads");
    let content_static = state.base_dir.join("content");

    // Zola Blog 靜態網站目錄
    let blog_path = state.base_dir.join("blog");

    // Unveil 簡報系統目錄
    let public_path = state.base_dir.join("public");

    // --------------------------------------------------
    // Zola Blog 專用 Router
    // --------------------------------------------------

    let blog_router = Router::new()
        .fallback_service(
            ServeDir::new(blog_path)
                .append_index_html_on_directories(true),
        )
        .layer(middleware::from_fn(fix_blog_redirect));

    // --------------------------------------------------
    // CMS 主 Router
    // --------------------------------------------------

    Router::new()

        // ----------------------------------------------
        // CMS 首頁及頁面路由
        // ----------------------------------------------

        .route("/", get(page::index))
        .route("/get_page/:heading", get(page::get_page))
        .route("/get_page/:heading/:edit", get(page::get_page_edit))
        .route("/sitemap", get(page::sitemap))
        .route("/search", get(page::search))
        .route("/search_form", get(page::search))
        .route("/login", get(page::login_page))
        .route("/check_login", post(page::check_login))
        .route("/logout", get(page::logout))

        // ----------------------------------------------
        // CMS 編輯器
        // ----------------------------------------------

        .route("/edit_all", get(editor::edit_all))
        .route("/edit_page", get(editor::edit_all))
        .route("/save_single_page", post(editor::save_single_page))
        .route("/file_selector", get(editor::file_selector))

        // ----------------------------------------------
        // CMS 管理功能
        // ----------------------------------------------

        .route("/save_page", post(admin::save_page))
        .route("/edit_config", get(admin::edit_config))
        .route("/save_config", post(admin::save_config))
        .route("/acpform", get(admin::acpform))
        .route("/do_acp", post(admin::do_acp))
        .route("/start_static", get(admin::start_static))

        // ----------------------------------------------
        // CMS 靜態頁面產生
        // ----------------------------------------------

        .route("/generate_pages", get(generate::generate_pages))

        // ----------------------------------------------
        // CMS 檔案管理
        // ----------------------------------------------

        .route("/image_list", get(files::image_list))
        .route("/download_list", get(files::download_list))
        .route("/image_upload_form", get(files::image_upload_form))
        .route("/file_upload_form", get(files::file_upload_form))
        .route("/imageaxupload", post(files::imageaxupload))
        .route("/fileaxupload", post(files::fileaxupload))
        .route("/imageuploadform", get(files::image_upload_form))
        .route("/fileuploadform", get(files::file_upload_form))
        .route("/delete_file", post(files::delete_file))

        // ----------------------------------------------
        // CMS 原有靜態資源
        // ----------------------------------------------

        .nest_service(
            "/static",
            ServeDir::new(static_path),
        )
        .nest_service(
            "/images",
            ServeDir::new(images_path),
        )
        .nest_service(
            "/downloads",
            ServeDir::new(downloads_path),
        )
        .nest_service(
            "/content",
            ServeDir::new(content_static),
        )

        // ----------------------------------------------
        // Zola Blog 靜態網站
        // ----------------------------------------------

        .nest("/blog", blog_router)

        // ----------------------------------------------
        // Unveil 簡報系統
        // ----------------------------------------------

        .nest_service(
            "/public",
            ServeDir::new(public_path)
                .append_index_html_on_directories(true),
        )

        // ----------------------------------------------
        // 設定 AppState
        // ----------------------------------------------

        .with_state(state)

        // 統一處理 /blog/ 根目錄
        .layer(middleware::from_fn(normalize_blog_root))
}
