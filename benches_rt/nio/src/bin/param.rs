use ohkami::prelude::*;

#[nio::main]
async fn main() {
    Ohkami::new((
        "/user/:id"
            .GET(|id: String| async {id}),
    )).howl("0.0.0.0:3000").await
}