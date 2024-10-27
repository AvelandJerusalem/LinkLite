mod schema;
use actix_web::{
    error::ErrorNotFound,
    get, post,
    web::{self, Redirect},
    App, HttpServer, Result,
};
use diesel::{
    insert_into,
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use dotenvy::dotenv;
use schema::urls;
use serde::{Deserialize, Serialize};
use std::env;
//The connection pool type - used in services
type DbPool = web::Data<Pool<ConnectionManager<SqliteConnection>>>;

//The request struct defines the format of requests during abbreviation creation
#[derive(Serialize, Deserialize)]
struct Request {
    url: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //Use the .env file to define the database url env var
    dotenv().ok();
    //Read the URL
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    //Create a connection manager
    let manager = ConnectionManager::<SqliteConnection>::new(url);
    //Create the connection pool
    let pool = Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool");
    //Run the http server, including the redirect, and create routes and the DB pool
    HttpServer::new(move || {
        App::new()
            .service(redirect)
            .service(create)
            .app_data(web::Data::new(pool.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

//Perform the creation of new shortened URLs
#[post("/create")]
async fn create(pool: DbPool, form: web::Form<Request>) -> Result<String> {
    let url = form.into_inner().url;
    match web::block(move || {
        let mut conn = pool.get().expect("Couldn't get DB connection");
        insert_into(urls::table)
            .values(urls::url.eq(url))
            .get_result::<(i32, String)>(&mut conn)
    })
    .await?
    {
        Ok(u) => Ok(format!("http://127.0.0.1/{}", u.0)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(format!(
            "Failed to create entry: {e:?}"
        ))),
    }
}

#[get("/{id}")]
async fn redirect(pool: DbPool, path: web::Path<i32>) -> Result<Redirect> {
    let id = path.into_inner();
    if let Some(url) = web::block(move || {
        let mut conn = pool.get().expect("Couldn't get DB connection");
        urls::table
            .select(urls::url)
            .filter(urls::id.eq(id))
            .get_result::<String>(&mut conn)
            .ok()
    })
    .await?
    {
        Ok(Redirect::to(url).permanent())
    } else {
        Err(ErrorNotFound(
            "URL not found - check the link is correctly typed.",
        ))
    }
}
