mod schema;
use actix_cors::Cors;
use actix_web::{
    delete,
    error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized},
    get,
    http::header::{HeaderName, CONTENT_TYPE},
    post,
    web::{self, Redirect},
    App, HttpRequest, HttpResponse, HttpServer, Responder, Result,
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

#[derive(Serialize, Deserialize)]
struct Auth {
    #[serde(rename = "Authorization")]
    auth: String,
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
        //Define liberal CORS rules as this is a public API
        //Within the closure as cannot be passed safely between threads
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "DELETE"])
            .allowed_header(CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .service(redirect)
            .service(create)
            .service(delete)
            .app_data(web::Data::new(pool.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// Create new shortened URLs - using an integer as the ID - for the scale of this application
// hashes are longer
#[post("/")]
async fn create(pool: DbPool, form: web::Json<Request>) -> Result<String> {
    let url_in = form.into_inner().url;
    let server_url = env::var("SERVER_URL").expect("SERVER_URL must be set");

    match reqwest::get(url_in.clone()).await {
        Ok(resp) => {
            if !resp.status().is_success() {
                return Err(ErrorBadRequest(format!(
                    "Failed to query from URL - {}",
                    resp.status().to_string()
                )));
            }
        }
        Err(e) => return Err(ErrorBadRequest(format!("Failed to check URL - {e}"))),
    };

    match web::block(move || {
        let mut conn = pool.get().expect("Couldn't get DB connection");
        if let Ok(res) = urls::table
            .select(urls::id)
            .filter(urls::url.eq(&url_in))
            .get_result::<i32>(&mut conn)
        {
            return Ok((res, String::default()));
        }
        insert_into(urls::table)
            .values(urls::url.eq(url_in))
            .get_result::<(i32, _)>(&mut conn)
    })
    .await?
    {
        Ok(u) => Ok(format!("{}/{}", server_url, u.0)),
        Err(e) => Err(ErrorInternalServerError(format!(
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
        Err(ErrorNotFound("URL not found"))
    }
}

#[delete("/{id}")]
async fn delete(pool: DbPool, path: web::Path<i32>, req: HttpRequest) -> Result<impl Responder> {
    let headers = req.headers();

    if let Some(auth) = headers.get(HeaderName::from_static("x-auth")) {
        let secret = env::var("DELETE_SECRET").expect("DELETE_SECRET must be set");
        if secret != auth.to_str().unwrap() {
            return Err(ErrorUnauthorized("Invalid x-auth header"));
        }
    } else {
        return Err(ErrorUnauthorized("The x-auth header must be set"));
    }

    if let Err(e) = web::block(move || {
        let mut conn = pool.get().expect("Couldn't get DB connection");
        diesel::delete(urls::table.filter(urls::id.eq(path.into_inner()))).execute(&mut conn)
    })
    .await?
    {
        Err(ErrorNotFound(e))
    } else {
        Ok(HttpResponse::Ok())
    }
}
