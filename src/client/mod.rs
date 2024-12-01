pub mod routes;
pub fn routes() -> Vec<rocket::Route> {
    routes![
        routes::receive_status,
    ]
}