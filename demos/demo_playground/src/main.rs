use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
}

struct AppState {
    users: Mutex<Vec<User>>,
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome to the API!")
}

#[get("/users")]
async fn get_users(data: web::Data<AppState>) -> impl Responder {
    let users = data.users.lock().unwrap();
    HttpResponse::Ok().json(users.clone())
}

#[post("/users")]
async fn create_user(user: web::Json<User>, data: web::Data<AppState>) -> impl Responder {
    let mut users = data.users.lock().unwrap();
    users.push(user.into_inner());
    HttpResponse::Created().body("User created")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        users: Mutex::new(vec![]),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(index)
            .service(get_users)
            .service(create_user)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
